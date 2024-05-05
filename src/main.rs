use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use spade_api::ApplicationState;
use std::error::Error;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    spade_api::welcome();

    let app_state = ApplicationState::init().await;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap());

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
        .layer(cors)
        .layer(CookieManagerLayer::new());

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
