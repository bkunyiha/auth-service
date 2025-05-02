use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::data_stores::{
    UserStore,
    BannedTokenStore,
    TwoFACodeStore
};
use crate::domain::MockEmailClient;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<Box<dyn UserStore>>>;
pub type BannedTokenStoreType = Arc<RwLock<Box<dyn BannedTokenStore>>>;
pub type TwoFACodeStoreType = Arc<RwLock<Box<dyn TwoFACodeStore>>>;
pub type EmailClientType = Arc<RwLock<Box<MockEmailClient>>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType, 
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
    ) -> Self {
        Self { 
            user_store, 
            banned_token_store, 
            two_fa_code_store,
            email_client,
        }
    }
}