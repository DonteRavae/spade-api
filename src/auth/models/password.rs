use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use fancy_regex::Regex;
use rand::rngs::OsRng;

#[derive(Debug)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Self, String> {
        let password_validation_test =
            Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[!@#$%]).{8,24}$").unwrap();
        match password_validation_test.is_match(&password) {
            Ok(result) => {
                if result {
                    Ok(Password(password))
                } else {
                    Err("Please enter a valid email or password".to_string())
                }
            }
            Err(_) => Err("There seems to be an error on our end. Please try again.".to_string()),
        }
    }

    pub fn verify(&self, hash: &str) -> Result<(), String> {
        if Argon2::default()
            .verify_password(self.0.as_bytes(), &PasswordHash::new(hash).unwrap())
            .is_err()
        {
            return Err("Please enter a valid email or password.".to_string());
        }

        Ok(())
    }

    pub fn hash(&self) -> Result<String, String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let Ok(hashed_password) = argon2.hash_password(self.0.as_bytes(), &salt) else {
            eprintln!("PASSWORD_ERROR: There was an error hashing password.");
            return Err("Server error. Please try again.".to_string());
        };

        Ok(hashed_password.to_string())
    }
}
