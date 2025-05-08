use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod test {
    pub const APP_SERVICE_HOST: &str = "127.0.0.1:0";
}

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref APP_SERVICE_HOST: String = set_app_service_host();
    pub static ref DATABASE_URL: String = set_db_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const APP_SERVICE_HOST_ENV_VAR: &str = "APP_SERVICE_HOST";
    pub const DB_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

// Set the JWT secret from the environment variable
fn set_token() -> String {
    dotenv().ok(); // Load environment variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    secret
}

// Set the app host from the environment variable
fn set_app_service_host() -> String {
    dotenv().ok(); // Load environment variables
    let prod_host = std_env::var(env::APP_SERVICE_HOST_ENV_VAR).expect("APP_SERVICE_HOST must be set.");
    if prod_host.is_empty() {
        panic!("APP_SERVICE_HOST must not be empty.");
    }
    prod_host
}

// Set the db url from the environment variable
fn set_db_url() -> String {
    dotenv().ok(); // Load environment variables
    let prod_host = std_env::var(env::DB_URL_ENV_VAR).expect("DB_URL must be set.");
    if prod_host.is_empty() {
        panic!("DB_URL must not be empty.");
    }
    prod_host
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}