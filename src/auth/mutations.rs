use std::sync::Arc;

use async_graphql::*;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    auth::models::{AuthAccessRequest, Email, Password},
    db::DbController,
};

use super::{
    models::{AuthRegistrationRequest, AuthResponse},
    AccessToken, Auth, AuthError,
};

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
        let (access_token, refresh_token) = Auth::register(db, email, password).await?;

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
        let (access_token, refresh_token) = Auth::login(db, email, password).await?;

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

    async fn update_email(&self, ctx: &Context<'_>, email: String) -> Result<bool, Error> {
        // Validate input
        let email = Email::parse(email)?;

        // Update email in database
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to update email.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let community_id = AccessToken::decode(cookie.value())?.sub;

        let db = ctx.data::<Arc<DbController>>()?;

        Ok(Auth::update_email(db, email, community_id).await?)
    }

    async fn update_password(
        &self,
        ctx: &Context<'_>,
        new_password: String,
    ) -> Result<bool, Error> {
        // Validate input
        let new_password = Password::parse(new_password)?;

        // Update password in database
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to update password.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let community_id = AccessToken::decode(cookie.value())?.sub;

        let db = ctx.data::<Arc<DbController>>()?;

        Ok(Auth::update_password(db, new_password, community_id).await?)
    }

    async fn permanent_delete(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let Some(cookie) = ctx.data::<Cookies>()?.get("sat") else {
            return Err(
                AuthError::Unauthorized("Please log in to update password.".to_string())
                    .extend_with(|_, e| e.set("code", 401)),
            );
        };

        let community_id = AccessToken::decode(cookie.value())?.sub;

        let db = ctx.data::<Arc<DbController>>()?;

        Ok(Auth::delete(db, community_id).await?)
    }
}
