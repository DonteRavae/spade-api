use std::sync::Arc;

use async_graphql::{Context, Error, ErrorExtensions, Object};
use tower_cookies::Cookies;

use crate::{
    auth::{AccessToken, AuthError},
    community::models::user_profile::UserProfile,
    db::DbController,
};

use super::models::expression_post::ExpressionPost;

pub struct Query;

#[Object]
impl Query {
    async fn get_user_profile(&self, ctx: &Context<'_>) -> Result<UserProfile, Error> {
        let Some(cookie): Option<tower_cookies::Cookie<'_>> = ctx.data::<Cookies>()?.get("sat")
        else {
            return Err(AuthError::Unauthorized(
                "User must be signed in to retrieve profile.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 401))); // Extend with message
        };

        let access_token_claims = AccessToken::decode(cookie.value())?;

        let db = ctx.data::<Arc<DbController>>()?;

        UserProfile::get_by_id(db, access_token_claims.sub).await
    }

    async fn get_expression_post(
        &self,
        ctx: &Context<'_>,
        post_id: String,
    ) -> Result<ExpressionPost, Error> {
        let db = ctx.data::<Arc<DbController>>()?;
        ExpressionPost::get_by_id(db, post_id).await
    }
}
