use std::vec;

use crate::{community::CommunityError, db::DbController};
use async_graphql::{Error, ErrorExtensions, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
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
    cover_image: Option<String>,
    author: UserProfile,
    content: ExpressionPostContent,
    replies: Vec<Reply>,
    reply_count: i32,
    likes: i32,
    created_at: DateTime<Utc>,
    last_modified: DateTime<Utc>,
}

impl ExpressionPost {
    pub fn new(
        id: String,
        title: String,
        subtitle: Option<String>,
        cover_image: Option<String>,
        author: UserProfile,
        content_type: String,
        content_value: String,
        replies: Vec<Reply>,
        reply_count: i32,
        likes: i32,
        created_at: DateTime<Utc>,
        last_modified: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            subtitle,
            cover_image,
            author,
            content: ExpressionPostContent {
                kind: content_type,
                value: content_value,
            },
            replies,
            reply_count,
            likes,
            created_at,
            last_modified,
        }
    }

    pub async fn get_by_id(db: &DbController, id: String) -> Result<Self, Error> {
        // Get post from database
        let Ok(post) = sqlx::query(
            r#"
            SELECT
                post.id AS id, 
                post.title AS title, 
                post.subtitle AS subtitle,
                post.cover_image AS cover_image,
                profile.id AS author_id, 
                profile.username AS author_username, 
                profile.avatar AS author_avatar, 
                post.content_type AS content_type, 
                post.content_value AS content_value, 
                post.created_at AS created_at, 
                post.last_modified AS last_modified,
                IFNULL(COUNT(reply.id), 0) as reply_count,
                (
                    SELECT IFNULL(COUNT(parent_id), 0) FROM likes WHERE parent_id = post.id
                ) AS likes
            FROM expression_posts AS post
            LEFT JOIN user_profiles AS profile ON profile.id = post.author
            LEFT JOIN replies AS reply ON reply.parent = post.id
            WHERE post.id = ?
            GROUP BY post.id
        "#,
        )
        .bind(id)
        .fetch_one(&db.community_pool)
        .await
        else {
            return Err(
                CommunityError::BadRequest("Expression post does not exist.".to_string()).extend(),
            );
        };

        let post_id: String = post.get("id");

        // Get replies to post from database
        let Ok(replies) = Reply::get_all_recursively(db, post_id.clone()).await else {
            return Err(CommunityError::ServerError(
                "Error retrieving expression post replies from database".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        };

        // Return post
        Ok(Self::new(
            post_id,
            post.get("title"),
            post.get("subtitle"),
            post.get("cover_image"),
            UserProfile::new(
                post.get("author_id"),
                post.get("author_username"),
                post.get("author_avatar"),
                vec![],
            ),
            post.get("content_type"),
            post.get("content_value"),
            replies,
            post.get("reply_count"),
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
                    cover_image,
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
        .bind(&post.cover_image)
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
            cover_image: post.cover_image,
            author: UserProfile::new(profile.id, profile.username, profile.avatar, profile.likes),
            content: ExpressionPostContent {
                kind: post.content.kind,
                value: post.content.value,
            },
            replies: vec![],
            reply_count: 0,
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
        let posts = sqlx::query(
            r#"
            SELECT
                post.id AS id, 
                post.title AS title, 
                post.subtitle AS subtitle, 
                post.cover_image AS cover_image,
                profile.id AS author_id, 
                profile.username AS author_username, 
                profile.avatar AS author_avatar, 
                post.content_type AS content_type, 
                post.content_value AS content_value,
                IFNULL(COUNT(reply.parent), 0) AS reply_count, 
                post.created_at AS created_at, 
                post.last_modified AS last_modified,
                (
                    SELECT IFNULL(COUNT(DISTINCT parent_id), 0) FROM likes WHERE parent_id = post.id
                ) AS likes
            FROM expression_posts AS post
            LEFT JOIN user_profiles AS profile ON profile.id = post.author
            LEFT JOIN replies AS reply ON reply.parent = post.id
            WHERE post.created_at > now() - interval 7 day
            GROUP BY post.id
            ORDER BY post.created_at
            DESC LIMIT ?
        "#,
        )
        .bind(limit)
        .map(|x| {
            ExpressionPost::new(
                x.get("id"),
                x.get("title"),
                x.get("subtitle"),
                x.get("cover_image"),
                UserProfile::new(
                    x.get("author_id"),
                    x.get("author_username"),
                    x.get("author_avatar"),
                    vec![],
                ),
                x.get("content_type"),
                x.get("content_value"),
                vec![],
                x.get("reply_count"),
                x.get("likes"),
                x.get("created_at"),
                x.get("last_modified"),
            )
        })
        .fetch_all(&db.community_pool)
        .await?;

        Ok(posts)
    }

    pub async fn get_trending_posts(db: &DbController, limit: u16) -> Result<Vec<Self>, Error> {
        // Change QUERY to find most liked within a two week span
        let posts = sqlx::query(
            r#"
            SELECT
                post.id AS id, 
                post.title AS title, 
                post.subtitle AS subtitle, 
                post.cover_image AS cover_image,
                profile.id AS author_id, 
                profile.username AS author_username, 
                profile.avatar AS author_avatar, 
                post.content_type AS content_type, 
                post.content_value AS content_value,
                IFNULL(COUNT(reply.parent), 0) AS reply_count, 
                post.created_at AS created_at, 
                post.last_modified AS last_modified,
                (
                    SELECT IFNULL(COUNT(DISTINCT parent_id), 0) FROM likes WHERE parent_id = post.id
                ) AS likes
            FROM expression_posts AS post
            LEFT JOIN user_profiles AS profile ON profile.id = post.author
            LEFT JOIN replies AS reply ON reply.parent = post.id
            WHERE post.created_at > now() - interval 14 day
            GROUP BY post.id
            ORDER BY likes
            DESC LIMIT ?
        "#,
        )
        .bind(limit)
        .map(|x| {
            ExpressionPost::new(
                x.get("id"),
                x.get("title"),
                x.get("subtitle"),
                x.get("cover_image"),
                UserProfile::new(
                    x.get("author_id"),
                    x.get("author_username"),
                    x.get("author_avatar"),
                    vec![],
                ),
                x.get("content_type"),
                x.get("content_value"),
                vec![],
                x.get("reply_count"),
                x.get("likes"),
                x.get("created_at"),
                x.get("last_modified"),
            )
        })
        .fetch_all(&db.community_pool)
        .await?;

        Ok(posts)
    }

    pub async fn update_content(
        db: &DbController,
        request: UpdateContentRequest,
        logged_in_user: String,
    ) -> Result<Self, Error> {
        let mut tx = db.community_pool.begin().await?;

        let author = sqlx::query("SELECT author FROM expression_posts WHERE id = ?")
            .bind(&logged_in_user)
            .fetch_optional(&mut *tx)
            .await?;

        // Check to make sure person deleting post is author
        if author.is_none() {
            return Err(
                CommunityError::BadRequest("Expression post does not exist.".to_string()).extend(),
            );
        } else {
            let author: String = author.unwrap().get("author");
            if author != logged_in_user {
                return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                    e.set(
                        "reason",
                        "User making request and expression post author do not match.",
                    )
                }));
            }
        }

        sqlx::query("UPDATE expression_posts SET content_type = ?, content_value = ? WHERE id = ?")
            .bind(request.content_type)
            .bind(request.content_value)
            .bind(&request.post_id)
            .execute(&mut *tx)
            .await?;

        let post = Self::get_by_id(db, request.post_id).await?;

        tx.commit().await?;

        Ok(post)
    }

    pub async fn delete(
        db: &DbController,
        post_id: String,
        logged_in_user: String,
    ) -> Result<bool, Error> {
        let mut tx = db.community_pool.begin().await?;

        let author = sqlx::query("SELECT author FROM expression_posts WHERE id = ?")
            .bind(&post_id)
            .fetch_optional(&mut *tx)
            .await?;

        // Check to make sure person deleting post is author
        if author.is_none() {
            return Err(
                CommunityError::BadRequest("Expression post does not exist.".to_string()).extend(),
            );
        } else {
            let author: String = author.unwrap().get("author");
            if author != logged_in_user {
                return Err(CommunityError::Unauthorized.extend_with(|_, e| {
                    e.set(
                        "reason",
                        "User making request and expression post author do not match.",
                    )
                }));
            }
        }

        // Delete post and likes
        if sqlx::query("DELETE FROM expression_posts WHERE id = ?")
            .bind(&post_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return Err(CommunityError::ServerError(
                "There seems to be an issue deleting this post. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        }

        if sqlx::query("DELETE FROM likes WHERE parent_id = ?")
            .bind(&post_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return Err(CommunityError::ServerError(
                "There seems to be an issue deleting this post. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        }

        // Delete all replies associated with post
        if Reply::delete_all_from_post(db, post_id).await.is_err() {
            return Err(CommunityError::ServerError(
                "There seems to be an issue deleting this post. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        };

        tx.commit().await?;
        Ok(true)
    }
}

/********** REQUEST OBJECTS **********/

/****** ADD VALIDATION CHECKS ******/
#[derive(InputObject, Debug)]
pub struct NewExpressionPost {
    title: String,
    subtitle: Option<String>,
    cover_image: Option<String>,
    content: ExpressionPostContent,
}

/****** ADD VALIDATION CHECKS ******/
#[derive(InputObject, Debug)]
pub struct UpdateLikesRequest {
    pub post_id: String,
    pub update_value: u8,
}

/****** ADD VALIDATION CHECKS ******/
#[derive(InputObject, Debug)]
pub struct UpdateContentRequest {
    post_id: String,
    content_type: String,
    content_value: String,
}
