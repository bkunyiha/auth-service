use auth_service::{
    Application,
    app_state::{AppState, UserStoreType, BannedTokenStoreType, TwoFACodeStoreType, EmailClientType}, 
    services::{HashmapUserStore, HashsetBannedTokenStore, HashmapTwoFACodeStore}, 
    domain::MockEmailClient,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(Box::new(HashmapUserStore::default())));
    let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::default())));
    let two_fa_token_store: TwoFACodeStoreType = Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::default())));
    let email_client: EmailClientType = MockEmailClient;
    
    let app_state: AppState = AppState::new(user_store, banned_token_store, two_fa_token_store, email_client);
    
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}


