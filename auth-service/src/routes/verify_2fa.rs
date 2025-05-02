use axum::http::StatusCode;
use serde::Deserialize;

use axum::{extract::State, extract::Json, debug_handler};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},    
    utils::auth::generate_auth_cookie,
};

#[debug_handler]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<StatusCode, AuthAPIError>) {

    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };
    let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id) {
        Ok(id) => id,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };
    let two_fa_code = match TwoFACode::parse(request.two_fa_code) {
        Ok(code) => code,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let two_fa_code_store = state.two_fa_code_store.read().await;
    // Validate that the `login_attempt_id` and `two_fa_code`
    // in the request body matches values in the `code_tuple`. 
    // If not, return a `AuthAPIError::IncorrectCredentials`.
    let _ = match two_fa_code_store.get_code(&email).await {
        Ok((id, code)) if id == login_attempt_id && code == two_fa_code  => (),
        Ok((id, _)) if id != login_attempt_id  => return (jar, Err(AuthAPIError::InvalidCredentials)),
        Ok(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };
    let updated_jar = jar.add(auth_cookie);

    //  Remove 2FA code from the code store after successful authentication.
    //let _ = match two_fa_code_store.remove_code(&email).await {
    //    Ok(_) => (),
    //    Err(_) => return (updated_jar, Err(AuthAPIError::UnexpectedError)),
    //};

    // Return the updated cookie jar and a 200 status code
    (updated_jar, Ok(StatusCode::OK))
}

#[derive(Deserialize, Debug)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
    
}

impl Verify2FARequest {
    pub fn new(email: String, login_attempt_id: String, two_fa_code: String) -> Self {
        Self { email, login_attempt_id, two_fa_code}
    }
}
