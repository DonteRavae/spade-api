use axum::{routing::get, Router};
use spade_api::ApplicationState;
use std::error::Error;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    spade_api::welcome();

    let app_state = ApplicationState::init().await;

    let app = Router::new()
        .route(
            "/auth",
            get(spade_api::auth_playground).post(spade_api::auth_gateway),
        )
        .route(
            "/community",
            get(spade_api::community_playground).post(spade_api::community_gateway),
        )
        .with_state(app_state)
        .layer(CookieManagerLayer::new());

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
