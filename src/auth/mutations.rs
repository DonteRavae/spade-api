use std::sync::Arc;

use async_graphql::*;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    auth::models::{AuthAccessRequest, Email, Password},
    community::UserProfile,
    db::DbController,
    GatewayResponse,
};

use super::{models::AuthRegistrationRequest, AccessToken, Auth};

pub struct Mutation;

#[Object]
impl Mutation {
    async fn register(
        &self,
        ctx: &Context<'_>,
        registration: AuthRegistrationRequest,
    ) -> Result<GatewayResponse<UserProfile>> {
        // Validate inputs
        let email = match Email::parse(registration.email) {
            Ok(str) => str,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        let password = match Password::parse(registration.password) {
            Ok(str) => str,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        // Retrieve database controller from state and register user
        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Register");
            return Ok(GatewayResponse::new(
                false,
                Some("Error creating user. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let (access_token, refresh_token) =
            match Auth::register(db, email, password, registration.username).await {
                Ok((at, rt)) => (at, rt),
                Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
            };

        // Once user is registered in database, create cookies containing access and refresh tokens
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Register");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let access_cookie = Cookie::build(("sat", access_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(1))
            .same_site(SameSite::Strict)
            .build();
        let refresh_cookie = Cookie::build(("srt", refresh_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(14))
            .same_site(SameSite::Strict)
            .build();

        cookies.add(refresh_cookie);
        cookies.add(access_cookie);

        Ok(GatewayResponse::new(true, None, None, 201))
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        credentials: AuthAccessRequest,
    ) -> Result<GatewayResponse<UserProfile>> {
        // Validate inputs
        let email = match Email::parse(credentials.email) {
            Ok(str) => str,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        let password = match Password::parse(credentials.password) {
            Ok(str) => str,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        // Retrieve database controller from state and register user
        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Login");
            return Ok(GatewayResponse::new(
                false,
                Some("Error logging in. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let (access_token, refresh_token) = match Auth::login(db, email, password).await {
            Ok((at, rt)) => (at, rt),
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        };

        // Once user is registered in database, create cookies containing access and refresh tokens
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Login");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let access_cookie = Cookie::build(("sat", access_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(1))
            .same_site(SameSite::Strict)
            .build();
        let refresh_cookie = Cookie::build(("srt", refresh_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(14))
            .same_site(SameSite::Strict)
            .build();

        cookies.add(access_cookie);
        cookies.add(refresh_cookie);

        Ok(GatewayResponse::new(true, None, None, 200))
    }

    async fn update_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<GatewayResponse<UserProfile>> {
        // Validate input
        let email = match Email::parse(email) {
            Ok(str) => str,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Update Email");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        // Update email in database
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Update Email");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to update email".to_string()),
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
            eprintln!("SERVER ERROR: Error getting database in Update Email");
            return Ok(GatewayResponse::new(
                false,
                Some("Error updating email. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = Auth::update_email(db, email, community_id).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        } else {
            return Ok(GatewayResponse::new(true, None, None, 200));
        }
    }

    async fn update_password(
        &self,
        ctx: &Context<'_>,
        new_password: String,
    ) -> Result<GatewayResponse<UserProfile>> {
        // Validate input
        let new_password = match Password::parse(new_password) {
            Ok(pwd) => pwd,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 400)),
        };

        // Update password in database
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Update Password");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Update Password");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to update password".to_string()),
                None,
                401,
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
            eprintln!("SERVER ERROR: Error getting database in Update Password");
            return Ok(GatewayResponse::new(
                false,
                Some("Error updating password. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = Auth::update_password(db, new_password, community_id).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        } else {
            return Ok(GatewayResponse::new(true, None, None, 200));
        }
    }

    async fn permanent_delete(
        &self,
        ctx: &Context<'_>,
    ) -> Result<GatewayResponse<UserProfile>> {
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Permanent Delete");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Permanent Delete");
            return Ok(GatewayResponse::new(
                false,
                Some("Please log in to delete user".to_string()),
                None,
                401,
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
            eprintln!("SERVER ERROR: Error getting database in Permanent Delete");
            return Ok(GatewayResponse::new(
                false,
                Some("Error deleting user. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = Auth::delete(db, community_id).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        } else {
            return Ok(GatewayResponse::new(true, None, None, 204));
        }
    }
}
