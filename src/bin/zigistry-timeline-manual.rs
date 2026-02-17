use chrono::NaiveDateTime;
use chrono::Utc;
use std::sync::Arc;
use std::{env, error::Error};
use zigistry::codeberg;
use zigistry::database;
use zigistry::github;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let pool = Arc::new(database::connect_to_database().await.unwrap());

    let start_date = NaiveDateTime::parse_from_str(&args[1], "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let end_date = NaiveDateTime::parse_from_str(&args[2], "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let step = args[3].parse::<u64>().unwrap();

    eprintln!("Starting");
    let timer_start = Utc::now();
    // Just found out that codeberg doesn't depend on date, time, or created at
    // the pagination is like, unlimited.
    codeberg::codeberg_main(Arc::clone(&pool)).await.unwrap();
    eprintln!(
        "Codeberg completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );

    let timer_start = Utc::now();
    eprintln!(
        "Sections completed successfully in {} minutes",
        (Utc::now() - timer_start).num_minutes()
    );

    github::github_main_0_9(Arc::clone(&pool), start_date, end_date, step)
        .await
        .unwrap();
    eprintln!(
        "GitHub completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );
    Ok(())
}
