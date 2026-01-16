use sqlx::{self, Executor, Pool, SqlitePool};
use std::fs::File;
mod database_schema;

const DATABASE_SCHEMA: &str = include_str!("../../databaseSchema.sql");

pub async fn init_database() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    File::create("./zigistry.db")?;
    let pool = SqlitePool::connect("sqlite:./zigistry.db").await?;
    pool.execute(DATABASE_SCHEMA).await.unwrap();
    Ok(pool)
}
