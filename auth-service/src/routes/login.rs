use axum::{debug_handler, extract::Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{email_client::EmailClient, AuthAPIError, Email, Password},
    services::{LoginAttemptId, TwoFACode},
    utils::auth::generate_auth_cookie,
};

#[debug_handler]
#[tracing::instrument(skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };
    let password = match Password::parse(request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = state.user_store.read().await;
    let _ = match user_store.validate_user(&email, &password).await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // Handle request based on user's 2FA configuration
    match user.requires_2fa {
        true => handle_2fa(&email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

#[tracing::instrument(skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // Return a TwoFactorAuthResponse. The message should be "2FA required".
    // The login attempt ID should be "123456". We will replace this hard-coded login attempt ID soon!
    let login_attempt_id = LoginAttemptId::default();

    let message = "2FA required".to_string();
    let tw_code = TwoFACode::default();
    // Store the ID and code in our 2FA code store. Return `AuthAPIError::UnexpectedError` if the operation fails

    if let Err(e) = state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), tw_code.clone())
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    // send 2FA code via the email client. Return `AuthAPIError::UnexpectedError` if the operation fails.
    if let Err(e) = state
        .email_client
        .write()
        .await
        .send_email(email, "2FA Code", tw_code.as_ref())
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(eyre!(e))));
    }

    // Return
    let response = TwoFactorAuthResponse {
        message,
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    };

    (
        jar,
        Ok((
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(response)),
        )),
    )
}

#[tracing::instrument(skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
    };
    let updated_jar = jar.add(auth_cookie);

    // Return the updated cookie jar and a 200 status code
    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
