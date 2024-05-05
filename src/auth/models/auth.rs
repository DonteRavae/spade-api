use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use ulid::Ulid;
use uuid::Uuid;

use crate::{community::UserProfile, db::DbController};

use super::{
    email::Email,
    jwt::{AccessToken, RefreshToken, Tokens},
    Password,
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
    pub fn new(email: Email, hash: String) -> Result<(Self, AccessToken), String> {
        let auth_id = Uuid::new_v4();
        let community_id = Ulid::new();

        let Ok(refresh_token) = RefreshToken::new(&auth_id.to_string()) else {
            return Err("Error creating refresh token".to_string());
        };
        let Ok(access_token) = AccessToken::new(&community_id.to_string()) else {
            return Err("Error creating access token".to_string());
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
        username: String,
    ) -> Result<Tokens, String> {
        // Check if email exists
        if Self::does_email_exist(db, email.as_str()).await {
            return Err("User already exists".to_string());
        }

        let Ok(mut tx) = db.auth_pool.begin().await else {
            eprintln!("DATABASE_ERROR: Error starting transaction.");
            return Err("Server error. Please try again".to_string());
        };

        // Create User representation for database
        let (auth, access_token) = Auth::new(email, password.hash()?)?;

        if let Err(err) =
            sqlx::query("INSERT INTO auths (id, email, hash, community_id, refresh_token) VALUES (?, ?, ?, ?, ?)")
                .bind(auth.id.to_string())
                .bind(auth.email.as_str())
                .bind(auth.hash)
                .bind(auth.community_id.to_string())
                .bind(auth.refresh_token.as_str())
                .execute(&mut *tx)
                .await
        {
            eprintln!("{:#?}", err);
            return Err("Server error. Please try again".to_string())
        };

        if UserProfile::register(db, auth.community_id.to_string(), username)
            .await
            .is_err()
        {
            return Err(
                "There was an issue creating the user profile. Please try again.".to_string(),
            );
        }

        let _ = tx.commit().await;
        Ok((access_token, auth.refresh_token))
    }

    pub async fn login(
        db: &DbController,
        email: Email,
        password: Password,
    ) -> Result<Tokens, String> {
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
            let Ok(access_token) = AccessToken::new(community_id) else {
                eprintln!("JWT ERROR: Error creating access token in Login");
                return Err("Server error. Please try again".to_string());
            };
            let Ok(refresh_token) = RefreshToken::new(auth_id) else {
                eprintln!("JWT ERROR: Error creating refresh token in Login");
                return Err("Server error. Please try again".to_string());
            };

            // Update refresh token in database
            if sqlx::query("UPDATE auths SET refresh_token = ? WHERE email = ?")
                .bind(refresh_token.as_str())
                .bind(email.as_str())
                .execute(&db.auth_pool)
                .await
                .is_err()
            {
                return Err("There seems to be a server error. Please try again".to_string());
            }

            Ok((access_token, refresh_token))
        } else {
            Err("Please enter a valid email or password".to_string())
        }
    }

    pub async fn logout(db: &DbController, id: &str) -> Result<(), String> {
        if sqlx::query("UPDATE auths SET refresh_token = ? WHERE community_id = ?")
            .bind("")
            .bind(id)
            .execute(&db.auth_pool)
            .await
            .is_err()
        {
            return Err("Error logging out. Please try again".to_string());
        }

        Ok(())
    }

    pub async fn refresh(db: &DbController, id: &str) -> Result<AccessToken, String> {
        let Ok(auth) = sqlx::query("SELECT id FROM auths WHERE id = ?")
            .bind(id)
            .fetch_one(&db.auth_pool)
            .await
        else {
            return Err("User does not exist.".to_string());
        };

        let auth_id: &str = auth.get("id");
        let Ok(token) = AccessToken::new(auth_id) else {
            eprintln!("JWT ERROR: Error creating access token in Refresh");
            return Err("Server error. Please try again".to_string());
        };

        Ok(token)
    }

    pub async fn update_email(
        db: &DbController,
        email: Email,
        community_id: String,
    ) -> Result<bool, String> {
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
            return Err("There was a problem updating your email. Please try again.".to_string());
        };

        Ok(true)
    }

    pub async fn update_password(
        db: &DbController,
        password: Password,
        community_id: String,
    ) -> Result<bool, String> {
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
            return Err(
                "There was a problem updating your password. Please try again.".to_string(),
            );
        };

        Ok(true)
    }

    pub async fn delete(db: &DbController, community_id: String) -> Result<bool, String> {
        let Ok(mut tx) = db.auth_pool.begin().await else {
            eprintln!("DATABASE_ERROR: Error starting transaction.");
            return Err("Server error. Please try again".to_string());
        };

        if sqlx::query("DELETE FROM auths WHERE community_id = ?")
            .bind(&community_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return Err("There seems to be an issue on our end. Please try again.".to_string());
        }

        if UserProfile::delete(db, community_id).await.is_err() {
            return Err(
                "DATABASE ERROR: Error deleting user in database. Please try again.".to_string(),
            );
        }

        let _ = tx.commit().await;

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
    pub username: String,
}
