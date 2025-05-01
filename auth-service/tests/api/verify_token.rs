use super::helpers::TestApp;
use serde_json::json;
use fake::{faker::internet::en::SafeEmail, faker::internet::en::Password as FakerPassword, Fake};
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn verify_token_returns_200() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();
       
    // Signup a new user
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": false
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

    // Get the auth cookie from login response
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    let login_token = auth_cookie.value();

    // Verify the token
    let verify_token_request = serde_json::json!({
        "token": login_token
    });
    let response = app.post_verify_token(&verify_token_request).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let response = app.post_verify_token(&json!({})).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();
       
    // Signup a new user
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": false
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

    // Set the auth cookie to an invalid token
    let response = app.post_verify_token(&json!({
        "token": "invalid_token"
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();
       
    // Signup a new user
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": false
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

    // Get the auth cookie from login response
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
    let login_token = auth_cookie.value();

    // Logout to add the token from the banned token store
    let logout_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_logout(&logout_request).await;
    assert_eq!(response.status().as_u16(), 200);

    // Verify the token
    let verify_token_request = serde_json::json!({
        "token": login_token
    });
    let response = app.post_verify_token(&verify_token_request).await;
    assert_eq!(response.status().as_u16(), 401);
}