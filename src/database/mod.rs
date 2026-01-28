use dotenv::dotenv;
use libsql::{Builder, Connection};
// use std::env;

const START_FROM_SCRATCH: bool = false;

const DATABASE_SCHEMA: &str = include_str!("../../Database_SQL_Files/database_schema.sql");

pub async fn connect_to_database() -> Result<Connection, Box<dyn std::error::Error>> {
    dotenv().ok();
    // let db = Builder::new_remote(
    //     env::var("DATABASE_URL").expect("DATABASE_URL not found"),
    //     env::var("API_KEY").expect("API_KEY not found"),
    // )
    let db = Builder::new_local("./zigistry.db").build().await.unwrap();
    let pool = db.connect().unwrap();
    if START_FROM_SCRATCH {
        pool.execute_batch(DATABASE_SCHEMA).await.unwrap();
    }
    Ok(pool)
}
