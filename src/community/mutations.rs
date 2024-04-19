use std::sync::Arc;

use async_graphql::{Context, Error, ErrorExtensions, Object};
use tower_cookies::Cookies;

use crate::{
    auth::{AccessToken, AuthError},
    community::{
        models::{expression_post::NewExpressionPost, reply::Reply},
        user_profile::{NewProfileRequest, UserProfile},
    },
    db::DbController,
};

use super::models::{
    expression_post::{ExpressionPost, UpdateLikesRequest},
    reply::NewReplyRequest,
};

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_user_profile(
        &self,
        ctx: &Context<'_>,
        request: NewProfileRequest,
    ) -> Result<UserProfile, Error> {
        // Check if user is authenticated
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(AuthError::Unauthorized(
                "Please log in to create a user profile.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 401)));
        };

        let access_token_claims = AccessToken::decode(cookie.value())?;

        let db = ctx.data::<Arc<DbController>>()?;

        UserProfile::register(db, access_token_claims.sub, request).await
    }

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

        println!("{}", cookie.value());

        ExpressionPost::add_reply(db, access_token_claims.sub, request).await
    }
}
