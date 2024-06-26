use sqlx::{MySql, MySqlPool, Pool};
use std::error::Error;

#[derive(Debug)]
pub struct DbController {
    pub auth_pool: Pool<MySql>,
    pub community_pool: Pool<MySql>,
}

impl DbController {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let auth_db_url: String = dotenv::var("AUTH_DB_URL")?;
        let community_db_url: String = dotenv::var("COMMUNITY_DB_URL")?;

        Ok(Self {
            auth_pool: MySqlPool::connect(&auth_db_url).await?,
            community_pool: MySqlPool::connect(&community_db_url).await?,
        })
    }
}
