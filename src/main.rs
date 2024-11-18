use axum::*;
use body::Body;
use email_address::EmailAddress;
use http::StatusCode;
use response::Html;
use routing::{get, post};
use serde::Deserialize;
use thiserror::Error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/hello", get(|| async move { Html("<p> Hello World</p>") }))
        .route("/user/create", post(create_user));
    // `POST /users` goes to `create_user`

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_user(Json(create_user): Json<CreateUser>) -> (StatusCode) {
    StatusCode::OK
}

#[derive(Deserialize)]
struct CreateUser {
    username: UserName,
    email: Email,
}
#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
struct Email(String);

#[derive(Debug, Error)]
#[error("Error with validating Email")]
pub struct EmailError;

impl Email {
    pub fn try_new(email: String) -> Result<Self, EmailError> {
        if EmailAddress::is_valid(&email) {
            Ok(Self(email))
        } else {
            Err(EmailError)
        }
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Email {
    type Error = EmailError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Email::try_new(value)
    }
}

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
struct UserName(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserNameError {
    #[error("Username is too short; It needs a minimum length of 12 Characters")]
    TooShort,
    #[error("Username is too long; The maximum Length is 32")]
    TooLong,
    #[error("Invalid Character {0} in Username")]
    InvalidCharacter(String),
}

impl UserName {
    pub fn try_new(username: String) -> Result<Self, UserNameError> {
        let INVALID_CHARS = "!§$%&/()=?";
        if username.len() < 12 {
            Err(UserNameError::TooShort)
        } else if username.len() >= 32 {
            Err(UserNameError::TooLong)
        } else {
            for char in username.chars() {
                if INVALID_CHARS.contains(char) {
                    return Err(UserNameError::InvalidCharacter(char.to_string()));
                }
            }
            Ok(Self(username))
        }
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for UserName {
    type Error = UserNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        UserName::try_new(value)
    }
}

#[cfg(test)]
mod test_email {
    use super::*;

    #[test]
    fn test_good() {
        assert!(Email::try_new("example@s.example".to_string()).is_ok());
        assert!(Email::try_new("admin@mailserver1".to_string()).is_ok());
        assert!(Email::try_new("example-indeed@strange-example.com".to_string()).is_ok());
    }

    #[test]
    fn test_bad() {
        assert!(Email::try_new("this\\ still\"not\\allowed@example.com".to_string()).is_err());
        assert!(Email::try_new("a\"b(c)d,e:f;g<h>i[j\\k]l@example.com".to_string()).is_err());
        assert!(Email::try_new("A@b@c@example.com".to_string()).is_err());
        assert!(Email::try_new("Abc.example.com".to_string()).is_err());
        assert!(Email::try_new(
            "1234567890123456789012345678901234567890123456789012345678901234+x@example.co"
                .to_string()
        )
        .is_err());
    }
}

#[cfg(test)]
mod test_username {
    use super::*;

    #[test]
    fn test_good() {
        assert!(UserName::try_new("HelloWorldIAmTim".to_string()).is_ok());
        assert!(UserName::try_new("HelloWorld123141IAmTim".to_string()).is_ok());
        assert!(UserName::try_new("HelloWorld.....IAmTim".to_string()).is_ok());
    }

    #[test]
    fn test_bad() {
        assert!(UserName::try_new("test".to_string()).is_err());
        assert!(UserName::try_new("?testhallowkfahfla".to_string()).is_err());
        assert!(
            UserName::try_new("halloweltichbindertimundichhasselangeusernames".to_string())
                .is_err()
        );
        assert!(UserName::try_new("test!%$/".to_string()).is_err());
    }
    #[test]
    fn test_correct_errors() {
        assert_eq!(
            UserName::try_new("test".to_string()).unwrap_err(),
            UserNameError::TooShort
        );

        assert_eq!(
            UserName::try_new("?testhallowkfahfla".to_string()).unwrap_err(),
            UserNameError::InvalidCharacter('?'.to_string())
        );

        assert_eq!(
            UserName::try_new("halloweltichbindertimundichhasselangeusernames".to_string())
                .unwrap_err(),
            UserNameError::TooLong
        );

        assert_eq!(
            UserName::try_new("test!%$/".to_string()).unwrap_err(),
            UserNameError::TooShort
        );
    }
}
