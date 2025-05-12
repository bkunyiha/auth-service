use color_eyre::eyre::Report;
use thiserror::Error;

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn get_token(&self, token: &String) -> Result<String, BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Token already exists")]
    TokenAlreadyExists,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                BannedTokenStoreError::TokenAlreadyExists,
                BannedTokenStoreError::TokenAlreadyExists
            ) | (
                BannedTokenStoreError::TokenNotFound,
                BannedTokenStoreError::TokenNotFound
            ) | (
                BannedTokenStoreError::UnexpectedError(_),
                BannedTokenStoreError::UnexpectedError(_)
            )
        )
    }
}
