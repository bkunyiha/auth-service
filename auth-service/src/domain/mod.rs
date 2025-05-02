pub mod user;
pub mod error;
pub mod data_stores;
pub mod email_client;
pub mod mock_email_client;

// re-export items from sub-modules
pub use user::{User, Email, Password};
pub use email_client::*;
pub use mock_email_client::MockEmailClient;
pub use error::{AuthAPIError, AuthAPIError::*};
pub use data_stores::{
    UserStore, UserStoreError, 
    BannedTokenStore, BannedTokenStoreError, 
    TwoFACodeStore, TwoFACodeStoreError};