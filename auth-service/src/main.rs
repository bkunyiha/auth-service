use auth_service::{
    Application,
    app_state::{AppState, UserStoreType, BannedTokenStoreType, TwoFACodeStoreType, EmailClientType}, 
    services::data_stores::{PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore}, 
    domain::MockEmailClient,
    utils::{DATABASE_URL, REDIS_HOST_NAME},
    get_postgres_pool,
    get_redis_client,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;
use auth_service::utils::init_tracing;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    
    let pg_pool = configure_postgresql().await;
    let user_store: UserStoreType = Arc::new(RwLock::new(Box::new(PostgresUserStore::new(pg_pool))));
    let banned_token_store: BannedTokenStoreType = 
        Arc::new(RwLock::new(Box::new(RedisBannedTokenStore::new(Arc::new(RwLock::new(configure_redis()))))));
    let two_fa_token_store: TwoFACodeStoreType = 
        Arc::new(RwLock::new(Box::new(RedisTwoFACodeStore::new(Arc::new(RwLock::new(configure_redis()))))));
    let email_client: EmailClientType = Arc::new(RwLock::new(Box::new(MockEmailClient)));
    
    let app_state: AppState = AppState::new(user_store, banned_token_store, two_fa_token_store, email_client);
    
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let db_url = &DATABASE_URL;
    let pg_pool = get_postgres_pool(db_url)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database! 
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}