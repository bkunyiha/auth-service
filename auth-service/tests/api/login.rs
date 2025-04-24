use crate::helpers::TestApp;
use fake::{faker::internet::en::SafeEmail, faker::internet::en::Password as FakerPassword, Fake};
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn should_return_200_if_valid_credentials() {
    let app = TestApp::new().await;
    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
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
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

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
    let short_password_str: String = FakerPassword(std::ops::Range {start: 1, end: 7}).fake();

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
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 401);
}