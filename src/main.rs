mod config;
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

static GLOBAL: Lazy<Mutex<custom_types::Root>> = Lazy::new(|| {
    Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    })
});

async fn process_github_user(user: String) -> Result<(), GenericErr> {
    let fetch_user_url = format!("https://api.github.com/users/{}", user);
    let client = reqwest::Client::new();
    let json = client
        .get(&fetch_user_url)
        .header("Authorization", KEY.to_string())
        .header("User-Agent", "zigistry")
        .send()
        .await?
        .json::<types::User>()
        .await;

    match json {
        Ok(json) => {
            let user_type_as_custom_user_type = custom_types::User {
                avatar_url: json.avatar_url,
                profile_link: json.html_url,
                type_field: json.type_field,
                followers: json.followers,
                following: json.following,
                email: json.email,
                description: json.bio,
                location: json.location,
                company: json.company,
            };
            GLOBAL
                .lock()
                .await
                .users
                .insert(user, user_type_as_custom_user_type);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    Ok(())
}

async fn process_github_repository(repository: types::Item) -> Result<(), GenericErr> {
    let user_name = repository.full_name.split("/").next().unwrap();
    let already_has_user = {
        GLOBAL
            .lock()
            .await
            .users
            .contains_key(&user_name.to_string())
    };
    if !already_has_user {
        println!("Processing User: {}", user_name.to_string());
        process_github_user(user_name.to_string()).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GenericErr> {
    let mut futures = Vec::new();

    for package_url in config::PACKAGES.iter() {
        let client = reqwest::Client::new();
        let package_url = package_url.to_string();
        futures.push(async move {
            client
                .get(&package_url)
                .header("Authorization", KEY.to_string())
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
