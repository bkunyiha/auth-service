use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate)]
pub struct Email{
    #[validate(email)]
    email: String
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Password{
    #[validate(length(min = 8))]
    password: String
}

impl Email {
    pub fn parse(email: String) -> Result<Self, String>  {
       let email = Email {email};
       email.validate().map_err(|_| "Invalid email format".to_string())?;

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

impl Password {
    pub fn parse(password: String) -> Result<Self, String> {
        let password = Password{password};
        password.validate().map_err(|_| "Invalid password format".to_string())?;

        Ok(password)
    }

    pub fn to_str(&self) -> &str {
    "******"
    }   
}

// The AsRef trait is used to convert a reference of one type to a reference of another type. In this case, 
// we're implementing the AsRef trait for the Password type to allow us to convert a &Password to a &str. 
// This is useful when we want to expose the inner password string in a read-only manner.
impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
    &self.password
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
        Self { email, password, requires_2fa }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::{internet::en::SafeEmail, internet::en::Password as FakerPassword}, Fake};

    #[test]
    fn test_email_parse() {
        let email_str: String = SafeEmail().fake();
        let email = Email::parse(email_str.clone());
        assert_eq!(email, Ok(Email{email: email_str}));
    }

    #[test]
    fn test_email_parse_empty() {
        let email = Email::parse("".to_string());
        assert_eq!(email, Err("Invalid email format".to_string()));
    }

    #[test]
    fn test_email_parse_invalid() {
        let email = Email::parse("test".to_string());
        assert_eq!(email, Err("Invalid email format".to_string()));
    }
    
    #[test]
    fn test_password_parse() {
        let password_str: String = FakerPassword(std::ops::Range {start: 8, end: 30}).fake();
        let password = Password::parse(password_str.clone());
        assert_eq!(password, Ok(Password{ password:password_str}));
    }

    #[test]
    fn test_password_parse_empty() {
        let password = Password::parse("".to_string());
        assert_eq!(password, Err("Invalid password format".to_string()));
    }

    #[test]
    fn test_password_parse_short() {
        let password = Password::parse("short".to_string());
        assert_eq!(password, Err("Invalid password format".to_string()));
    }

    #[test]
    fn test_user_new() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password_str: String = SafeEmail().fake();

        let password = Password::parse(password_str).unwrap();
        let user = User::new(email.clone(), password, false);
        assert_eq!(user.email, email);  
    }

    #[test]
    fn test_user_new_invalid_email() {
        let email = Email::parse("invalid-email".to_string());
        assert_eq!(email, Err("Invalid email format".to_string()));
    }   

    #[test]
    fn test_user_new_invalid_password() {
        let password = Password::parse("short".to_string());
        assert_eq!(password, Err("Invalid password format".to_string()));
    }
 
    #[test]
    fn test_user_new_empty_email() {
        let email = Email::parse("".to_string());
        assert_eq!(email, Err("Invalid email format".to_string()));
    }
 
    #[test]
    fn test_user_new_empty_password() {
        let password = Password::parse("".to_string());
        assert_eq!(password, Err("Invalid password format".to_string()));
    }
}
