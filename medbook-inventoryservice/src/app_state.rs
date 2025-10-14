use anyhow::Result;

use rmq_wrappers::Rmq;

use crate::db::{self, DbPool};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub rmq_client: Rmq,
}

impl AppState {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            db_pool: db::connect(&std::env::var("DATABASE_URL")?).await?,
            rmq_client: Rmq::connect(&std::env::var("RMQ_URL")?).await?,
        })
    }
}
