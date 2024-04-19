use crate::{community::CommunityError, db::DbController};
use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use sqlx::{FromRow, Row};
use ulid::Ulid;

use super::{
    reply::{NewReplyRequest, Reply},
    user_profile::UserProfile,
};

#[derive(Debug, FromRow, SimpleObject, InputObject)]
#[graphql(input_name = "NewExpressionPostContent")]
struct ExpressionPostContent {
    kind: String,
    value: String,
}

#[derive(Debug, FromRow, SimpleObject)]
pub struct ExpressionPost {
    id: String,
    title: String,
    subtitle: Option<String>,
    author: UserProfile,
    content: ExpressionPostContent,
}

impl ExpressionPost {
    pub fn new(
        id: String,
        title: String,
        subtitle: String,
        author: UserProfile,
        content_type: String,
        content_value: String,
    ) -> Self {
        Self {
            id,
            title,
            subtitle: if subtitle.is_empty() {
                None
            } else {
                Some(subtitle)
            },
            author,
            content: ExpressionPostContent {
                kind: content_type,
                value: content_value,
            },
        }
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
        let post = sqlx::query(r#"
            SELECT
                id, 
                title, 
                subtitle, 
                JSON_OBJECT('id', profile.id, 'username', profile.username, 'avatar', profile.avatar) AS author, 
                content_type, 
                content_value 
            FROM expression_posts 
            JOIN user_profiles 
                AS profile 
                ON expressions_post.author = profile.id 
            WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(&db.community_pool)
        .await?;

        Ok(Self::new(
            post.get("id"),
            post.get("title"),
            post.get("subtitle"),
            UserProfile::new(
                post.get("author.id"),
                post.get("author.username"),
                post.get("author.avatar"),
                None,
            ),
            post.get("content_type"),
            post.get("content_value"),
        ))
    }

    pub async fn save(
        db: &DbController,
        post: NewExpressionPost,
        author: String,
    ) -> Result<Self, Error> {
        let Ok(profile) = sqlx::query("SELECT * FROM user_profiles WHERE id = ?")
            .bind(&author)
            .fetch_one(&db.community_pool)
            .await
        else {
            return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                e.set("reason", "User is either not logged in or does not exist")
            }));
        };

        let post_id = Ulid::new().to_string();
        sqlx::query("INSERT INTO expression_posts(id, title, subtitle, author, content_type, content_value) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&post_id)
            .bind(&post.title)
            .bind(&post.subtitle)
            .bind(author)
            .bind(&post.content.kind)
            .bind(&post.content.value)
            .execute(&db.community_pool)
            .await?;

        Ok(ExpressionPost {
            id: post_id,
            title: post.title,
            subtitle: post.subtitle,
            author: UserProfile::new(
                profile.get("id"),
                profile.get("username"),
                profile.get("avatar"),
                None,
            ),
            content: ExpressionPostContent {
                kind: post.content.kind,
                value: post.content.value,
            },
        })
    }

    pub async fn update_likes(
        db: &DbController,
        update_request: UpdateLikesRequest,
        user_id: String,
    ) -> Result<(), Error> {
        let statement = if update_request.update_value == 1 {
            r#"
                INSERT INTO likes(parent_id, author) VALUES (?, ?)
            "#
        } else {
            r#"
                DELETE FROM likes WHERE parent_id = ? AND author = ?
            "#
        };

        sqlx::query(statement)
            .bind(update_request.post_id)
            .bind(user_id)
            .execute(&db.community_pool)
            .await?;
        Ok(())
    }

    pub async fn add_reply(
        db: &DbController,
        author: String,
        request: NewReplyRequest,
    ) -> Result<Reply, Error> {
        let reply_id = Ulid::new().to_string();
        if sqlx::query("INSERT INTO replies(id, author, parent, content) VALUES(?, ?, ?, ?)")
            .bind(&reply_id)
            .bind(&author)
            .bind(&request.parent)
            .bind(&request.content)
            .execute(&db.community_pool)
            .await
            .is_err()
        {
            return Err(CommunityError::ServerError(
                "Seems there was an error adding your request. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        }

        let user_profile = UserProfile::get_by_id(db, author).await?;

        let row = sqlx::query("SELECT created_at, last_modified FROM replies WHERE id = ?")
            .bind(&reply_id)
            .fetch_one(&db.community_pool)
            .await?;

        Ok(Reply::new(
            reply_id,
            user_profile,
            request.parent,
            request.content,
            row.get("created_at"),
            row.get("last_modified"),
        ))
    }
}

/********** REQUEST OBJECTS **********/

#[derive(InputObject, Debug)]
pub struct NewExpressionPost {
    title: String,
    subtitle: Option<String>,
    content: ExpressionPostContent,
}

#[derive(InputObject, Debug)]
pub struct UpdateLikesRequest {
    pub post_id: String,
    pub update_value: u8,
}
