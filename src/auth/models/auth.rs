use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use ulid::Ulid;
use uuid::Uuid;

use crate::{auth::AuthError, community::user_profile::UserProfile, db::DbController};

use super::{
    email::Email,
    jwt::{AccessToken, RefreshToken, Tokens},
    password, Password,
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

    async fn does_email_exist(db: &DbController, email: &str) -> bool {
        sqlx::query("SELECT id FROM auths WHERE email = ?")
            .bind(email)
            .fetch_one(&db.auth_pool)
            .await
            .is_ok()
    }

    pub async fn register(
        db: &DbController,
        email: Email,
        password: Password,
    ) -> Result<Tokens, Error> {
        // Check if email exists
        if Self::does_email_exist(db, email.as_str()).await {
            return Err(AuthError::DuplicateUser("User already exists".to_string())
                .extend_with(|_, e| e.set("code", 400)));
        }

        // Create User representation for database
        let (auth, access_token) = Auth::new(email, password.hash()?)?;

        if let Err(err) =
            sqlx::query("INSERT INTO auths (id, email, hash, community_id, refresh_token) VALUES (?, ?, ?, ?, ?)")
                .bind(auth.id.to_string())
                .bind(auth.email.as_str())
                .bind(auth.hash)
                .bind(auth.community_id.to_string())
                .bind(auth.refresh_token.as_str())
                .execute(&db.auth_pool)
                .await
        {
            println!("{:#?}", err);
            return Err(
                AuthError::ServerError("Server error. Please try again".to_string())
                    .extend_with(|_, e| e.set("code", 500)),
            );
        };

        Ok((access_token, auth.refresh_token))
    }

    pub async fn login(
        db: &DbController,
        email: Email,
        password: Password,
    ) -> Result<Tokens, Error> {
        if let Ok(auth) = sqlx::query("SELECT * FROM auths WHERE email = ?")
            .bind(email.as_str())
            .fetch_one(&db.auth_pool)
            .await
        {
            // Verify password sent by user
            let hash: &str = auth.get("hash");
            password.verify(hash)?;

            // Generate JWT tokens
            let auth_id: &str = auth.get("id");
            let community_id: &str = auth.get("community_id");
            let access_token = AccessToken::new(community_id)?;
            let refresh_token = RefreshToken::new(auth_id)?;

            // Update refresh token in database
            if sqlx::query("UPDATE auths SET refresh_token = ? WHERE email = ?")
                .bind(refresh_token.as_str())
                .bind(email.as_str())
                .execute(&db.auth_pool)
                .await
                .is_err()
            {
                return Err(AuthError::ServerError(
                    "There seems to be a server error. Please try again".to_string(),
                )
                .extend_with(|_, e| e.set("code", 500)));
            }

            Ok((access_token, refresh_token))
        } else {
            Err(
                AuthError::BadRequest("Please enter a valid email or password".to_string())
                    .extend_with(|_, e| e.set("code", 400)),
            )
        }
    }

    pub async fn logout(db: &DbController, id: &str) -> Result<(), Error> {
        if sqlx::query("UPDATE auths SET refresh_token = ? WHERE community_id = ?")
            .bind("")
            .bind(id)
            .execute(&db.auth_pool)
            .await
            .is_err()
        {
            return Err(
                AuthError::ServerError("Error logging out. Please try again".to_string())
                    .extend_with(|_, e| e.set("code", 500)),
            );
        }

        Ok(())
    }

    pub async fn refresh(db: &DbController, id: &str) -> Result<AccessToken, Error> {
        let Ok(auth) = sqlx::query("SELECT id FROM auths WHERE id = ?")
            .bind(id)
            .fetch_one(&db.auth_pool)
            .await
        else {
            return Err(AuthError::Forbidden.extend());
        };

        let auth_id: &str = auth.get("id");
        Ok(AccessToken::new(auth_id)?)
    }

    pub async fn update_email(
        db: &DbController,
        email: Email,
        community_id: String,
    ) -> Result<bool, Error> {
        if sqlx::query(
            r#"
            UPDATE 
                auths 
            SET email = ? 
            WHERE community_id = ?
        "#,
        )
        .bind(email.as_str())
        .bind(community_id)
        .execute(&db.auth_pool)
        .await
        .is_err()
        {
            return Err(AuthError::ServerError(
                "There was a problem updating your email. Please try again.".to_string(),
            )
            .extend());
        };

        Ok(true)
    }

    pub async fn update_password(
        db: &DbController,
        password: Password,
        community_id: String,
    ) -> Result<bool, Error> {
        if sqlx::query(
            r#"
            UPDATE 
                auths 
            SET hash = ? 
            WHERE community_id = ?
        "#,
        )
        .bind(password.hash()?)
        .bind(community_id)
        .execute(&db.auth_pool)
        .await
        .is_err()
        {
            return Err(AuthError::ServerError(
                "There was a problem updating your password. Please try again.".to_string(),
            )
            .extend());
        };

        Ok(true)
    }

    pub async fn delete(db: &DbController, community_id: String) -> Result<bool, Error> {
        let mut tx = db.auth_pool.begin().await?;

        if sqlx::query("DELETE FROM auths WHERE community_id = ?")
            .bind(&community_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            // Return error. Transaction will rollback once it's out of scope and the transaction is dropped
            return Err(AuthError::ServerError(
                "There seems to be an issue on our end. Please try again.".to_string(),
            )
            .extend());
        }

        UserProfile::delete(db, community_id).await?;

        tx.commit().await?;

        Ok(true)
    }
}

/********** REQUEST AND RESPONSE OBJECTS **********/

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
