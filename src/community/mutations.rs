use std::sync::Arc;

use async_graphql::*;
use tower_cookies::Cookies;

use crate::{
    auth::AccessToken,
    community::models::{expression_post::NewExpressionPost, reply::Reply},
    db::DbController,
    GatewayResponse,
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
    ) -> Result<GatewayResponse<ExpressionPost>> {
        // Check if user is authenticated
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Create New Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Create New Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to create new expression post".to_string()),
                None,
                400,
            ));
        };

        let community_id = match AccessToken::decode(cookie.value()) {
            Ok(claims) => claims.sub,
            Err(_) => {
                return Ok(GatewayResponse::new(
                    false,
                    Some("Invalid user".to_string()),
                    None,
                    400,
                ))
            }
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Create New Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Error creating expression post. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let post = match ExpressionPost::save(db, post, community_id).await {
            Ok(post) => post,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        };

        Ok(GatewayResponse::new(true, None, Some(post), 201))
    }

    pub async fn update_expression_post(
        &self,
        ctx: &Context<'_>,
        request: UpdateContentRequest,
    ) -> Result<GatewayResponse<ExpressionPost>> {
        // Check if user is authenticated
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Update Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Update Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to update expression post".to_string()),
                None,
                400,
            ));
        };

        let logged_in_user = match AccessToken::decode(cookie.value()) {
            Ok(claims) => claims.sub,
            Err(_) => {
                return Ok(GatewayResponse::new(
                    false,
                    Some("Invalid user".to_string()),
                    None,
                    400,
                ))
            }
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Update Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Error updating expression post. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let post = match ExpressionPost::update_content(db, request, logged_in_user).await {
            Ok(post) => post,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        };

        Ok(GatewayResponse::new(true, None, Some(post), 200))
    }

    pub async fn update_likes(
        &self,
        ctx: &Context<'_>,
        request: UpdateLikesRequest,
    ) -> Result<GatewayResponse<ExpressionPost>> {
        // Check if user is authenticated
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Update Likes");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Update Likes");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to add like.".to_string()),
                None,
                400,
            ));
        };

        let logged_in_user = match AccessToken::decode(cookie.value()) {
            Ok(claims) => claims.sub,
            Err(_) => {
                return Ok(GatewayResponse::new(
                    false,
                    Some("Invalid user".to_string()),
                    None,
                    400,
                ))
            }
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Update Likes");
            return Ok(GatewayResponse::new(
                false,
                Some("Error add like. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = ExpressionPost::update_likes(db, request, logged_in_user).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        }

        Ok(GatewayResponse::new(true, None, None, 200))
    }

    pub async fn reply_to_expression(
        &self,
        ctx: &Context<'_>,
        request: NewReplyRequest,
    ) -> Result<GatewayResponse<Reply>> {
        // Check if user is authenticated
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Reply To Expression");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Reply To Expression");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to reply to expression post.".to_string()),
                None,
                400,
            ));
        };

        let logged_in_user = match AccessToken::decode(cookie.value()) {
            Ok(claims) => claims.sub,
            Err(_) => {
                return Ok(GatewayResponse::new(
                    false,
                    Some("Invalid user".to_string()),
                    None,
                    400,
                ))
            }
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Reply To Expression");
            return Ok(GatewayResponse::new(
                false,
                Some("Error replying to expression post. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let reply = match ExpressionPost::add_reply(db, logged_in_user, request).await {
            Ok(reply) => reply,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        };

        Ok(GatewayResponse::new(true, None, Some(reply), 200))
    }

    pub async fn delete_expression_post(
        &self,
        ctx: &Context<'_>,
        post_id: String,
    ) -> Result<GatewayResponse<ExpressionPost>> {
        // Check if user is authenticated
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Delete Expressin Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Delete Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to delete expression post.".to_string()),
                None,
                400,
            ));
        };

        let logged_in_user = match AccessToken::decode(cookie.value()) {
            Ok(claims) => claims.sub,
            Err(_) => {
                return Ok(GatewayResponse::new(
                    false,
                    Some("Invalid user".to_string()),
                    None,
                    400,
                ))
            }
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Delete Expression Post");
            return Ok(GatewayResponse::new(
                false,
                Some("Error deleting expression post. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = ExpressionPost::delete(db, post_id, logged_in_user).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        } else {
            return Ok(GatewayResponse::new(true, None, None, 204));
        }
    }
}
