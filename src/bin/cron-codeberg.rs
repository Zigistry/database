use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use zigistry::{codeberg, database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await?);
    let started_at = Utc::now();

    codeberg::codeberg_main_cron(Arc::clone(&pool)).await?;

    let update_start = Utc::now();
    codeberg::run_cron_update_once(Arc::clone(&pool)).await?;
    eprintln!(
        "cb updates finished in {} minutes.",
        (Utc::now() - update_start).num_minutes(),
    );

    eprintln!(
        "cb finished in {} minutes.",
        (Utc::now() - started_at).num_minutes(),
    );
    Ok(())
}
