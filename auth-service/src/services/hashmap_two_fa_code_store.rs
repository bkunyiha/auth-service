use std::collections::HashMap;

use crate::domain::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError,user::Email,};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            self.codes.remove(&email);
            return Ok(())
        } else {
            return Err(TwoFACodeStoreError::LoginAttemptIdNotFound);
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes.get(email).cloned().ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::Email;
    use fake::{faker::internet::en::SafeEmail, Fake};

    #[tokio::test]
    async fn test_add_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(email_str.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // First add should succeed
        assert!(store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await.is_ok());

        // Second add with same email should fail
        assert_eq!(
            store.add_code(email, login_attempt_id, code).await,
            Err(TwoFACodeStoreError::UnexpectedError)
        );
    }

    #[tokio::test]
    async fn test_remove_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(email_str.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // Remove non-existent code should fail
        assert_eq!(
            store.remove_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );

        // Add code
        store.add_code(email.clone(), login_attempt_id, code).await.unwrap();

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
        let mut store = HashmapTwoFACodeStore::default();
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(email_str.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        // Get non-existent code should fail
        assert_eq!(
            store.get_code(&email).await,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        );

        // Add code
        store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await.unwrap();

        // Get existing code should succeed
        let result = store.get_code(&email).await;
        assert!(result.is_ok());
        let (retrieved_id, retrieved_code) = result.unwrap();
        assert_eq!(retrieved_id, login_attempt_id);
        assert_eq!(retrieved_code, code);
    }
}