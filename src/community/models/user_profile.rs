use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use sqlx::{types::Json, FromRow, Row};

use crate::{community::CommunityError, db::DbController};

#[derive(Debug, FromRow, SimpleObject, InputObject, Default)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub avatar: String,
    pub likes: Option<Vec<String>>,
}

impl UserProfile {
    pub fn new(id: String, username: String, avatar: String, likes: Option<Vec<String>>) -> Self {
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

        sqlx::query("INSERT INTO user_profiles(id, username, avatar) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(&details.username)
            .bind(&details.avatar)
            .execute(&db.community_pool)
            .await?;

        Ok(Self::new(id, details.username, details.avatar, None))
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
        let Ok(profile) = sqlx::query("SELECT id, username, avatar, (SELECT JSON_ARRAYAGG(likes.parent_id) FROM likes) as likes FROM user_profiles WHERE id = ? GROUP BY user_profiles.id")
            .bind(id)
            .fetch_one(&db.community_pool)
            .await else {
                return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                e.set("reason", "User is either not logged in or does not exist")
            }))};

        let likes: Json<Option<Vec<String>>> = profile.get("likes");

        let likes = if likes.0.is_some() { likes.0 } else { None };

        Ok(Self::new(
            profile.get("id"),
            profile.get("username"),
            profile.get("avatar"),
            likes,
        ))
    }
}

/********** REQUEST OBJECTS **********/

#[derive(InputObject, Debug)]
pub struct NewProfileRequest {
    pub username: String,
    pub avatar: String,
}
