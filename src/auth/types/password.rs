use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_graphql::{Error, ErrorExtensions};
use fancy_regex::Regex;
use rand::rngs::OsRng;

use super::AuthError;

#[derive(Debug)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Self, Error> {
        let password_validation_test =
            Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[!@#$%]).{8,24}$").unwrap();
        match password_validation_test.is_match(&password) {
            Ok(result) => {
                if result {
                    Ok(Password(password))
                } else {
                    Err(AuthError::ValidationError(
                        "Please enter a valid email or password".to_string(),
                    )
                    .extend_with(|_, e| e.set("code", 400)))
                }
            }
            Err(_) => Err(AuthError::ValidationError(
                "There seems to be an error on our end. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500))),
        }
    }

    pub fn verify(&self, hash: &str) -> Result<(), Error> {
        if Argon2::default()
            .verify_password(self.0.as_bytes(), &PasswordHash::new(hash).unwrap())
            .is_err()
        {
            return Err(AuthError::VerificationError(
                "Please enter a valid email or password.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 400)));
        }

        Ok(())
    }

    pub fn hash(&self) -> Result<String, Error> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        Ok(argon2.hash_password(self.0.as_bytes(), &salt)?.to_string())
    }
}
