use axum::{extract::State, http::StatusCode, response::IntoResponse, extract::Json};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::domain::User;

pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> impl IntoResponse {

    match (request.email, request.password, request.requires_2fa) {
        (email, password, true) if email == "test@example.com" && password == "password123" => {
            let user = User::new(email, password, true);
            let mut user_store = state.user_store.write().await;

            // TODO: Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.
            user_store.add_user(user).unwrap();

            
            let response: Json<SignupResponse> = Json(SignupResponse {
                message: "User created successfully!".to_string(),
            });
            (StatusCode::CREATED, response)
        },
        _ => {
            let response = Json(SignupResponse {
                message: "User Creation Failed!".to_string(),
            });
            (StatusCode::BAD_REQUEST, response)
        },
    }
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SignupResponse {
    pub message: String,
}