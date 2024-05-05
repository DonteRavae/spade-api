use std::sync::Arc;

use async_graphql::*;
use tower_cookies::Cookies;

use crate::{
    auth::AccessToken,
    community::{
        models::{expression_post::ExpressionPostAggregate, user_profile::UserProfile},
        ExpressionPost,
    },
    db::DbController,
    GatewayResponse,
};

pub struct Query;

#[Object]
impl Query {
    async fn get_logged_in_user_profile(
        &self,
        ctx: &Context<'_>,
    ) -> Result<GatewayResponse<UserProfile>> {
        // Get access cookie from headers
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Get Logged In User Profile");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Get Logged In User Profile");
            return Ok(GatewayResponse::new(
                false,
                Some("It seems we have a problem. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Decode access token and logout user in database
        let access_token = cookie.value();
        let Ok(claims) = AccessToken::decode(access_token) else {
            eprintln!("JWT ERROR: Error decoding access token in Get Logged In User Profile");
            return Ok(GatewayResponse::new(
                false,
                Some("Error getting user. Please try again.".to_string()),
                None,
                400,
            ));
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Get Logged In User Profile");
            return Ok(GatewayResponse::new(
                false,
                Some("Error getting user. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let Ok(profile) = UserProfile::get_by_id(db, claims.sub).await else {
            return Ok(GatewayResponse::new(
                false,
                Some(String::from("Unable to find user profile.")),
                None,
                400,
            ));
        };

        Ok(GatewayResponse::new(true, None, Some(profile), 200))
    }

    async fn get_expression_post(
        &self,
        ctx: &Context<'_>,
        post_id: String,
    ) -> Result<GatewayResponse<ExpressionPost>> {
        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Get Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Error getting expression post. Please try again.".to_string()),
                None,
                500,
            ));
        };

        match ExpressionPost::get_by_id(db, post_id).await {
            Ok(post) => return Ok(GatewayResponse::new(true, None, Some(post), 200)),
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        }
    }

    async fn get_recent_posts(
        &self,
        ctx: &Context<'_>,
        mut limit: Option<u16>,
    ) -> Result<GatewayResponse<ExpressionPostAggregate>> {
        if limit.is_some() && limit == Some(0) || limit.is_none() {
            limit = Some(20);
        }

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Get Recent Expression Posts");
            return Ok(GatewayResponse::new(
                false,
                Some("Error getting recent expression posts. Please try again.".to_string()),
                None,
                500,
            ));
        };

        match ExpressionPost::get_recent_posts(db, limit.unwrap()).await {
            Ok(posts) => Ok(GatewayResponse::new(
                true,
                None,
                Some(ExpressionPostAggregate { posts }),
                200,
            )),
            Err(err) => Ok(GatewayResponse::new(false, Some(err), None, 500)),
        }
    }

    async fn get_trending_posts(
        &self,
        ctx: &Context<'_>,
        mut limit: Option<u16>,
    ) -> Result<GatewayResponse<ExpressionPostAggregate>> {
        if limit.is_some() && limit == Some(0) || limit.is_none() {
            limit = Some(20);
        }

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Get Trending Expression Posts");
            return Ok(GatewayResponse::new(
                false,
                Some("Error getting trending expression posts. Please try again.".to_string()),
                None,
                500,
            ));
        };

        match ExpressionPost::get_trending_posts(db, limit.unwrap()).await {
            Ok(posts) => Ok(GatewayResponse::new(
                true,
                None,
                Some(ExpressionPostAggregate { posts }),
                200,
            )),
            Err(err) => Ok(GatewayResponse::new(false, Some(err), None, 500)),
        }
    }
}
