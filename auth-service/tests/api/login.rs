use crate::helpers::TestApp;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use auth_service::{domain::Email, routes::TwoFactorAuthResponse};
use fake::{Fake, faker::internet::en::Password as FakerPassword, faker::internet::en::SafeEmail};
use secrecy::SecretBox;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn should_return_200_if_valid_credentials() {
    let app = TestApp::new().await;
    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": false
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    let signup_body = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let email_str: String = SafeEmail().fake();
    let short_password_str: String = FakerPassword(std::ops::Range { start: 1, end: 7 }).fake();

    let login_request = serde_json::json!({
        "email": email_str,
        "password": short_password_str,
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let login_request = serde_json::json!({
        "email": "",
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_401_if_invalid_credentials() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    // signup a new user with 2FA enabled
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    // Define an expectation for the mock server
    Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
        .and(method("POST")) // Expect the HTTP method to be POST
        .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status for email service whhich is different from reponse in this route
        .expect(1) // Expect this request to be made exactly once
        .mount(&app.email_server) // Mount this expectation on the mock email server
        .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding

    // login the user
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 206);
}

#[tokio::test]
async fn should_return_2fa_required_message_when_2fa_enabled() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    // signup a new user with 2FA enabled
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // login the user
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    // assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
    let two_fa_store = app.two_fa_code_store.read().await;
    let email = Email::parse(SecretBox::new(Box::new(email_str))).unwrap();
    let (stored_login_attempt_id, _) = two_fa_store.get_code(&email).await.unwrap();
    assert_eq!(json_body.login_attempt_id, stored_login_attempt_id.as_ref());
}
