use actix_web::{App, HttpServer, Responder, get, web};
use chrono::{NaiveDateTime, Utc};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use zigistry::{database, github};

#[get("/")]
async fn index(last_updated: web::Data<Arc<RwLock<NaiveDateTime>>>) -> impl Responder {
    format!(
        r#"[cron_job]
status = "Active"
last_updated = "{}"
current_utc_time = "{}""#,
        last_updated.read().await,
        Utc::now().naive_utc()
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await.unwrap());

    let last_time_stamp = Arc::new(RwLock::new(Utc::now().naive_utc()));
    let last_time_stamp_clone = Arc::clone(&last_time_stamp);

    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    tokio::spawn(async move {
        loop {
            let timer_start = Utc::now();

            github::github_main(
                Arc::clone(&pool),
                Utc::now().naive_utc() - chrono::Duration::minutes(15),
                Utc::now().naive_utc(),
                200,
                "stars:10..20",
            )
            .await
            .unwrap();

            github::github_main(
                Arc::clone(&pool),
                NaiveDateTime::parse_from_str("2016-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                    .unwrap(),
                Utc::now().naive_utc(),
                200,
                "stars:10..20",
            )
            .await
            .unwrap();

            let current_time = Utc::now().naive_utc();
            *last_time_stamp_clone.write().await = current_time;

            eprintln!(
                "github completed successfully in {} minutes.",
                (Utc::now() - timer_start).num_minutes(),
            );

            tokio::time::sleep(std::time::Duration::from_hours(24)).await;
        }
    });

    let my_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&last_time_stamp)))
            .service(index)
    })
    .bind(("0.0.0.0", 7860))
    .unwrap();

    println!("Server at: http://0.0.0.0:7860");
    my_server.run().await.unwrap();
    Ok(())
}
