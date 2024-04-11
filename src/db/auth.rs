use crate::auth::types::{Auth, JwtManager, UserAccessRequest, UserRegistrationRequest};
use sqlx::Row;

use super::DbController;

type Tokens = (String, String);

impl DbController {
    async fn does_email_exist(&self, email: &str) -> bool {
        return sqlx::query("SELECT id FROM auths WHERE email = ?")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .is_ok();
    }

    pub async fn register(&self, request: UserRegistrationRequest) -> Result<Tokens, String> {
        // Check if email exists
        if self.does_email_exist(&request.email).await {
            return Err("User already exists".to_string());
        }

        // Create User representation for database
        let (auth, access_token) = if request.team.is_none() {
            Auth::new(request)
        } else {
            Auth::new_with_team(request)
        };

        if auth.team.is_some() {
            if let Err(err) = sqlx::query(
                "INSERT INTO auths (id, team, email, hash, refresh_token) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(auth.id.to_string())
            .bind(auth.team.unwrap().to_string())
            .bind(auth.email)
            .bind(auth.hash)
            .bind(auth.refresh_token.clone())
            .execute(&self.pool)
            .await
            {
                println!("{:#?}", err);
                return Err("Server error. Please try again".to_string());
            }
        } else {
            if let Err(err) = sqlx::query(
                "INSERT INTO auths (id, email, hash, refresh_token) VALUES (?, ?, ?, ?)",
            )
            .bind(auth.id.to_string())
            .bind(auth.email)
            .bind(auth.hash)
            .bind(auth.refresh_token.clone())
            .execute(&self.pool)
            .await
            {
                println!("{:#?}", err);
                return Err("Server error. Please try again".to_string());
            };
        }
        Ok((access_token, auth.refresh_token))
    }

    pub async fn login(&self, request: UserAccessRequest) -> Result<Tokens, String> {
        if let Ok(user) = sqlx::query("SELECT * FROM auths WHERE email = ?")
            .bind(&request.email)
            .fetch_one(&self.pool)
            .await
        {
            let user_id: &str = user.get("id");
            let hash: &str = user.get("hash");

            // If it gets to this point, there's guaranteed to be a hash value returned from the user row.
            let is_auth_valid = Auth::verify_password(request.password.as_bytes(), hash);

            if !is_auth_valid {
                return Err("Please enter a valid email or password".to_string());
            }

            let access_token = JwtManager::new_access_token(user_id).unwrap();
            let refresh_token = JwtManager::new_refresh_token(user_id).unwrap();

            if sqlx::query("UPDATE auths SET refresh_token = ? WHERE email = ?")
                .bind(&refresh_token)
                .bind(&request.email)
                .execute(&self.pool)
                .await
                .is_err()
            {
                return Err("There seems to be a server error. Please try again".to_string());
            }

            return Ok((access_token, refresh_token));
        } else {
            return Err("Please enter a valid email or password".to_string());
        }
    }

    pub async fn logout(&self, id: &str) -> Result<(), String> {
        if sqlx::query("UPDATE auths SET refresh_token = ? WHERE id = ?")
            .bind("")
            .bind(id)
            .execute(&self.pool)
            .await
            .is_err()
        {
            return Err("Error logging out. Please try again".to_string());
        }

        Ok(())
    }

    pub async fn refresh(&self, id: &str) -> Result<String, String> {
        let Ok(user) = sqlx::query("SELECT id FROM auths WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
        else {
            return Err("Forbidden".to_string());
        };

        let user_id: &str = user.get("id");

        let Ok(access_token) = JwtManager::new_access_token(user_id) else {
            return Err("Server Error. Please try again.".to_string());
        };

        Ok(access_token)
    }
}
