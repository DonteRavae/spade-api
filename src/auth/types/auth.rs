use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::JwtManager;

#[derive(Debug)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    _members: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Auth {
    pub id: Uuid,
    pub team: Option<Uuid>,
    pub email: String,
    pub hash: String,
    pub refresh_token: String,
}

impl Auth {
    pub fn new(request: UserRegistrationRequest) -> (Self, String) {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let hash = argon2
            .hash_password(request.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let mut auth = Self {
            id: uuid::Uuid::new_v4(),
            team: None,
            email: request.email,
            hash,
            refresh_token: String::new(),
        };

        auth.refresh_token = JwtManager::new_refresh_token(&auth.id.to_string()).unwrap();
        let access_token = JwtManager::new_access_token(&auth.id.to_string())
            .expect("error creating access token");

        (auth, access_token)
    }

    pub fn new_with_team(request: UserRegistrationRequest) -> (Self, String) {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let hash = argon2
            .hash_password(request.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let mut auth = Self {
            id: uuid::Uuid::new_v4(),
            team: request.team,
            email: request.email,
            hash,
            refresh_token: String::new(),
        };

        auth.refresh_token = JwtManager::new_refresh_token(&auth.refresh_token).unwrap();
        let access_token = JwtManager::new_access_token(&auth.id.to_string())
            .expect("error creating access token");

        (auth, access_token)
    }

    pub fn verify_password(password: &[u8], hash: &str) -> bool {
        Argon2::default()
            .verify_password(password, &PasswordHash::new(hash).unwrap())
            .is_ok()
    }

    pub fn validate_email(email: &str) -> bool {
        let validation_test = Regex::new(r"^(\w+@[a-zA-Z_]+?\.[a-zA-Z.]{2,6})$").unwrap();
        validation_test.is_match(email).unwrap_or(false)
    }

    pub fn validate_password(password: &str) -> bool {
        let validation_test =
            Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[!@#$%]).{8,24}$").unwrap();
        validation_test.is_match(password).unwrap_or(false)
    }
}

#[derive(Default, Deserialize)]
pub struct UserAccessRequest {
    pub email: String,
    pub password: String,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct UserRegistrationRequest {
    pub email: String,
    pub password: String,
    pub team: Option<Uuid>,
}

#[derive(Debug, Default, Serialize)]
pub struct AuthResponse {
    success: bool,
    message: Option<String>,
}

impl AuthResponse {
    pub fn new(success: bool, message: Option<String>) -> Self {
        Self { success, message }
    }
}
