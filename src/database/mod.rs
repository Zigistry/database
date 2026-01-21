use std::fs::File;

use libsql::{Builder, Connection};

const START_FROM_SCRATCH: bool = false;

const DATABASE_SCHEMA: &str = include_str!("../../Database_SQL_Files/database_schema.sql");

pub async fn connect_to_database() -> Result<Connection, Box<dyn std::error::Error>> {
    File::create("./zigistry.db")?;
    let db = Builder::new_remote(
        "libsql://my-remote-db.com".to_string(),
        "my-auth-token".to_string(),
    )
    .build()
    .await
    .unwrap();
    let pool = db.connect().unwrap();
    if START_FROM_SCRATCH {
        pool.execute(DATABASE_SCHEMA, ()).await.unwrap();
    }
    Ok(pool)
}

pub async fn wrap_up(pool: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    if START_FROM_SCRATCH {
        pool.execute("VACUUM", ()).await.unwrap();
    }
    Ok(())
}
