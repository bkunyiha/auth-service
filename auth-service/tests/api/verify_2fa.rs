use crate::helpers::TestApp;
use auth_service::domain::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::services::{LoginAttemptId, TwoFACode};
use fake::{faker::internet::en::Password as FakerPassword, faker::internet::en::SafeEmail, Fake};
use serde_json;

#[tokio::test]
async fn verify_2fa_returns_200() {
    let app = TestApp::new().await;

    let email_str: String = SafeEmail().fake();
    let email = Email::parse(email_str.clone()).unwrap();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    // signup a new user with 2FA enabled
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

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

    // Since you can only have one mutable RWLock then, wrap the code in a block to drop lock
    #[allow(unused_assignments)]
    let mut stored_login_attempt_id: LoginAttemptId = LoginAttemptId::default();
    #[allow(unused_assignments)]
    let mut two_factor_code: TwoFACode = TwoFACode::default();
    {
        // assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
        let two_fa_store = app.two_fa_code_store.read().await;
        let (id, code) = two_fa_store.get_code(&email).await.unwrap();
        stored_login_attempt_id = id;
        two_factor_code = code;
        assert_eq!(json_body.login_attempt_id, stored_login_attempt_id.as_ref());
    }

    // Verify 2 FA Auth
    let request = serde_json::json!({
        "email": email_str.clone(),
        "loginAttemptId": stored_login_attempt_id.as_ref(),
        "2FACode": two_factor_code.as_ref()
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let malformed_request = serde_json::json!({
        "loginAttemptId": "",  // Empty login attempt ID
        "code": "123"  // Invalid code length
    });

    let response = app.post_verify_2fa(&malformed_request).await;
    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
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

    // login the user
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;

    response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    // Try to verify with incorrect login attempt ID
    let request = serde_json::json!({
        "email": email_str,
        "loginAttemptId": "00000000-0000-0000-0000-000000000000",  // Incorrect login attempt ID
        "2FACode": "000000"  // Incorrect code
    });

    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
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

    // Try to verify with incorrect 2FA code
    let request = serde_json::json!({
        "email": email_str,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": "111111"  // Valid But Incorrect code
    });

    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;
    let email_str: String = SafeEmail().fake();
    let password_str: String = FakerPassword(std::ops::Range { start: 8, end: 30 }).fake();

    // 1) signup a new user with 2FA enabled
    let signup_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status().as_u16(), 201);

    // 2) Login
    let login_request = serde_json::json!({
        "email": email_str,
        "password": password_str,
    });
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status().as_u16(), 206);
    let login_json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    // *************************
    // two_fa_code_store.read()
    // *************************
    // 3) Get the first 2FA code from store
    let email = Email::parse(email_str.clone()).unwrap();
    let (_, two_factor_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();
    //let (_, two_factor_code) = two_fa_store.get_code(&email).await.unwrap();
    // 4) Verify with the 2FA code

    let request = serde_json::json!({
        "email": email_str,
        "loginAttemptId": login_json_body.login_attempt_id,
        "2FACode": two_factor_code.as_ref()
    });

    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status().as_u16(), 200);

    // 5) Try to verify with the old 2FA code
    let request = serde_json::json!({
        "email": email_str,
        "loginAttemptId": login_json_body.login_attempt_id,
        "2FACode": two_factor_code.as_ref()
    });

    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status().as_u16(), 401);
}
