use std::{error::Error, sync::Arc};

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use spade_api::{ApplicationState, AuthHandlers, Query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    spade_api::welcome();

    let app_state = ApplicationState::init().await;
    let community_schema = GraphQL::new(
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(Arc::clone(&app_state))
            .finish(),
    );

    let app = Router::new()
        .route("/auth/register", post(AuthHandlers::register_user))
        .route("/auth/login", post(AuthHandlers::login_user))
        .route("/auth/logout", get(AuthHandlers::logout_user))
        .route("/auth/refresh", get(AuthHandlers::refresh_user))
        .route(
            "/community",
            get(Html(
                GraphiQLSource::build().endpoint("/community").finish(),
            ))
            .post_service(community_schema),
        )
        .with_state(app_state)
        .layer(CookieManagerLayer::new());

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
