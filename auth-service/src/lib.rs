pub mod routes;
pub mod domain;
pub mod services;
pub mod app_state;

use std::error::Error;
use axum::Router;
use axum::serve::Serve;
use app_state::AppState;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, routes::get_routes(app_state));

        // Create a new Application instance and return it
        Ok(Application{server,address})
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}