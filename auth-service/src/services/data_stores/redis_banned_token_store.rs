use std::sync::Arc;
use redis::{Commands, Connection, RedisResult};
use tokio::sync::RwLock;

use crate::{
    services::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

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
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(&token);
        // 2. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL). 
        // The value should simply be a `true` (boolean value).
        // The expiration time should be set to TOKEN_TTL_SECONDS.
        // NOTE: The TTL is expected to be a u64 so you will have to cast TOKEN_TTL_SECONDS to a u64. 
        // Return BannedTokenStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let ttl = TOKEN_TTL_SECONDS as u64;
        let result: RedisResult<()> = self.conn.write().await.set_ex(key, true, ttl);
        if result.is_err() {
            return Err(BannedTokenStoreError::UnexpectedError);
        }
        Ok(())
    }

    async fn get_token(&self, token: &String) -> Result<String, BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(token);
        // 2. Call the get command on the Redis connection to get the value of the key.
        let result: RedisResult<Option<String>> = self.conn.write().await.get(key);
        if result.is_err() {
            return Err(BannedTokenStoreError::UnexpectedError);
        }
        // 3. Return the value as a &str.
        // 4. Return BannedTokenStoreError::TokenNotFound if the key does not exist.
        // 5. Return BannedTokenStoreError::UnexpectedError if the call to get fails.
        match result {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(BannedTokenStoreError::TokenNotFound),
            Err(_) => Err(BannedTokenStoreError::UnexpectedError),
        }
    }
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        // Check if the token exists by calling the exists method on the Redis connection
        let key = get_key(token);
        let result: RedisResult<bool> = self.conn.write().await.exists(&key);
        match result {
            Ok(exists) => Ok(exists),
            Err(_) => Err(BannedTokenStoreError::UnexpectedError),
        }
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}