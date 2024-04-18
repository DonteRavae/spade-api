use std::sync::Arc;

use async_graphql::*;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    auth::{
        types::{AuthAccessRequest, Email, Password},
        AccessToken, AuthError, RefreshToken,
    },
    db::DbController,
};

use super::types::{AuthRegistrationRequest, AuthResponse};

pub struct Mutation;

#[Object]
impl Mutation {
    async fn register(
        &self,
        ctx: &Context<'_>,
        registration: AuthRegistrationRequest,
    ) -> Result<AuthResponse> {
        // Validate inputs
        let email = Email::parse(registration.email)?;
        let password = Password::parse(registration.password)?;

        // Retrieve database controller from state and register user
        let db = ctx.data::<Arc<DbController>>()?;
        let (access_token, refresh_token) = db.register(email, password).await?;

        // Once user is registered in database, create cookies containing access and refresh tokens
        let cookies = ctx.data::<Cookies>()?;
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

        Ok(AuthResponse::new(true, None))
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        credentials: AuthAccessRequest,
    ) -> Result<AuthResponse> {
        // Validate inputs
        let email = Email::parse(credentials.email)?;
        let password = Password::parse(credentials.password)?;

        // Retrieve database controller from state and register user
        let db = ctx.data::<Arc<DbController>>()?;
        let (access_token, refresh_token) = db.login(email, password).await?;

        // Once user is registered in database, create cookies containing access and refresh tokens
        let cookies = ctx.data::<Cookies>()?;
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

        Ok(AuthResponse::new(true, None))
    }

    async fn logout(&self, ctx: &Context<'_>) -> Result<AuthResponse> {
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

    async fn refresh(&self, ctx: &Context<'_>) -> Result<AuthResponse> {
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
