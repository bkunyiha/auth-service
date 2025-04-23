use axum::{extract::State, http::StatusCode, response::IntoResponse, extract::Json, debug_handler};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, Password};


#[debug_handler]
pub async fn login(State(state): State<AppState>, Json(request): Json<LoginRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.user_store.read().await;
    let _ = user_store.validate_user(&email, &password).await.map_err(|_| AuthAPIError::IncorrectCredentials)?;

    Ok(StatusCode::OK.into_response())
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