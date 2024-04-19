use async_graphql::{Error, ErrorExtensions};
use fancy_regex::Regex;

use crate::auth::AuthError;

#[derive(Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, Error> {
        let email_validation_test = Regex::new(r"^[\w\-\.]+@([\w-]+\.)+[\w-]{2,}$").unwrap();
        match email_validation_test.is_match(&email) {
            Ok(result) => {
                if result {
                    Ok(Email(email))
                } else {
                    Err(
                        AuthError::BadRequest("Please enter a valid email or password".to_string())
                            .extend_with(|_, e| e.set("code", 400)),
                    )
                }
            }
            Err(_) => Err(AuthError::ServerError(
                "We seem to be having an error on our end. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500))),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
