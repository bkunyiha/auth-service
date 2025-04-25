
use auth_service::{
    Application,
    app_state::{AppState, UserStoreType, BannedTokenStoreType},
    services::{HashmapUserStore, HashsetBannedTokenStore},
    utils::constants::test,
};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;
use reqwest::cookie::Jar;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
}

impl TestApp {
    pub async fn new() -> Self {

        let user_store: UserStoreType = Arc::new(RwLock::new(Box::new(HashmapUserStore::default())));
        let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::default())));
        let app_state: AppState = AppState::new(user_store, banned_token_store.clone());

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
            .json(&json!({ "email": email, "password": password, "requires2FA": true }))
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

    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
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
}