use dotenv::dotenv;
use libsql::{Builder, Connection};
use std::env;

const START_FROM_SCRATCH: bool = false;

const DATABASE_SCHEMA: &str = include_str!("../../Database_SQL_Files/database_schema.sql");

pub async fn connect_to_database() -> Result<Connection, Box<dyn std::error::Error>> {
    dotenv().ok();
    let db = Builder::new_synced_database(
        "./database.db",
        env::var("DATABASE_URL").expect("DATABASE_URL not found"),
        env::var("API_KEY").expect("API_KEY not found"),
    ).sync_interval(std::time::Duration::from_hours(1));
    let client = db.build().await.unwrap();
    let pool = client.connect().unwrap();
    if START_FROM_SCRATCH {
        pool.execute_batch(DATABASE_SCHEMA).await.unwrap();
    }
    Ok(pool)
}
