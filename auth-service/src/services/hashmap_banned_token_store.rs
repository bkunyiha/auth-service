use std::collections::HashSet;
use crate::services::{BannedTokenStore, BannedTokenStoreError};

// Create a new struct called `HashsetBannedTokenStore` containing a `token` field
// which stores a `HashSet`` of token `String`s.
// Derive the `Default` trait for `HashsetBannedTokenStore`.
#[derive(Clone,Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

impl HashsetBannedTokenStore {
    pub fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        // Return `BannedTokenStoreError::TokenAlreadyExists` if the user already exists,
        // otherwise insert the user into the HashSet and return `Ok(())`.
        if self.tokens.contains(&token) {
            return Err(BannedTokenStoreError::TokenAlreadyExists);
        }
        self.tokens.insert(token);
        Ok(())
    }

    // Implement a public method called `get_token`, which takes an
    // immutable reference to self and an token string slice as arguments.
    // This function should return a `Result` type containing either a
    // `token` String or a `BannedTokenStoreError`.
    // Return `BannedTokenStoreError::TokenNotFound` if the token can not be found.
    pub fn get_token(&self, token: &str) -> Result<String, BannedTokenStoreError> {
        self.tokens.get(token).map(|s| s.to_string()).ok_or(BannedTokenStoreError::TokenNotFound)
    }

}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.add_token(token)
    }

    async fn get_token(&self, token: &String) -> Result<String, BannedTokenStoreError> {
        self.get_token(token)
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

// Add unit tests for your `HashsetBannedTokenStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut token_store = HashsetBannedTokenStore::default();
        let token = "test_token".to_string();
        let result = token_store.add_token(token.clone());
        assert!(result.is_ok());
        let result = token_store.add_token(token.clone());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_token() {
        let mut token_store = HashsetBannedTokenStore::default();
        let token = "test_token".to_string();
        token_store.add_token(token.clone()).unwrap();
        let result = token_store.get_token(&token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), token);
    }

    #[tokio::test]
    async fn test_contains_token() {
        let mut token_store = HashsetBannedTokenStore::default();
        let token = "test_token".to_string();
        token_store.add_token(token.clone()).unwrap();
        let result = token_store.contains_token(&token).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        let result = token_store.contains_token(&"not_a_token".to_string()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
}