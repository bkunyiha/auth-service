pub mod email_client;
pub mod error;
pub mod mock_email_client;
pub mod user;

// re-export items from sub-modules
pub use email_client::*;
pub use error::{AuthAPIError, AuthAPIError::*};
pub use mock_email_client::MockEmailClient;
pub use user::{Email, Password, User};
