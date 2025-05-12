use crate::app_state::AppState;
use crate::utils::tracing::{make_span_with_request_id, on_request, on_response};
use axum::routing::post;
use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

mod login;
mod logout;
mod signup;
mod verify_2fa;
mod verify_token;

// re-export items from sub-modules
pub use login::*;
pub use logout::*;
pub use signup::*;
pub use verify_2fa::*;
pub use verify_token::*;

pub fn get_routes(app_state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/verify-2fa", post(verify_2fa))
        .route("/verify-token", post(verify_token))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
        .layer(cors)
        .layer(
            // Add a TraceLayer for HTTP requests to enable detailed tracing
            // This layer will create spans for each request using the make_span_with_request_id function,
            // and log events at the start and end of each request using on_request and on_response functions.
            TraceLayer::new_for_http()
                .make_span_with(make_span_with_request_id)
                .on_request(on_request)
                .on_response(on_response),
        )
}
