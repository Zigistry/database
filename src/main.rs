mod config;
use chrono::{Days, Local, Months};
mod types;
use futures::future::join_all;
use std::{collections::HashMap, env};
use tokio::sync::Mutex;
mod constants;
mod custom_types;
mod helper_functions;

use crate::helper_functions::*;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;

type GenericErr = Box<dyn std::error::Error>;

lazy_static! {
    static ref KEY: String = "Bearer ".to_string()
        + &env::var("GH_API_KEY")
            .expect("GH_API_KEY not set")
            .to_string();
}

static GLOBAL: Lazy<Mutex<custom_types::Root>> = Lazy::new(|| {
    Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    })
});

async fn process_github_repository(repository: types::Node) -> Result<(), GenericErr> {
    let user_name = repository.owner.login;
    let already_has_user = {
        GLOBAL
            .lock()
            .await
            .users
            .contains_key(&user_name.to_string())
    };
    if !already_has_user {
        println!("Processing User: {}", user_name.to_string());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GenericErr> {
    let now = Local::now();
    let date = now.date_naive();
    let current_date_plus_one_day = date.checked_add_days(Days::new(1)).unwrap();
    let mut start_date = chrono::NaiveDate::from_ymd_opt(2022, 2, 8).unwrap();
    while start_date < current_date_plus_one_day {
        let mut nodes = Vec::new();
        let lower_range = start_date.to_string();
        start_date = start_date.checked_add_months(Months::new(6)).unwrap();
        let upper_range = start_date.to_string();
        println!("range: {}..{}", lower_range, upper_range);
        let client = reqwest::Client::new();
        let mut has_next_page = true;
        let mut next_value: Option<String> = None;
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
                    next_value = Option::from(res.data.search.page_info.end_cursor);
                    nodes.append(&mut res.data.search.nodes);
                }
                Err(err) => {
                    println!("{:#?}", err);
                }
            }
        }
        for repository in nodes {
            let default_branch = repository.default_branch_ref.name;
            let repo_full_name = repository.owner.login + repository.name.as_str();
            let readme_url = git_hub::get_readme_url(&repo_full_name, &default_branch);
            let readme_url = git_hub::get_build_zig_zon_data(&repo_full_name, &default_branch);
        }
        break;
    }
    Ok(())
}
