use std::sync::Arc;

pub use auth::handlers::AuthHandlers;
pub use community::Query;
use db::DbController;

mod auth;
mod community;
mod db;

#[derive(Debug)]
pub struct ApplicationState {
    pub db: DbController,
}

impl ApplicationState {
    pub async fn init() -> Arc<ApplicationState> {
        Arc::new(ApplicationState {
            db: DbController::init()
                .await
                .expect("Error initializing database"),
        })
    }
}

pub fn welcome() {
    println!("SPADE Mental Health API!");
}
