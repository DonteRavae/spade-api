use crate::{community::CommunityError, db::DbController};
use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, Row};
use ulid::Ulid;

use super::{
    reply::{NewReplyRequest, Reply},
    user_profile::UserProfile,
};

#[derive(Debug, FromRow, SimpleObject, InputObject, Deserialize, Serialize)]
#[graphql(input_name = "NewExpressionPostContent")]
struct ExpressionPostContent {
    kind: String,
    value: String,
}

#[derive(Debug, FromRow, SimpleObject, Serialize, Deserialize)]
pub struct ExpressionPost {
    id: String,
    title: String,
    subtitle: Option<String>,
    author: UserProfile,
    content: ExpressionPostContent,
    replies: Vec<Reply>,
    likes: i32,
    created_at: DateTime<Utc>,
    last_modified: DateTime<Utc>,
}

impl ExpressionPost {
    pub fn new(
        id: String,
        title: String,
        subtitle: String,
        author: UserProfile,
        content_type: String,
        content_value: String,
        replies: Vec<Reply>,
        likes: i32,
        created_at: DateTime<Utc>,
        last_modified: DateTime<Utc>,
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
            replies,
            likes,
            created_at,
            last_modified,
        }
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
        let post = sqlx::query(
            r#"
            SELECT
                post.id AS id, 
                post.title AS title, 
                post.subtitle AS subtitle, 
                profile.id AS author_id, 
                profile.username AS author_username, 
                profile.avatar AS author_avatar, 
                post.content_type AS content_type, 
                post.content_value AS content_value, 
                post.created_at AS created_at, 
                post.last_modified AS last_modified,
                (
                    SELECT IFNULL(COUNT(parent_id), 0) FROM likes WHERE parent = id
                ) AS likes
            FROM expression_posts AS post
            JOIN user_profiles AS profile ON profile.id = post.author
            WHERE post.id = ?
        "#,
        )
        .bind(id)
        .fetch_one(&db.community_pool)
        .await?;

        let post_id: String = post.get("id");

        let replies: Vec<Reply> = sqlx::query(
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
        .bind(&post_id)
        .fetch_all(&db.community_pool)
        .await?
        .iter()
        .map(|reply| {
            Reply::new(
                reply.get("id"),
                UserProfile::new(
                    reply.get("author_id"),
                    reply.get("author_username"),
                    reply.get("author_avatar"),
                    vec![],
                ),
                post_id.clone(),
                reply.get("content"),
                reply.get("created_at"),
                reply.get("last_modified"),
            )
        })
        .collect();

        Ok(Self::new(
            post_id,
            post.get("title"),
            post.try_get("subtitle").unwrap_or_else(|_| String::new()),
            UserProfile::new(
                post.get("author_id"),
                post.get("author_username"),
                post.get("author_avatar"),
                vec![],
            ),
            post.get("content_type"),
            post.get("content_value"),
            replies,
            post.get("likes"),
            post.get("created_at"),
            post.get("last_modified"),
        ))
    }

    pub async fn save(
        db: &DbController,
        post: NewExpressionPost,
        author: String,
    ) -> Result<Self, Error> {
        let profile = UserProfile::get_by_id(db, author).await?;

        let post_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO expression_posts
                (
                    id, 
                    title, 
                    subtitle, 
                    author, 
                    content_type, 
                    content_value
                ) 
            VALUES (?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&post_id)
        .bind(&post.title)
        .bind(&post.subtitle)
        .bind(&profile.id)
        .bind(&post.content.kind)
        .bind(&post.content.value)
        .execute(&db.community_pool)
        .await?;

        let row = sqlx::query(
            r#"
            SELECT 
                created_at, 
                last_modified 
            FROM expression_posts 
            WHERE id = ?
        "#,
        )
        .bind(&post_id)
        .fetch_one(&db.community_pool)
        .await?;

        Ok(ExpressionPost {
            id: post_id,
            title: post.title,
            subtitle: post.subtitle,
            author: UserProfile::new(profile.id, profile.username, profile.avatar, profile.likes),
            content: ExpressionPostContent {
                kind: post.content.kind,
                value: post.content.value,
            },
            replies: vec![],
            likes: 0,
            created_at: row.get("created_at"),
            last_modified: row.get("last_modified"),
        })
    }

    pub async fn update_likes(
        db: &DbController,
        update_request: UpdateLikesRequest,
        user_id: String,
    ) -> Result<(), Error> {
        let statement = if update_request.update_value == 1 {
            r#"
                INSERT INTO likes
                    (
                        parent_id, 
                        author
                    ) VALUES (?, ?)
            "#
        } else {
            r#"
                DELETE FROM likes 
                WHERE parent_id = ? 
                AND author = ?
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
        if sqlx::query(
            r#"
            INSERT INTO replies
                (
                    id, 
                    author, 
                    parent, 
                    content
                ) 
            VALUES(?, ?, ?, ?)
        "#,
        )
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

        let row = sqlx::query(
            r#"
            SELECT 
                created_at, 
                last_modified 
            FROM replies 
            WHERE id = ?
        "#,
        )
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

    pub async fn get_recent_posts(db: &DbController, limit: u16) -> Result<Vec<Self>, Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                post.id AS id, 
                post.title AS title, 
                post.subtitle AS subtitle, 
                profile.id AS author_id, 
                profile.username AS author_username, 
                profile.avatar AS author_avatar, 
                post.content_type AS content_type, 
                post.content_value AS content_value, 
                post.created_at AS created_at, 
                post.last_modified AS last_modified
            FROM expression_posts AS post
            JOIN user_profiles AS profile ON profile.id = post.author
            WHERE post.created_at > now() - interval 7 day
            GROUP BY post.id
            ORDER BY post.created_at
            DESC LIMIT ?
        "#,
        )
        .bind(limit)
        .fetch_all(&db.community_pool)
        .await?;

        let posts: Vec<ExpressionPost> = rows
            .iter()
            .map(|x| {
                ExpressionPost::new(
                    x.get("id"),
                    x.get("title"),
                    x.try_get("subtitle").unwrap_or_else(|_| String::new()),
                    UserProfile::new(
                        x.get("author_id"),
                        x.get("author_username"),
                        x.get("author_avatar"),
                        vec![],
                    ),
                    x.get("content_type"),
                    x.get("content_value"),
                    vec![],
                    0,
                    x.get("created_at"),
                    x.get("last_modified"),
                )
            })
            .collect();

        Ok(posts)
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
