use std::str::FromStr;
use auth_service::{
    Application,
    app_state::{AppState, UserStoreType, BannedTokenStoreType, TwoFACodeStoreType, EmailClientType},
    services::{HashsetBannedTokenStore, HashmapTwoFACodeStore, data_stores::PostgresUserStore},
    utils::constants::{test, DATABASE_URL},
    domain::MockEmailClient,
    get_postgres_pool,
};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;
use reqwest::cookie::Jar;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}, PgConnection, Executor, Connection};
use uuid::Uuid;

pub struct DBName(String);

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub db_name: DBName
}

impl TestApp {
    pub async fn new() -> Self {

        let (pg_pool, db_name) = configure_postgresql().await;
        let user_store: UserStoreType = Arc::new(RwLock::new(Box::new(PostgresUserStore::new(pg_pool))));
        let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::default())));
        let two_fa_code_store: TwoFACodeStoreType = Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::default())));
        let email_client: EmailClientType = Arc::new(RwLock::new(Box::new(MockEmailClient)));

        let app_state: AppState = AppState::new(user_store, banned_token_store.clone(), two_fa_code_store.clone(), email_client.clone());

        let app = Application::build(app_state, test::APP_SERVICE_HOST)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        // This is a new feature in Rust 1.71.0
        // and is not yet available in the stable version of Rust.        
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        // Create a new cookie jar and HTTP client
        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();
        // Create new `TestApp` instance and return it
        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            db_name
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // Implement helper functions for all other routes (signup, login, logout, verify-2fa, and verify-token)
    pub async fn signup(&self, email: &str, password: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(&json!({ "email": email, "password": password, "requires2FA": false }))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        println!("Dropping test db {}!", self.db_name.0);
        let db_name = self.db_name.0.clone();
        tokio::task::spawn_blocking(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { delete_database(&db_name).await });
        });
        println!("Dropped test db!");
    }
}

async fn configure_postgresql() -> (PgPool, DBName) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    let pg_pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!");
    
    (pg_pool, DBName(db_name))
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");


    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}