use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, Password, User};
use axum::{
    debug_handler, extract::Json, extract::State, http::StatusCode, response::IntoResponse,
};
use secrecy::Secret;
use serde::{Deserialize, Serialize};

#[debug_handler]
#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(&email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let user = User::new(email, password, request.requires_2fa);

    if let Err(e) = user_store.add_user(user).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl SignupRequest {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        Self {
            email: Secret::new(email),
            password: Secret::new(password),
            requires_2fa,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SignupResponse {
    pub message: String,
}
