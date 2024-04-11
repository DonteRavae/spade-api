use std::error::Error;

use sqlx::{MySql, MySqlPool, Pool};

mod auth;

#[derive(Debug)]
pub struct DbController {
    pool: Pool<MySql>,
}

impl DbController {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let db_url: String = dotenv::var("DB_URL")?;

        Ok(Self {
            pool: MySqlPool::connect(&db_url).await?,
        })
    }
}
