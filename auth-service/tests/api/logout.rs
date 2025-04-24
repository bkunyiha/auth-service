use crate::helpers::TestApp;
use fake::{faker::internet::en::SafeEmail, faker::internet::en::Password as FakerPassword, Fake};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

#[tokio::test]
async fn logout_returns_200() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

    // Signup a new user
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    // Login the user
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 200);

    // Logout the user
    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

    let login_body = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_logout(&login_body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();

    let response = app.post_login(&serde_json::json!({
        "email": email_str,
        "password": password_str,
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();
       
    // Signup a new user
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    // Login the user
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 200);

    // First logout
    let logout_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });

    let response = app.post_logout(&logout_request).await;
    assert_eq!(response.status().as_u16(), 200);

    // Second logout
    let response = app.post_logout(&logout_request).await;
    assert_eq!(response.status().as_u16(), 400);
}