use std::error::Error;
use zigistry::database;
use zigistry::github::process_last_30_minutes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = database::connect_to_database().await.unwrap();
    println!("Connected");
    loop {
        eprintln!("Continuing cron job iteration");
        process_last_30_minutes(&pool, "zig".to_string(), false).await;
        eprintln!("Zig completed");
        process_last_30_minutes(&pool, "zig-package".to_string(), true).await;
        eprintln!("Zig-package completed");
        eprintln!("Entering hault for 9000 seconds");
        tokio::time::sleep(std::time::Duration::from_secs(9000)).await;
    }
}
