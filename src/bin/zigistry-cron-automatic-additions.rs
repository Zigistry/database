use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::get;
use actix_web::web;
use chrono::NaiveDateTime;
use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;
use zigistry::codeberg;
use zigistry::database;
use zigistry::github;

#[get("/")]
async fn index(last_updated: actix_web::web::Data<Arc<RwLock<NaiveDateTime>>>) -> impl Responder {
    format!(
        r#"[status]
value = "Active"

[last_updated]
value = "{}"

[current_utc_time]
value = "{}"
"#,
        last_updated.read().unwrap(),
        Utc::now().naive_utc()
    )
}

#[actix_web::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await.unwrap());

    let last_time_stamp = Arc::new(RwLock::new(
        Utc::now().naive_utc() - chrono::Duration::minutes(30),
    ));
    let last_time_stamp_clone = Arc::clone(&last_time_stamp);

    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    actix_web::rt::spawn(async move {
        // I am doing  - chrono::Duration::minutes(15) to make sure
        // If the api takes some time to update, that is still covered.
        loop {
            let current_time = Utc::now().naive_utc() - chrono::Duration::minutes(15);
            let last_ts = *last_time_stamp_clone.read().unwrap();

            codeberg::process_last_15_minutes("zig", last_ts, Arc::clone(&pool)).await;
            codeberg::process_last_15_minutes("zig-package", last_ts, Arc::clone(&pool)).await;
            github::process_last_15_minutes(Arc::clone(&pool), "zig".to_string(), false, last_ts)
                .await;
            github::process_last_15_minutes(
                Arc::clone(&pool),
                "zig-package".to_string(),
                true,
                last_ts,
            )
            .await;

            *last_time_stamp_clone.write().unwrap() = current_time;

            actix_web::rt::time::sleep(std::time::Duration::from_secs(900)).await;
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
