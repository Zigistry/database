use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use zigistry::database;
use zigistry::github;
use zigistry::codeberg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await.unwrap());
    println!("Connected");
    // I am doing  - chrono::Duration::minutes(15) to make sure
    // If the api takes some time to update, that is still covered.
    let mut last_time_stamp = Utc::now().naive_utc() - chrono::Duration::minutes(30);
    codeberg::process_last_15_minutes("zig", last_time_stamp, pool).await;
    codeberg::process_last_15_minutes("zig-package", last_time_stamp, pool).await;
    loop {
        let current_time = Utc::now().naive_utc() - chrono::Duration::minutes(15);
        eprintln!("Starting cron job iteration at {}", current_time);

        github::process_last_15_minutes(Arc::clone(&pool), "zig".to_string(), false, last_time_stamp).await;
        eprintln!("Zig completed");
        github::process_last_15_minutes(
            Arc::clone(&pool),
            "zig-package".to_string(),
            true,
            last_time_stamp,
        )
        .await;
        eprintln!("Zig-package completed");
        eprintln!("Entering halt for 900 seconds");
        // Now I also need to update https://codeberg.org/api/v1/repos/search?q=zig-package&sort=updated&order=desc&limit=50&page=1
        last_time_stamp = current_time;
        tokio::time::sleep(std::time::Duration::from_secs(900)).await;
    }
}
