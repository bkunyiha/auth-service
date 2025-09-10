use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(skip_all)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value().to_owned();

    // Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    match validate_token(&token, state.banned_token_store.clone()).await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    };

    // Add the token to the banned token store
    let mut banned_token_store = state.banned_token_store.write().await;
    let _ = banned_token_store
        .add_token(token.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError);

    // Remove the JWT cookie from the `CookieJar`
    let mut cookie_for_removal = Cookie::from(JWT_COOKIE_NAME);
    cookie_for_removal.set_path("/"); // Needed for https context removal
    let updated_jar = jar.remove(cookie_for_removal);
    // Return the updated cookie jar and a 200 status code
    (updated_jar, Ok(StatusCode::OK))
}
