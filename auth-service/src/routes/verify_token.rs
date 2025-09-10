use crate::app_state::AppState;
use crate::domain::AuthAPIError;
use crate::utils::{auth::validate_token, constants::JWT_COOKIE_NAME};
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[tracing::instrument(skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    let req_token = request.token;

    // Check if token is banned first - use block scope to control lifetime
    {
        let banned_token_store = state.banned_token_store.read().await;
        match banned_token_store.get_token(&req_token).await {
            Ok(_) => return Err(AuthAPIError::InvalidToken),
            Err(_) => Ok(()),
        }?;
    } // banned_token_store is dropped here, releasing the read lock

    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Err(AuthAPIError::MissingToken),
    };

    let token = cookie.value().to_owned();

    // Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    match validate_token(&token, state.banned_token_store).await {
        Ok(_) => {
            if token == req_token {
                Ok(StatusCode::OK)
            } else {
                Err(AuthAPIError::InvalidToken)
            }
        }
        Err(_) => return Err(AuthAPIError::InvalidToken),
    }
}

#[derive(Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

impl VerifyTokenRequest {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct VerifyTokenResponse {}
