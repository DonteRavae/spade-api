use std::sync::Arc;

use async_graphql::*;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{community::UserProfile, db::DbController, GatewayResponse};

use super::{AccessToken, Auth, RefreshToken};

pub struct Query;

#[Object]
impl Query {
    async fn logout(&self, ctx: &Context<'_>) -> Result<GatewayResponse<UserProfile>> {
        // Get access cookie from headers
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Logout");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };
        let Some(cookie) = cookies.get("sat") else {
            eprintln!("SERVER ERROR: Access token doesn't exist in Logout");
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
            eprintln!("JWT ERROR: Error decoding access token in Logout");
            return Ok(GatewayResponse::new(
                false,
                Some("Error logging out. Please try again.".to_string()),
                None,
                400,
            ));
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Logout");
            return Ok(GatewayResponse::new(
                false,
                Some("Error logging out. Please try again.".to_string()),
                None,
                500,
            ));
        };

        if let Err(err) = Auth::logout(db, &claims.sub).await {
            return Ok(GatewayResponse::new(false, Some(err), None, 500));
        }

        // Remove cookies from cookie jar
        cookies.remove(Cookie::new("sat", ""));
        cookies.remove(Cookie::new("srt", ""));

        Ok(GatewayResponse::new(true, None, None, 204))
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<GatewayResponse<UserProfile>> {
        // Get refresh cookie from headers
        let Ok(cookies) = ctx.data::<Cookies>() else {
            eprintln!("SERVER ERROR: Error getting cookies in Refresh");
            return Ok(GatewayResponse::new(
                false,
                Some("Server error. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let Some(cookie) = cookies.get("srt") else {
            eprintln!("SERVER ERROR: Refresh token doesn't exist in Refresh");
            return Ok(GatewayResponse::new(
                false,
                Some("It seems we have a problem. Please try again.".to_string()),
                None,
                401,
            ));
        };
        // Decode refresh token and if valid, issue user a new access token
        let Ok(claims) = RefreshToken::decode(cookie.value()) else {
            eprintln!("JWT ERROR: Error decoding refresh token in Refresh");
            return Ok(GatewayResponse::new(
                false,
                Some("Error logging out. Please try again.".to_string()),
                None,
                400,
            ));
        };

        let Ok(db) = ctx.data::<Arc<DbController>>() else {
            eprintln!("SERVER ERROR: Error getting database in Refresh");
            return Ok(GatewayResponse::new(
                false,
                Some("Error logging out. Please try again.".to_string()),
                None,
                500,
            ));
        };

        let access_token = match Auth::refresh(db, &claims.sub).await {
            Ok(token) => token,
            Err(err) => return Ok(GatewayResponse::new(false, Some(err), None, 500)),
        };

        // Create a new cookie with access token and add to cookie jar
        let access_cookie = Cookie::build(("sat", access_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(1))
            .same_site(SameSite::Strict)
            .build();

        cookies.add(access_cookie);

        Ok(GatewayResponse::new(true, None, None, 204))
    }
}
