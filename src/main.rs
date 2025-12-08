mod config;
use chrono::{Days, Local, Months};
mod types;
use futures::future::join_all;
use std::{collections::HashMap, env};
use tokio::sync::Mutex;
mod custom_types;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;

type GenericErr = Box<dyn std::error::Error>;

lazy_static! {
    static ref KEY: String = "Bearer ".to_string()
        + &env::var("GH_API_KEY")
            .expect("GH_API_KEY not set")
            .to_string();
}

fn date_appender(page: i32) -> String {
    // YYYY-MM-DD
    // "2019-01-01"
    // This function will have a greater exception to no
    // more than 1000 packages in 6 months.
    let effective_months = page * 6;
    let yyyy = 2019;
    let mm = 01;
    let total_months = effective_months;
    let mut year = yyyy + (total_months / 12);
    let mut month = mm + (total_months % 12);
    if month > 12 {
        year += 1;
        month -= 12;
    }
    format!("{:04}-{:02}-01", year, month)
}

static GLOBAL: Lazy<Mutex<custom_types::Root>> = Lazy::new(|| {
    Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    })
});

// async fn process_github_user(user: String) -> Result<(), GenericErr> {
//     let fetch_user_url = format!("https://api.github.com/users/{}", user);
//     let client = reqwest::Client::new();
//     let json = client
//         .get(&fetch_user_url)
//         .header("Authorization", KEY.to_string())
//         .header("User-Agent", "zigistry.dev")
//         .send()
//         .await?
//         .json::<types::User>()
//         .await;

//     match json {
//         Ok(json) => {
//             let user_type_as_custom_user_type = custom_types::User {
//                 avatar_url: json.avatar_url,
//                 profile_link: json.html_url,
//                 type_field: json.type_field,
//                 followers: json.followers,
//                 following: json.following,
//                 email: json.email,
//                 description: json.bio,
//                 location: json.location,
//                 company: json.company,
//             };
//             GLOBAL
//                 .lock()
//                 .await
//                 .users
//                 .insert(user, user_type_as_custom_user_type);
//         }
//         Err(e) => {
//             println!("{}", e);
//         }
//     }
//     Ok(())
// }

// async fn process_github_repository(repository: types::Item) -> Result<(), GenericErr> {
//     let user_name = repository.full_name.split("/").next().unwrap();
//     let already_has_user = {
//         GLOBAL
//             .lock()
//             .await
//             .users
//             .contains_key(&user_name.to_string())
//     };
//     if !already_has_user {
//         println!("Processing User: {}", user_name.to_string());
//         process_github_user(user_name.to_string()).await?;
//     }
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), GenericErr> {
    let mut nodes = Vec::new();
    let now = Local::now();
    let date = now.date_naive();
    let current_date_plus_one_day = date.checked_add_days(Days::new(1)).unwrap();
    let mut start_date = chrono::NaiveDate::from_ymd_opt(2022, 2, 8).unwrap();
    while (start_date < current_date_plus_one_day) {
        let lower_range = start_date.to_string();
        start_date = start_date.checked_add_months(Months::new(6)).unwrap();
        let upper_range = start_date.to_string();
        println!("range: {}..{}", lower_range, upper_range);
        let client = reqwest::Client::new();
        let mut has_next_page = true;
        let mut next_value:String = String::from("");
        while has_next_page {
            let query_to_send = serde_json::json!({
                "query": include_str!("../gqlFiles/main.gql"),
                "variables": {
                    "query": format!("topic:zig-package created:{}..{}", lower_range, upper_range),
                    "next_value": next_value
                }
            });
            let res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", KEY.to_string())
                .header("User-Agent", "zigistry.dev")
                .json(&query_to_send)
                .send()
                .await?
                .json::<types::Root>()
                // .text()
                .await;
            match res {
                Ok(mut res) => {
                    has_next_page = res.data.search.page_info.has_next_page;
                    next_value = res.data.search.page_info.end_cursor;
                    nodes.append(&mut res.data.search.nodes);
                }
                Err(err) => {
                    println!("{:#?}", err);
                }
            }
        }
        break;
    }
    Ok(())
}
