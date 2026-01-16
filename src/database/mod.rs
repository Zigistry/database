use database_schema::INIT_REPOS;

use futures::StreamExt;
use sqlx::Executor;
use sqlx::{self, Pool, Sqlite};
mod database_schema;

const DATABASE_SCHEMA: &str = include_str!("../../databaseSchema.sql");

pub async fn init_database(db: Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    let _ = db.execute(DATABASE_SCHEMA).await.unwrap();

    Ok(())
}
