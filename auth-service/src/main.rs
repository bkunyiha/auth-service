use auth_service::{
    Application,
    app_state::{AppState, UserStoreType, BannedTokenStoreType}, 
    services::{HashmapUserStore, HashsetBannedTokenStore}, 
};

use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(Box::new(HashmapUserStore::default())));
    let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::default())));
    let app_state: AppState = AppState::new(user_store, banned_token_store);

    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

