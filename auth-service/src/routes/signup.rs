use axum::{extract::State, http::StatusCode, response::IntoResponse, extract::Json, debug_handler};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::domain::{AuthAPIError, User, Email, Password};

#[debug_handler]
pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> impl IntoResponse {

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    
    let mut user_store = state.user_store.write().await;

    // TODO: early return AuthAPIError::UserAlreadyExists if email exists in user_store.
    if user_store.get_user(&email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let user = User::new(email, password, request.requires_2fa);

    // TODO: instead of using unwrap, early return AuthAPIError::UnexpectedError if add_user() fails.
    if user_store.add_user(user).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl SignupRequest {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password, requires_2fa: true }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SignupResponse {
    pub message: String,
}