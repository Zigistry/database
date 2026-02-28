use chrono::{NaiveDateTime, Utc};
use std::error::Error;
use std::sync::Arc;
use zigistry::{database, github};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await?);
    let started_at = Utc::now();
    let now = started_at.naive_utc();
    let from = NaiveDateTime::parse_from_str("2016-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")?;

    github::github_main_cron(Arc::clone(&pool), from, now, 50, "stars:>20").await?;

    eprintln!(
        "cron-github-part-3 finished in {} minutes.",
        (Utc::now() - started_at).num_minutes(),
    );
    Ok(())
}
