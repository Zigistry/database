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

mod bzz_stuff;

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
        // println!("Processing User: {}", user_name.to_string());
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
        // println!("range: {}..{}", lower_range, upper_range);
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
                    // println!("{:#?}", err);
                }
            }
        }
        for repository in nodes {
            let mut repository_resultant = custom_types::Repo {
                description: repository.description,
                issues_count: repository.issues.total_count,
                default_branch: repository.default_branch_ref.name.to_string(),
                fork_count: repository.fork_count,
                stargazer_count: repository.stargazer_count,
                watchers_count: repository.watchers.total_count,
                pushed_at: repository.pushed_at,
                created_at: repository.created_at.to_string(),
                is_archived: repository.is_archived,
                is_disabled: repository.is_disabled,
                is_fork: repository.is_fork,
                license: match repository.license_info {
                    Some(l) => l.spdx_id,
                    None => String::new(),
                },
                repository_topics: repository
                    .repository_topics
                    .edges
                    .iter()
                    .map(|e| e.node.topic.name.clone())
                    .collect(),
                primary_language: match repository.primary_language {
                    Some(language) => language.name,
                    None => String::new(),
                },
                default_branch_information: custom_types::Release {
                    is_prerelease: false,
                    published_at: repository.created_at,
                    release_assets: HashMap::new(),
                    dependencies: Vec::new(),
                    minimum_zig_version: String::new(),
                    readme_url: match git_hub::get_readme_url(
                        &repository.owner.login,
                        repository.name.as_str(),
                        &repository.default_branch_ref.name,
                    )
                    .await
                    {
                        Some(url) => url,
                        None => String::new(),
                    },
                },
                releases: HashMap::new(),
            };
            let default_branch = repository.default_branch_ref.name;
            match git_hub::get_build_zig_zon_data(
                &repository.owner.login,
                repository.name.as_str(),
                &default_branch,
            )
            .await
            {
                Ok((_, dependencies)) => {
                    repository_resultant.default_branch_information.dependencies = dependencies;
                }
                Err(err) => {
                    // println!("{:#?}", err);
                }
            };
            for release in repository.releases.nodes {
                let tag_name = release.tag_name;
                let readme_url = git_hub::get_readme_url(
                    &repository.owner.login,
                    repository.name.as_str(),
                    &tag_name,
                )
                .await;
                let bzz_results = git_hub::get_build_zig_zon_data(
                    &repository.owner.login,
                    repository.name.as_str(),
                    &tag_name,
                )
                .await;

                repository_resultant.releases.insert(
                    tag_name,
                    custom_types::Release {
                        is_prerelease: release.is_prerelease,
                        published_at: release.published_at,
                        release_assets: release
                            .release_assets
                            .nodes
                            .iter()
                            .map(|n| {
                                (
                                    n.name.clone(),
                                    custom_types::Asset {
                                        download_url: n.download_url.clone(),
                                        size: n.size,
                                        content_type: n.content_type.clone(),
                                    },
                                )
                            })
                            .collect(),
                        dependencies: match &bzz_results {
                            Ok((_, dependencies)) => dependencies.clone(),
                            Err(err) => {
                                // println!("{:#?}", err);
                                Vec::new()
                            }
                        },
                        minimum_zig_version: match bzz_results {
                            Ok((minimum_zig_version, _)) => minimum_zig_version,
                            Err(_) => {
                                // println!("{:#?}", err);
                                "unknown".to_string()
                            }
                        },
                        readme_url: match readme_url {
                            Some(url) => url,
                            None => String::new(),
                        },
                    },
                );
                if GLOBAL
                    .lock()
                    .await
                    .users
                    .contains_key(&repository.owner.login)
                {
                    continue;
                } else {
                    // println!("Processing User: {}", repository.owner.login);
                    let user_resultant = custom_types::User {
                        avatar_url: repository.owner.avatar_url.clone(),
                        bio: repository.owner.bio.clone(),
                        company: repository.owner.company.clone(),
                        followers: repository
                            .owner
                            .followers
                            .clone()
                            .unwrap_or_default()
                            .total_count,
                        following: repository
                            .owner
                            .following
                            .clone()
                            .unwrap_or_default()
                            .total_count,
                        location: repository.owner.location.clone(),
                        description: repository.owner.description.clone(),
                        website_url: repository.owner.website_url.clone(),
                    };
                    GLOBAL
                        .lock()
                        .await
                        .users
                        .insert(repository.owner.login.clone(), user_resultant);
                }
            }
            GLOBAL.lock().await.packages.insert(
                format!("gh/{}/{}", repository.owner.login, repository.name),
                repository_resultant,
            );
        }
        break;
    }
    // println!("{}", GLOBAL.lock().await.users.len());
    println!("{}", serde_json::to_string(&GLOBAL.lock().await.packages).unwrap());
    // println!("{}", GLOBAL.lock().await.programs.len());
    Ok(())
}
