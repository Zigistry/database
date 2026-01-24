use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::get;
use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use std::thread;
use zigistry::codeberg;
use zigistry::database;
use zigistry::github;

#[get("/")]
async fn index() -> impl Responder {
    "Status: Active"
}

fn main() -> Result<(), Box<dyn Error>> {
    let server_thingy = thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let my_server = HttpServer::new(|| App::new().service(index))
                .bind(("0.0.0.0", 7860))
                .unwrap();
            println!("Server at: http://0.0.0.0:7860");
            my_server.run().await.unwrap();
        });
    });

    let cron_thingy = thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let pool = Arc::new(database::connect_to_database().await.unwrap());
            println!("Connected");
            // I am doing  - chrono::Duration::minutes(15) to make sure
            // If the api takes some time to update, that is still covered.
            let mut last_time_stamp = Utc::now().naive_utc() - chrono::Duration::minutes(30);
            loop {
                let current_time = Utc::now().naive_utc() - chrono::Duration::minutes(15);
                eprintln!("Starting cron job iteration at {}", current_time);
                codeberg::process_last_15_minutes("zig", last_time_stamp, Arc::clone(&pool)).await;
                codeberg::process_last_15_minutes(
                    "zig-package",
                    last_time_stamp,
                    Arc::clone(&pool),
                )
                .await;
                github::process_last_15_minutes(
                    Arc::clone(&pool),
                    "zig".to_string(),
                    false,
                    last_time_stamp,
                )
                .await;
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
                println!(
                    "This thing got completed within: {}",
                    Utc::now().naive_utc() - current_time
                );
                last_time_stamp = current_time;

                tokio::time::sleep(std::time::Duration::from_secs(900)).await;
            }
        });
    });

    server_thingy.join().unwrap();
    cron_thingy.join().unwrap();

    Ok(())
}
