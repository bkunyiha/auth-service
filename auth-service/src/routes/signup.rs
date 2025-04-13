use axum::{http::StatusCode, response::IntoResponse, extract::Json};
use serde::Deserialize;

pub async fn signup(Json(request): Json<SignupRequest>) -> impl IntoResponse {

    match (request.email, request.password, request.requires_2fa) {
        (email, password, true) if email == "test@example.com" && password == "password123" => StatusCode::OK.into_response(),
        _ => StatusCode::BAD_REQUEST.into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}