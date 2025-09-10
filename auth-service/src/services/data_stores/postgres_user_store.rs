use std::error::Error;

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};

use sqlx::PgPool;

use crate::domain::{Email, Password, User};
use crate::services::data_stores::{UserStore, UserStoreError};
use argon2::password_hash::rand_core::OsRng;
use color_eyre::eyre::{Context, Result, eyre};
use secrecy::{ExposeSecret, SecretBox};

pub struct DBUser {
    pub email: String,
    pub password_hash: String,
    pub requires_2fa: bool,
}
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Check if user already exists
        let _: Result<_, UserStoreError> = match self.get_user(&user.email).await {
            Ok(_) => return Err(UserStoreError::UserAlreadyExists),
            Err(UserStoreError::UserNotFound) => Ok(()),
            Err(other_error) => return Err(other_error),
        };

        // Hash the password before storing
        let password_hash = compute_password_hash(SecretBox::new(Box::new(
            user.password.as_ref().expose_secret().to_owned(),
        )))
        .await
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        // Insert the new user
        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref(),
            password_hash,
            user.requires_2fa,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
        //.map_err(UserStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let user_maybe = sqlx::query_as!(
            DBUser,
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        match user_maybe {
            Some(db_user) => {
                let user = User::new(
                    email.clone(),
                    Password::parse(SecretBox::new(Box::new(db_user.password_hash)))
                        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                    db_user.requires_2fa,
                );
                Ok(user)
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = sqlx::query!(
            "SELECT password_hash FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        match user {
            Some(user) => {
                verify_password_hash(
                    user.password_hash,
                    password.as_ref().expose_secret().to_owned(),
                )
                .await
                .map_err(|_| UserStoreError::InvalidCredentials)?;
                Ok(())
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

// Helper function to verify if a given password matches an expected hash
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, this function to performs hashing on a
// separate thread pool using tokio::task::spawn_blocking.
// #[tracing::instrument(name = "Verify password hash", skip_all)]
// async fn verify_password_hash_other(
//     expected_password_hash: String,
//     password_candidate: String,
// ) -> Result<(), Box<dyn Error + Send + Sync>> {
//     let current_span: tracing::Span = tracing::Span::current();
//     tokio::spawn(async move {
//         current_span.in_scope(|| {
//             let expected_password_hash: PasswordHash<'_> =
//                 PasswordHash::new(&expected_password_hash)?;

//             Argon2::default()
//                 .verify_password(password_candidate.as_bytes(), &expected_password_hash)
//                 .map_err(|e| e.into())
//         })
//     })
//     .await?
// }

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    // This line retrieves the current span from the tracing context.
    // The span represents the execution context for the compute_password_hash function.
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        // This code block ensures that the operations within the closure are executed within the context of the current span.
        // This is especially useful for tracing operations that are performed in a different thread or task, such as within tokio::task::spawn_blocking.
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify password hash")
            //.map_err(|e| e.into())
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(
    password: SecretBox<String>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let current_span: tracing::Span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await?
}
