mod config;
mod types;
use futures::future::join_all;
use std::{collections::HashMap, env};
use tokio::sync::Mutex;
mod custom_types;
use once_cell::sync::Lazy;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEY: String = env::var("GH_API_KEY").expect("GH_API_KEY not set");
}

static GLOBAL: Lazy<Mutex<custom_types::Root>> = Lazy::new(|| {
    Mutex::new(custom_types::Root {
        users: HashMap::new(),
        repos: HashMap::new(),
    })
});

async fn process_github_repository(
    repository: types::Item,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_name = repository.full_name.rsplit("/").next().unwrap();
    let already_has_user = {
        GLOBAL
            .lock()
            .await
            .users
            .contains_key(&user_name.to_string())
    };
    if !already_has_user {
        owner
        let client = reqwest::Client::new();
        let package_url = package_url.to_string();
        let indivisual_auth_key = auth_key.to_string();

    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut futures = Vec::new();
    let auth_key = "Bearer ".to_string() + &KEY.to_string();

    for package_url in config::PACKAGES.iter() {
        let client = reqwest::Client::new();
        let package_url = package_url.to_string();
        let indivisual_auth_key = auth_key.to_string();
        futures.push(async move {
            client
                .get(&package_url)
                .header("Authorization", indivisual_auth_key)
                .header("User-Agent", "zigistry")
                .send()
                .await?
                .json::<types::Root>()
                .await
        });
    }

    let results = join_all(futures).await;
    for result in results {
        match result {
            Ok(resp) => {
                for repository in resp.items {
                    process_github_repository(repository).await?;
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
    Ok(())
}
