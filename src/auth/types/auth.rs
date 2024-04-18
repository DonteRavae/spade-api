use async_graphql::{ErrorExtensions, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

use super::{
    email::Email,
    jwt::{AccessToken, RefreshToken},
    AuthError,
};

#[derive(Debug, FromRow)]
pub struct Auth {
    pub id: Uuid,
    pub email: Email,
    pub hash: String,
    pub community_id: Ulid,
    pub refresh_token: RefreshToken,
}

impl Auth {
    pub fn new(email: Email, hash: String) -> Result<(Self, AccessToken), async_graphql::Error> {
        let auth_id = Uuid::new_v4();
        let community_id = Ulid::new();

        let Ok(refresh_token) = RefreshToken::new(&auth_id.to_string()) else {
            return Err(
                AuthError::ServerError("Error creating refresh token".to_string())
                    .extend_with(|_, e| e.set("code", 500)),
            );
        };
        let Ok(access_token) = AccessToken::new(&community_id.to_string()) else {
            return Err(
                AuthError::ServerError("Error creating access token".to_string())
                    .extend_with(|_, e| e.set("code", 500)),
            );
        };

        let auth = Self {
            id: auth_id,
            email,
            hash,
            community_id,
            refresh_token,
        };

        Ok((auth, access_token))
    }
}

#[derive(Default, Deserialize, InputObject)]
pub struct AuthAccessRequest {
    pub email: String,
    pub password: String,
}

#[derive(Default, Deserialize, Serialize, Debug, InputObject)]
pub struct AuthRegistrationRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Default, Serialize, SimpleObject)]
pub struct AuthResponse {
    success: bool,
    message: Option<String>,
}

impl AuthResponse {
    pub fn new(success: bool, message: Option<String>) -> Self {
        Self { success, message }
    }
}
