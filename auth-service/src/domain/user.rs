use color_eyre::eyre::{eyre, Context, Result};
use secrecy::{ExposeSecret, Secret};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate)]
pub struct Email {
    #[validate(email)]
    email: String,
}

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Eq for Password {}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(s: Secret<String>) -> Result<Password> {
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

fn validate_password(s: &Secret<String>) -> bool {
    s.expose_secret().len() >= 8
}

// The AsRef trait is used to convert a reference of one type to a reference of another type. In this case,
// we're implementing the AsRef trait for the Password type to allow us to convert a &Password to a &str.
// This is useful when we want to expose the inner password string in a read-only manner.
impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl Email {
    pub fn parse(email: String) -> Result<Self> {
        let email = Email { email };
        email
            .validate()
            .wrap_err(format!("Invalid email format: {}", email.email))?;
        //.map_err(|_| eyre!("Invalid email format: {}", email.email))?;
        //.map_err(|_| "Invalid email format".to_string())?;

        Ok(email)
    }

    pub fn to_str(&self) -> &str {
        &self.email
    }
}

// The AsRef trait is used to convert a reference of one type to a reference of another type.
// In this case, we're implementing the AsRef trait for the Email type to allow us to convert a &Email to a &str.
// This is useful when we want to expose the inner email string in a read-only manner.
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.to_str()
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
        faker::{internet::en::Password as FakerPassword, internet::en::SafeEmail},
        Fake,
    };

    #[test]
    fn test_email_parse() {
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(email_str.clone());
        assert!(matches!(email, Ok(e) if e.email == email_str));
    }

    #[test]
    fn test_email_parse_empty() {
        let email = Email::parse("".to_string());
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_email_parse_invalid() {
        let email = Email::parse("test".to_string());
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_password_parse() {
        let password_secret: Secret<String> =
            Secret::new(FakerPassword(std::ops::Range { start: 8, end: 30 }).fake());
        let parsed_password = Password::parse(password_secret.clone());
        assert!(
            matches!(parsed_password, Ok(p) if p.0.expose_secret() == password_secret.expose_secret())
        );
    }

    #[test]
    fn test_password_parse_empty() {
        let password = Password::parse(Secret::new("".to_string()));
        assert!(matches!(password, Err(_)));
    }

    #[test]
    fn test_password_parse_short() {
        let password = Password::parse(Secret::new("short".to_string()));
        assert!(matches!(password, Err(_)));
    }

    #[test]
    fn test_user_new() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password_str: String = SafeEmail().fake();
        let password = Password::parse(Secret::new(password_str)).unwrap();
        let user = User::new(email.clone(), password, false);
        assert_eq!(user.email, email);
    }

    #[test]
    fn test_user_new_invalid_email() {
        let email = Email::parse("invalid-email".to_string());
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_user_new_invalid_password() {
        let password = Password::parse(Secret::new("short".to_string()));
        assert!(matches!(password, Err(_)));
    }

    #[test]
    fn test_user_new_empty_email() {
        let email = Email::parse("".to_string());
        assert!(matches!(email, Err(_)));
    }

    #[test]
    fn test_user_new_empty_password() {
        let password = Password::parse(Secret::new("".to_string()));
        assert!(matches!(password, Err(_)));
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>); // Updated!

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            let password = FakerPassword(8..30).fake();
            Self(Secret::new(password))
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}
