use std::sync::Arc;

use async_graphql::{Context, Error, ErrorExtensions, Object};
use tower_cookies::Cookies;

use crate::{
    auth::{AccessToken, AuthError},
    community::models::{expression_post::NewExpressionPost, reply::Reply},
    db::DbController,
};

use super::models::{
    expression_post::{ExpressionPost, UpdateContentRequest, UpdateLikesRequest},
    reply::NewReplyRequest,
};

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_new_expression_post(
        &self,
        ctx: &Context<'_>,
        post: NewExpressionPost,
    ) -> Result<ExpressionPost, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to create a new post.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let access_token_claims = AccessToken::decode(cookie.value())?;

        let db = ctx.data::<Arc<DbController>>()?;

        ExpressionPost::save(db, post, access_token_claims.sub).await
    }

    pub async fn update_expression_post(
        &self,
        ctx: &Context<'_>,
        request: UpdateContentRequest,
    ) -> Result<ExpressionPost, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to update content.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let logged_in_user = AccessToken::decode(cookie.value())?.sub;

        let db = ctx.data::<Arc<DbController>>()?;

        Ok(ExpressionPost::update_content(db, request, logged_in_user).await?)
    }

    pub async fn update_likes(
        &self,
        ctx: &Context<'_>,
        request: UpdateLikesRequest,
    ) -> Result<bool, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to like content.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let access_token_claims = AccessToken::decode(cookie.value())?;

        let db = ctx.data::<Arc<DbController>>()?;

        ExpressionPost::update_likes(db, request, access_token_claims.sub).await?;

        Ok(true)
    }

    pub async fn reply_to_expression(
        &self,
        ctx: &Context<'_>,
        request: NewReplyRequest,
    ) -> Result<Reply, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to comment.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let db = ctx.data::<Arc<DbController>>()?;

        let access_token_claims = AccessToken::decode(cookie.value())?;

        ExpressionPost::add_reply(db, access_token_claims.sub, request).await
    }

    pub async fn delete_expression_post(
        &self,
        ctx: &Context<'_>,
        post_id: String,
    ) -> Result<bool, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to comment.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let logged_in_user = AccessToken::decode(cookie.value())?.sub;

        let db = ctx.data::<Arc<DbController>>()?;

        Ok(ExpressionPost::delete(db, post_id, logged_in_user).await?)
    }
}
