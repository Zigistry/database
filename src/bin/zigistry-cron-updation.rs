use chrono::NaiveDateTime;
use chrono::Utc;
use std::{env, error::Error};
use zigistry::codeberg;
use zigistry::database;
use zigistry::dependents_calculator::calculate_dependents;
use zigistry::github::process_last_30_minutes;
use zigistry::sections::fetch_repos_for_sections;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = database::connect_to_database().await.unwrap();
    println!("Connected");
    loop {
        process_last_30_minutes(&pool, "zig".to_string(), false).await;
        println!("Zig completed");
        process_last_30_minutes(&pool, "zig-package".to_string(), true).await;
        println!("Zig-package completed");
        tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
    }
}
