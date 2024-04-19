use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use sqlx::{FromRow, Row};

use crate::{community::CommunityError, db::DbController};

#[derive(Debug, FromRow, SimpleObject, InputObject)]
pub struct UserProfile {
    id: String,
    username: String,
    avatar: String,
}

impl UserProfile {
    pub fn new(id: String, username: String, avatar: String) -> Self {
        Self {
            id,
            username,
            avatar,
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

        sqlx::query("INSERT INTO user_profiles(id, username, avatar) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(&details.username)
            .bind(&details.avatar)
            .execute(&db.community_pool)
            .await?;

        Ok(Self::new(id, details.username, details.avatar))
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
        let profile = sqlx::query("SELECT * FROM user_profiles WHERE id = ?")
            .bind(id)
            .fetch_one(&db.community_pool)
            .await?;

        Ok(Self::new(
            profile.get("id"),
            profile.get("username"),
            profile.get("avatar"),
        ))
    }
}

/********** REQUEST OBJECTS **********/

#[derive(InputObject, Debug)]
pub struct NewProfileRequest {
    pub username: String,
    pub avatar: String,
}
