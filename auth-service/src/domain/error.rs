use color_eyre::eyre::Report;
use thiserror::Error;

/*
 * The thiserror crate simplifies the creation of custom error types by providing 
 * the #[derive(Error)] attribute macro, which automatically implements the Error trait 
 * for the AuthAPIError enum. Each variant of the enum has a custom Display error message 
 * specified by the #[error(...)] attribute.
 */
#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}