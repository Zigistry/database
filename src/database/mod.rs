use sqlx::{self, Executor, SqlitePool};
use std::fs::File;

const DATABASE_SCHEMA: &str = include_str!("../../Database_SQL_Files/database_schema.sql");

pub async fn init_database() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    File::create("./zigistry.db")?;
    let pool = SqlitePool::connect("sqlite:./zigistry.db").await?;
    pool.execute(DATABASE_SCHEMA).await.unwrap();
    Ok(pool)
}

pub async fn wrap_up(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    pool.execute("VACUUM").await.unwrap();
    Ok(())
}
