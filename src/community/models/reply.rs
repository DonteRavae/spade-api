use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, Row};

use crate::{community::CommunityError, db::DbController};

use super::user_profile::UserProfile;

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
pub struct Reply {
    id: String,
    author: UserProfile,
    parent: String, // Identification of parent object
    content: String,
    created_at: DateTime<Utc>,
    last_modified: DateTime<Utc>,
}

impl Reply {
    pub fn new(
        id: String,
        author: UserProfile,
        parent: String,
        content: String,
        created_at: DateTime<Utc>,
        last_modified: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            author,
            parent,
            content,
            created_at,
            last_modified,
        }
    }

    /* This method needs to be changed to a recursive search through the database */
    pub async fn get_all_recursively(
        db: &DbController,
        parent_id: String,
    ) -> Result<Vec<Self>, Error> {
        sqlx::query(
            r#"
            SELECT
                reply.id AS id,
                profile.id AS author_id,
                profile.username AS author_username,
                profile.avatar AS author_avatar,
                reply.content AS content,
                reply.created_at AS created_at,
                reply.last_modified AS last_modified
            FROM replies AS reply
            JOIN user_profiles AS profile ON profile.id = reply.author
            WHERE parent = ?
        "#,
        )
        .bind(&parent_id)
        .map(|reply: MySqlRow| {
            Reply::new(
                reply.get("id"),
                UserProfile::new(
                    reply.get("author_id"),
                    reply.get("author_username"),
                    reply.get("author_avatar"),
                    vec![],
                ),
                parent_id.clone(),
                reply.get("content"),
                reply.get("created_at"),
                reply.get("last_modified"),
            )
        })
        .fetch_all(&db.community_pool)
        .await?;
        todo!()
    }

    pub async fn delete(
        db: &DbController,
        reply_id: String,
        logged_in_user: String,
    ) -> Result<bool, Error> {
        let author = sqlx::query("SELECT author FROM replies WHERE id = ?")
            .bind(&reply_id)
            .fetch_optional(&db.community_pool)
            .await?;

        // Check to make sure person deleting post is author
        if author.is_none() {
            return Err(
                CommunityError::BadRequest("Reply does not exist.".to_string()).extend(),
            );
        } else {
            let author: String = author.unwrap().get("author");
            if author != logged_in_user {
                return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                    e.set(
                        "reason",
                        "User making request and reply author do not match.",
                    )
                }));
            }
        }
        sqlx::query("DELETE FROM replies WHERE id = ?")
            .bind(reply_id)
            .execute(&db.community_pool)
            .await?;
        Ok(true)
    }

    pub async fn delete_all_from_post(db: &DbController, post_id: String) -> Result<(), Error> {
        let mut tx = db.community_pool.begin().await?;
        sqlx::query("DELETE FROM replies WHERE parent = ?")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

/********** REQUEST OBJECT **********/

/****** ADD VALIDATION CHECKS ******/
#[derive(Debug, InputObject)]
pub struct NewReplyRequest {
    pub content: String,
    pub parent: String,
}
