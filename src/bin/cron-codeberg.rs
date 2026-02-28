use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use zigistry::{codeberg, database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await?);
    let started_at = Utc::now();

    codeberg::codeberg_main(Arc::clone(&pool)).await?;

    eprintln!(
        "cron-codeberg finished in {} minutes.",
        (Utc::now() - started_at).num_minutes(),
    );
    Ok(())
}
