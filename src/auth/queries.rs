use std::sync::Arc;

use async_graphql::{Context, Error, ErrorExtensions, Object};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::db::DbController;

use super::{types::AuthResponse, AccessToken, AuthError, RefreshToken};

pub struct Query;

#[Object]
impl Query {
    async fn logout(&self, ctx: &Context<'_>) -> Result<AuthResponse, Error> {
        // Get access cookie from headers
        let cookies = ctx.data::<Cookies>()?;
        let Some(cookie) = cookies.get("sat") else {
            return Err(AuthError::ServerError(
                "It seems we have a problem. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        };

        // Decode access token and logout user in database
        let access_token = cookie.value();
        let claims = AccessToken::decode(access_token)?;
        let db = ctx.data::<Arc<DbController>>()?;
        db.logout(&claims.sub).await?;

        // Remove cookies from cookie jar
        cookies.remove(Cookie::new("sat", ""));
        cookies.remove(Cookie::new("srt", ""));

        Ok(AuthResponse::new(true, None))
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<AuthResponse, Error> {
        // Get refresh cookie from headers
        let cookies = ctx.data::<Cookies>()?;
        let Some(cookie) = cookies.get("srt") else {
            return Err(AuthError::ServerError(
                "It seems we have a problem. Please try again.".to_string(),
            )
            .extend_with(|_, e| e.set("code", 500)));
        };

        // Decode refresh token and if valid, issue user a new access token
        let refresh_token = cookie.value();
        let claims = RefreshToken::decode(refresh_token)?;
        let db = ctx.data::<Arc<DbController>>()?;
        let access_token = db.refresh(&claims.sub).await?;

        // Create a new cookie with access token and add to cookie jar
        let access_cookie = Cookie::build(("sat", access_token.as_str().to_string()))
            .http_only(true)
            .secure(false)
            .max_age(Duration::days(1))
            .same_site(SameSite::Strict)
            .build();
        cookies.add(access_cookie);

        Ok(AuthResponse::new(true, None))
    }
}
