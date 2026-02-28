use chrono::{NaiveDateTime, Utc};
use std::error::Error;
use std::sync::Arc;
use zigistry::{database, github};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await?);
    let started_at = Utc::now();

    {
        let current_time = Utc::now();
        let now = current_time.naive_utc();
        let from = now - chrono::Duration::days(3 * 365);

        github::github_main_cron(Arc::clone(&pool), from, now, 50, "stars:0..9").await?;

        eprintln!(
            "cron-github stars:0..9 finished in {} minutes.",
            (Utc::now() - current_time).num_minutes(),
        );
    }

    {
        let current_time = Utc::now();
        let now = current_time.naive_utc();
        let from = now - chrono::Duration::days(5 * 365);

        github::github_main_cron(Arc::clone(&pool), from, now, 50, "stars:10..20").await?;

        eprintln!(
            "cron-github stars:10..20 finished in {} minutes.",
            (Utc::now() - current_time).num_minutes(),
        );
    }

    {
        let current_time = Utc::now();
        let now = current_time.naive_utc();
        let from = NaiveDateTime::parse_from_str("2016-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")?;

        github::github_main_cron(Arc::clone(&pool), from, now, 50, "stars:>20").await?;

        eprintln!(
            "cron-github stars:>20 finished in {} minutes.",
            (Utc::now() - current_time).num_minutes(),
        );
    }

    {
        let current_time = Utc::now();

        github::run_cron_update_once(Arc::clone(&pool)).await?;

        eprintln!(
            "cron-github updates finished in {} minutes.",
            (Utc::now() - current_time).num_minutes(),
        );
    }

    eprintln!(
        "cron-github finished in {} minutes.",
        (Utc::now() - started_at).num_minutes(),
    );
    Ok(())
}
