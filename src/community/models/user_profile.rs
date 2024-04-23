use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, Row};

use crate::{community::CommunityError, db::DbController};

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

    pub async fn does_profile_exist(db: &DbController, id: &str) -> bool {
        sqlx::query("SELECT * FROM user_profiles WHERE id = ?")
            .bind(id)
            .fetch_one(&db.community_pool)
            .await
            .is_ok()
    }

    pub async fn register(
        db: &DbController,
        id: String,
        details: NewProfileRequest,
    ) -> Result<Self, Error> {
        if Self::does_profile_exist(db, &id).await {
            return Err(
                CommunityError::DuplicateProfile("Profile already exists.".to_string())
                    .extend_with(|_, e| e.set("code", 400)),
            );
        }

        sqlx::query(
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
        .bind(&details.username)
        .bind(&details.avatar)
        .execute(&db.community_pool)
        .await?;

        Ok(Self::new(id, details.username, details.avatar, vec![]))
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
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
            return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                e.set("reason", "User is either not logged in or does not exist")
            }));
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

    pub async fn delete(db: &DbController, id: String) -> Result<bool, Error> {
        let mut tx = db.community_pool.begin().await?;

        sqlx::query("DELETE FROM user_profiles WHERE id = ?")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM likes WHERE author = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(true)
    }
}

/********** REQUEST OBJECTS **********/

#[derive(InputObject, Debug)]
pub struct NewProfileRequest {
    pub username: String,
    pub avatar: String,
}
