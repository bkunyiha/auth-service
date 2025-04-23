pub mod user;
pub mod error;
pub mod data_stores;
pub use user::{User, Email, Password};
pub use error::{AuthAPIError, AuthAPIError::*};
pub use data_stores::{UserStore, UserStoreError};