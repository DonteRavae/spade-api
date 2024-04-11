use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::ApplicationState;

use super::types::{Auth, AuthResponse, JwtManager, UserAccessRequest, UserRegistrationRequest};

pub struct AuthHandlers;

impl AuthHandlers {
    pub async fn register_user(
        cookies: Cookies,
        State(state): State<Arc<ApplicationState>>,
        Json(request): Json<UserRegistrationRequest>,
    ) -> impl IntoResponse {
        if !Auth::validate_email(&request.email) || !Auth::validate_password(&request.password) {
            return (
                StatusCode::BAD_REQUEST,
                Json(AuthResponse::new(
                    false,
                    Some("Please enter a valid email or password".to_string()),
                )),
            );
        }

        // Retrieve from database
        match state.db.register(request).await {
            Ok((access_token, refresh_token)) => {
                // Add Access and Refresh Tokens to cookie jar
                let access_cookie = Cookie::build(("sat", access_token))
                    .http_only(true)
                    .secure(false)
                    .max_age(Duration::days(1))
                    .same_site(SameSite::Strict)
                    .build();

                let refresh_cookie = Cookie::build(("srt", refresh_token))
                    .http_only(true)
                    .secure(false)
                    .max_age(Duration::days(14))
                    .same_site(SameSite::Strict)
                    .build();

                cookies.add(access_cookie);
                cookies.add(refresh_cookie);

                (
                    StatusCode::CREATED,
                    Json(AuthResponse::new(
                        true,
                        Some("Successfully created new user. Welcome!".to_string()),
                    )),
                )
            }
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse::new(false, Some(err))),
            ),
        }
    }

    pub async fn login_user(
        cookies: Cookies,
        State(state): State<Arc<ApplicationState>>,
        Json(request): Json<UserAccessRequest>,
    ) -> impl IntoResponse {
        if !Auth::validate_email(&request.email) || !Auth::validate_password(&request.password) {
            return (
                StatusCode::BAD_REQUEST,
                Json(AuthResponse::new(
                    false,
                    Some("Please enter a valid email or password".to_string()),
                )),
            );
        }

        match state.db.login(request).await {
            Ok((access_token, refresh_token)) => {
                // Add Access and Refresh Tokens to cookie jar
                let access_cookie = Cookie::build(("sat", access_token))
                    .http_only(true)
                    .secure(false)
                    .max_age(Duration::days(1))
                    .same_site(SameSite::Strict)
                    .build();

                let refresh_cookie = Cookie::build(("srt", refresh_token))
                    .http_only(true)
                    .secure(false)
                    .max_age(Duration::days(14))
                    .same_site(SameSite::Strict)
                    .build();

                cookies.add(access_cookie);
                cookies.add(refresh_cookie);

                (
                    StatusCode::OK,
                    Json(AuthResponse::new(
                        true,
                        Some("Successful login. Welcome!".to_string()),
                    )),
                )
            }
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse::new(false, Some(err))),
            ),
        }
    }

    pub async fn logout_user(
        cookies: Cookies,
        State(state): State<Arc<ApplicationState>>,
    ) -> impl IntoResponse {
        let Some(access_token) = cookies.get("sat") else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse::new(
                    false,
                    Some("Oops. Please try again.".to_string()),
                )),
            );
        };

        let Ok(claims) = JwtManager::decode_access_token(access_token.value()) else {
            return (
                StatusCode::BAD_REQUEST,
                Json(AuthResponse::new(false, Some("Invalid token.".to_string()))),
            );
        };
        match state.db.logout(&claims.sub).await {
            Ok(_) => {
                cookies.remove(Cookie::new("sat", ""));
                (StatusCode::OK, Json(AuthResponse::new(true, None)))
            }
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse::new(false, Some(err))),
            ),
        }
    }

    pub async fn refresh_user(
        cookies: Cookies,
        State(state): State<Arc<ApplicationState>>,
    ) -> impl IntoResponse {
        let Some(refresh_cookie) = cookies.get("srt") else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse::new(false, None)),
            );
        };

        let Ok(claims) = JwtManager::decode_refresh_token(refresh_cookie.value()) else {
            return (
                StatusCode::FORBIDDEN,
                Json(AuthResponse::new(false, Some("Invalid token".to_string()))),
            );
        };

        match state.db.refresh(&claims.sub).await {
            Ok(access_token) => {
                let access_cookie = Cookie::build(("sat", access_token))
                    .http_only(true)
                    .secure(false)
                    .max_age(Duration::days(1))
                    .same_site(SameSite::Strict)
                    .build();

                cookies.add(access_cookie);

                return (StatusCode::NO_CONTENT, Json(AuthResponse::new(true, None)));
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse::new(false, Some(err))),
                )
            }
        }
    }
}
