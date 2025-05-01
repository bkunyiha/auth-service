use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::data_stores::{
    UserStore,
    BannedTokenStore,
    TwoFACodeStore
};

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<Box<dyn UserStore>>>;
pub type BannedTokenStoreType = Arc<RwLock<Box<dyn BannedTokenStore>>>;
pub type TwoFACodeStoreType = Arc<RwLock<Box<dyn TwoFACodeStore>>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType, 
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType
    ) -> Self {
        Self { 
            user_store, 
            banned_token_store, 
            two_fa_code_store 
        }
    }
}