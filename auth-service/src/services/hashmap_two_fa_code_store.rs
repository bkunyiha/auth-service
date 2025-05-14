use std::collections::HashMap;

use crate::domain::user::Email;
use crate::services::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};
use color_eyre::eyre::eyre;

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    #[tracing::instrument(name = "Adding 2-FA-Code To Local Memery 2FA-Code Cache", skip_all)]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            return Err(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Email already exists in the store"
            )));
        }
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    #[tracing::instrument(name = "Removing 2-FA-Code From Local Memery 2FA-Code Cache", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            self.codes.remove(&email);
            Ok(())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    #[tracing::instrument(name = "Getting 2-FA-Code From Local Memery 2FA-Code Cache", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::TwoFACodeStoreType;
    use crate::domain::user::Email;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use secrecy::Secret;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_add_code() {
        //let mut store = HashmapTwoFACodeStore::default();
        let two_fa_code_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::default())));
        let mut store = two_fa_code_store.write().await;
        let email_secret: Secret<String> = Secret::new(SafeEmail().fake());
        let email = Email::parse(email_secret).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // First add should succeed
        assert!(store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .is_ok());

        // Second add with same email should fail
        assert_eq!(
            store.add_code(email, login_attempt_id, code).await,
            Err(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Email already exists in the store"
            )))
        );
    }

    #[tokio::test]
    async fn test_remove_code() {
        let two_fa_code_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::default())));
        let mut store = two_fa_code_store.write().await;
        let email = Email::parse(Secret::new(SafeEmail().fake())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // Remove non-existent code should fail
        assert_eq!(
            store.remove_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );

        // Add code
        store
            .add_code(email.clone(), login_attempt_id, code)
            .await
            .unwrap();

        // Remove existing code should succeed
        assert!(store.remove_code(&email).await.is_ok());

        // Remove again should fail
        assert_eq!(
            store.remove_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );
    }

    #[tokio::test]
    async fn test_get_code() {
        let two_fa_code_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::default())));
        let mut store = two_fa_code_store.write().await;
        let email = Email::parse(Secret::new(SafeEmail().fake())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // Get non-existent code should fail
        assert_eq!(
            store.get_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );

        // Add code
        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        // Get existing code should succeed
        let result = store.get_code(&email).await;
        assert!(result.is_ok());
        let (retrieved_id, retrieved_code) = result.unwrap();
        assert_eq!(retrieved_id, login_attempt_id);
        assert_eq!(retrieved_code, code);
    }
}
