use axum::{extract::State, http::StatusCode, extract::Json, debug_handler};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::auth::generate_auth_cookie,
};

#[debug_handler]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar, // New!
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<StatusCode, AuthAPIError>) {

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

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let updated_jar = jar.add(auth_cookie);

    // Return the updated cookie jar and a 200 status code
    (updated_jar, Ok(StatusCode::OK))
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LoginResponse {
    pub token: String,
}