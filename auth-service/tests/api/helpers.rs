use auth_service::Application;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::app_state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json::json;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::default();
        let user_store = Arc::new(RwLock::new(user_store));
        let app_state = AppState {
            user_store,
        };
        let app = Application::build(app_state, "0.0.0.0:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread. 
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        // Create new `TestApp` instance and return it
        Self {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // TODO: Implement helper functions for all other routes (signup, login, logout, verify-2fa, and verify-token)
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

    pub async fn login(&self, email: &str, password: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(&json!({ "email": email, "password": password }))
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

    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }   
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}