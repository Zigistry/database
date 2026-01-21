use chrono::{NaiveDate, Utc};
use std::{env, error::Error};
use zigistry::codeberg;
use zigistry::database;
use zigistry::dependents_calculator::calculate_dependents;
use zigistry::github;
use zigistry::sections::fetch_repos_for_sections;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let pool = database::connect_to_database().await.unwrap();

    let start_date = NaiveDate::parse_from_str(&args[1], "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let end_date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%dT%H:%M:%SZ").unwrap();

    eprintln!("Starting");
    let timer_start = Utc::now();
    codeberg::codeberg_main(&pool, start_date, end_date)
        .await
        .unwrap();
    eprintln!(
        "Codeberg completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );

    let timer_start = Utc::now();
    fetch_repos_for_sections(&pool).await.unwrap();
    eprintln!(
        "Sections completed successfully in {} minutes",
        (Utc::now() - timer_start).num_minutes()
    );

    github::github_main(&pool, start_date, end_date)
        .await
        .unwrap();
    eprintln!(
        "GitHub completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );
    calculate_dependents(&pool).await;
    database::wrap_up(&pool).await.unwrap();
    Ok(())
}
