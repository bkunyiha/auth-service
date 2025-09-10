use color_eyre::eyre::{Result, eyre};
use regex::Regex;
use secrecy::{ExposeSecret, SecretBox};
use std::hash::{Hash, Hasher};
use validator::ValidationError;

#[derive(Debug)]
pub struct Password(SecretBox<String>);

impl Clone for Password {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl Eq for Password {}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(s: SecretBox<String>) -> Result<Password> {
        if validate_password(&s) {
            Ok(Self(s))
        } else {
            Err(eyre!("Failed to parse string to a Password type"))
        }
    }

    pub fn to_str(&self) -> &str {
        "******"
    }
}

fn validate_password(s: &SecretBox<String>) -> bool {
    s.expose_secret().len() >= 8
}

// The AsRef trait is used to convert a reference of one type to a reference of another type. In this case,
// we're implementing the AsRef trait for the Password type to allow us to convert a &Password to a &str.
// This is useful when we want to expose the inner password string in a read-only manner.
impl AsRef<SecretBox<String>> for Password {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

fn validate_email(email: &SecretBox<String>) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(email.expose_secret()) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_email"))
    }
}

#[derive(Debug)]
pub struct Email {
    email: SecretBox<String>,
}

impl Clone for Email {
    fn clone(&self) -> Self {
        Self {
            email: SecretBox::new(Box::new(self.email.expose_secret().clone())),
        }
    }
}

impl Email {
    pub fn parse(email: SecretBox<String>) -> Result<Self> {
        let email = Email { email };
        validate_email(&email.email)?;
        Ok(email)
    }

    pub fn to_str(&self) -> &str {
        self.email.expose_secret()
    }
}

impl Eq for Email {}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.email.expose_secret() == other.email.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.email.expose_secret().hash(state);
    }
}

// The AsRef trait is used to convert a reference of one type to a reference of another type.
// In this case, we're implementing the AsRef trait for the Email type to allow us to convert a &Email to a &str.
// This is useful when we want to expose the inner email string in a read-only manner.
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.email.expose_secret().as_str()
    }
}

// The User struct should contain 3 fields. email, which is a String;
// password, which is also a String; and requires_2fa, which is a boolean.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{
        Fake,
        faker::{internet::en::Password as FakerPassword, internet::en::SafeEmail},
    };

    #[test]
    fn test_email_parse() {
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(SecretBox::new(Box::new(email_str.clone())));
        assert!(matches!(email, Ok(e) if e.email.expose_secret() == &email_str));
    }

    #[test]
    fn test_email_parse_empty() {
        let email = Email::parse(SecretBox::new(Box::new("".to_string())));
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_email_parse_invalid() {
        let email = Email::parse(SecretBox::new(Box::new("test".to_string())));
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_password_parse() {
        let password_secret: SecretBox<String> = SecretBox::new(Box::new(
            FakerPassword(std::ops::Range { start: 8, end: 30 }).fake(),
        ));
        let parsed_password = Password::parse(SecretBox::new(Box::new(
            password_secret.expose_secret().clone(),
        )));
        assert!(
            matches!(parsed_password, Ok(p) if p.0.expose_secret() == password_secret.expose_secret())
        );
    }

    #[test]
    fn test_password_parse_empty() {
        let password = Password::parse(SecretBox::new(Box::new("".to_string())));
        assert!(matches!(password, Err(_)));
    }

    #[test]
    fn test_password_parse_short() {
        let password = Password::parse(SecretBox::new(Box::new("short".to_string())));
        assert!(matches!(password, Err(_)));
    }

    #[test]
    fn test_user_new() {
        let email = Email::parse(SecretBox::new(Box::new("test@example.com".to_string()))).unwrap();
        let password_str: String = SafeEmail().fake();
        let password = Password::parse(SecretBox::new(Box::new(password_str))).unwrap();
        let user = User::new(email.clone(), password, false);
        assert_eq!(user.email, email);
    }

    #[test]
    fn test_user_new_invalid_email() {
        let email = Email::parse(SecretBox::new(Box::new("invalid-email".to_string())));
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_user_new_empty_email() {
        let email = Email::parse(SecretBox::new(Box::new("".to_string())));
        assert!(matches!(email, Err(_)));
    }

    #[derive(Debug)]
    struct ValidPasswordFixture(pub SecretBox<String>); // Updated!

    impl Clone for ValidPasswordFixture {
        fn clone(&self) -> Self {
            Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
        }
    }

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            let password = FakerPassword(8..30).fake();
            Self(SecretBox::new(Box::new(password)))
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}
