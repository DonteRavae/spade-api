use crate::auth::{AccessToken, Auth, AuthError, Email, Password, RefreshToken};
use async_graphql::{Error, ErrorExtensions};
use sqlx::Row;

use super::DbController;

type Tokens = (AccessToken, RefreshToken);

impl DbController {
    async fn does_email_exist(&self, email: &str) -> bool {
        sqlx::query("SELECT id FROM auths WHERE email = ?")
            .bind(email)
            .fetch_one(&self.auth_pool)
            .await
            .is_ok()
    }

    pub async fn register(&self, email: Email, password: Password) -> Result<Tokens, Error> {
        // Check if email exists
        if self.does_email_exist(email.as_str()).await {
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
                .execute(&self.auth_pool)
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

    pub async fn login(&self, email: Email, password: Password) -> Result<Tokens, Error> {
        if let Ok(auth) = sqlx::query("SELECT * FROM auths WHERE email = ?")
            .bind(email.as_str())
            .fetch_one(&self.auth_pool)
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
                .execute(&self.auth_pool)
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
                AuthError::ValidationError("Please enter a valid email or password".to_string())
                    .extend_with(|_, e| e.set("code", 400)),
            )
        }
    }

    pub async fn logout(&self, id: &str) -> Result<(), Error> {
        if sqlx::query("UPDATE auths SET refresh_token = ? WHERE community_id = ?")
            .bind("")
            .bind(id)
            .execute(&self.auth_pool)
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

    pub async fn refresh(&self, id: &str) -> Result<AccessToken, Error> {
        let Ok(auth) = sqlx::query("SELECT id FROM auths WHERE id = ?")
            .bind(id)
            .fetch_one(&self.auth_pool)
            .await
        else {
            return Err(AuthError::Forbidden.extend());
        };

        let auth_id: &str = auth.get("id");
        Ok(AccessToken::new(auth_id)?)
    }
}
