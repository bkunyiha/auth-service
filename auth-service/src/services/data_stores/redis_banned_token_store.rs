use redis::{Commands, Connection, RedisResult};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    services::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};
use color_eyre::eyre::{Context, Result};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Adding Token To Keystore Cache", skip_all)]
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        let token_key = get_key(&token);
        let value = true;
        // 2. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        // The value should simply be a `true` (boolean value).
        // The expiration time should be set to TOKEN_TTL_SECONDS.
        // NOTE: The TTL is expected to be a u64 so you will have to cast TOKEN_TTL_SECONDS to a u64.
        // Return BannedTokenStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let _: () = self
            .conn
            .write()
            .await
            .set_ex(&token_key, value, ttl)
            .wrap_err("failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(name = "Getting Token From Keystore Cache", skip_all)]
    async fn get_token(&self, token: &str) -> Result<String, BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(token);
        // 2. Call the get command on the Redis connection to get the value of the key.
        let result: RedisResult<Option<String>> = self.conn.write().await.get(key);
        if let Err(e) = result {
            return Err(BannedTokenStoreError::UnexpectedError(e.into()));
        }
        // 3. Return the value as a &str.
        // 4. Return BannedTokenStoreError::TokenNotFound if the key does not exist.
        // 5. Return BannedTokenStoreError::UnexpectedError if the call to get fails.
        match result {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(BannedTokenStoreError::TokenNotFound),
            Err(e) => Err(BannedTokenStoreError::UnexpectedError(e.into())),
        }
    }

    #[tracing::instrument(name = "Checking If Token In Keystore Cache", skip_all)]
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        // Check if the token exists by calling the exists method on the Redis connection
        let token_key = get_key(token);

        let is_banned: bool = self
            .conn
            .write()
            .await
            .exists(&token_key)
            .wrap_err("failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(is_banned)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
