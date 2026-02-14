use chrono::NaiveDateTime;
use chrono::Utc;
use std::sync::Arc;
use std::{env, error::Error};
use zigistry::codeberg;
use zigistry::database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let timer_start = Utc::now();
    let pool = Arc::new(database::connect_to_database().await.unwrap());
    codeberg::codeberg_main(Arc::clone(&pool)).await.unwrap();
    eprintln!(
        "Codeberg completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );
    Ok(())
}
