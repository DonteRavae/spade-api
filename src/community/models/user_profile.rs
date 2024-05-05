use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, Row};

use crate::db::DbController;

#[derive(Debug, FromRow, SimpleObject, InputObject, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub avatar: String,
    pub likes: Vec<String>,
}

impl UserProfile {
    pub fn new(id: String, username: String, avatar: String, likes: Vec<String>) -> Self {
        Self {
            id,
            username,
            avatar,
            likes,
        }
    }

    pub async fn does_profile_exist(db: &DbController, id: &str, username: &str) -> bool {
        sqlx::query("SELECT * FROM user_profiles WHERE id = ? OR username = ?")
            .bind(id)
            .bind(username)
            .fetch_one(&db.community_pool)
            .await
            .is_ok()
    }

    pub async fn register(db: &DbController, id: String, username: String) -> Result<Self, String> {
        if Self::does_profile_exist(db, &id, &username).await {
            return Err("Profile already exists.".to_string());
        }

        let Ok(mut tx) = db.community_pool.begin().await else {
            eprintln!("DATABASE_ERROR: Error starting transaction in UserProfile Register.");
            return Err("Server error. Please try again".to_string());
        };

        let avatar = format!("https://api.multiavatar.com/${id}.svg");

        if let Err(_) = sqlx::query(
            r#"
            INSERT INTO user_profiles
                (
                    id, 
                    username, 
                    avatar
                ) 
                VALUES (?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&username)
        .bind(&avatar)
        .execute(&mut *tx)
        .await
        {
            eprintln!("DATABASE_ERROR: Error inserting profile in UserProfile Register.");
            return Err("Server error. Please try again".to_string());
        };

        let _ = tx.commit().await;
        Ok(Self::new(id, username, avatar, vec![]))
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, String> {
        let Ok(profile) = sqlx::query(
            r#"
            SELECT 
                id, 
                username, 
                avatar,
                (
                    SELECT JSON_ARRAYAGG(parent_id) FROM likes WHERE author = id
                ) AS likes
            FROM user_profiles 
            WHERE id = ? 
        "#,
        )
        .bind(id)
        .fetch_one(&db.community_pool)
        .await
        else {
            return Err("User is either not logged in or does not exist".to_string());
        };

        let likes: Option<Json<Vec<String>>> = profile.get("likes");

        Ok(Self::new(
            profile.get("id"),
            profile.get("username"),
            profile.get("avatar"),
            if likes.is_some() {
                likes.unwrap().0
            } else {
                vec![]
            },
        ))
    }

    pub async fn delete(db: &DbController, id: String) -> Result<bool, String> {

        let Ok(mut tx) = db.community_pool.begin().await else {
            eprintln!("DATABASE_ERROR: Error starting transaction in UserProfile Delete.");
            return Err("Server error. Please try again".to_string());
        };

        if sqlx::query("DELETE FROM user_profiles WHERE id = ?")
            .bind(&id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            eprintln!("DATABASE_ERROR: Error deleting profile in UserProfile Delete.");
            return Err("Server error. Please try again.".to_string());
        };

        if sqlx::query("DELETE FROM likes WHERE author = ?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            eprintln!("DATABASE_ERROR: Error deleting likes in UserProfile Delete.");
            return Err("Server error. Please try again.".to_string());
        };

        let _ = tx.commit().await;
        Ok(true)
    }
}
