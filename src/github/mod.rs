use std::collections::HashMap;
use chrono::{Days, Local, Months, NaiveDate};
use futures::future::join_all;
use crate::GLOBAL;
use crate::{custom_types, types};

async fn process_repository(repository: types::Node) {
    // eprintln!("Processing Repository: {}", repository.name);
    let mut repository_resultant = custom_types::Repo {
        description: repository.description.unwrap_or_default(),
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
        license: repository.license_info.unwrap_or_default().spdx_id,
        repository_topics: repository
            .repository_topics
            .edges
            .iter()
            .map(|e| e.node.topic.name.clone())
            .collect(),
        primary_language: repository.primary_language.unwrap_or_default().name,
        default_branch_information: custom_types::Release {
            is_prerelease: false,
            published_at: repository.created_at,
            release_assets: HashMap::new(),
            dependencies: Vec::new(),
            minimum_zig_version: String::new(),
            readme_url: match get_readme_url(
                &repository.owner.login,
                repository.name.as_str(),
                &repository.default_branch_ref.name,
            )
            .await
            {
                Some(url) => url,
                None => "404 unable to find readme.".to_string(),
            },
        },
        releases: HashMap::new(),
    };
    let data = get_build_zig_zon_data_wrapper(
        &repository.owner.login,
        repository.name.as_str(),
        &repository.default_branch_ref.name,
    )
    .await;
    repository_resultant
        .default_branch_information
        .minimum_zig_version = data.0;
    repository_resultant.default_branch_information.dependencies = data.1;
    for release in repository.releases.nodes {
        let tag_name = release.tag_name;
        let readme_url =
            get_readme_url(&repository.owner.login, repository.name.as_str(), &tag_name).await;
        let bzz_results =
            get_build_zig_zon_data(&repository.owner.login, repository.name.as_str(), &tag_name)
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
                    Err(_) => {
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

pub async fn github_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut start = NaiveDate::from_ymd_opt(2024, 2, 8).unwrap();
    // I am doing extra one day to accomedate for if all the universal time.
    let end = Local::now()
        .date_naive()
        .checked_add_days(Days::new(1))
        .unwrap();
    let client = reqwest::Client::new();
    let mut nodes = Vec::new();
    while start < end {
        // eprintln!("{start}..{end}");
        let lower = start.to_string();
        start = start.checked_add_months(Months::new(6)).unwrap();
        let upper = start.to_string();
        let mut has_next = true;
        let mut next: Option<String> = None;
        let mut asd = 1;
        while has_next {
            println!("HAS NEXT! {asd}");
            asd += 1;
            let query_to_send = serde_json::json!({
                "query": include_str!("../../gqlFiles/main.gql"),
                "variables": {
                    "query": format!("topic:zig-package created:{}..{}", lower, upper),
                    "next_value": next
                }
            });
            let mut res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", crate::KEY.to_string())
                .header("User-Agent", "zigistry.dev")
                .json(&query_to_send)
                .send()
                .await?;
                // .json::<types::Root>()
                // // .text()
                // .await?; // Errors are not allowed in this scenario, but, crashes are.

                let text = res.text().await?;
                println!("{text}");
                // println!("{}", text);
                let mut res2: types::Root = serde_json::from_str(&text)?;
            println!("{:#?}", res2.data.search.page_info.has_next_page);
            has_next = res2.data.search.page_info.has_next_page;
            next = Option::from(res2.data.search.page_info.end_cursor);
            nodes.append(&mut res2.data.search.nodes);
        }
        let futures = nodes.iter().map(|node| process_repository(node.clone()));
        join_all(futures).await;
        break;
    }
    return Ok(());
}

use crate::bzz_stuff::{parse, tokenize};
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::custom_types::Dependency;
use std::error::Error;

pub async fn get_readme_url(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
) -> Option<String> {
    let name =
        format!("https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/");

    let client = reqwest::Client::new();
    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let mine = name.to_string() + readme_file_name;
        let res = client.head(&mine).send().await.unwrap();
        if res.status().is_success() {
            return Option::from(mine);
        }
    }
    None
}

pub async fn get_build_zig_zon_data(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
) -> Result<(String, Vec<Dependency>), Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/build.zig.zon"
    );
    let client = reqwest::Client::new();
    let text = client.get(&url).send().await?.text().await?;

    let tokens = tokenize(&mut text.chars().collect::<Vec<_>>().into_iter().peekable())?;
    let parsed = parse(&mut tokens.into_iter().peekable())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}

// I am doing this because I don't want so much messy
// code in my github_main
async fn get_build_zig_zon_data_wrapper(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
) -> (String, Vec<Dependency>) {
    match get_build_zig_zon_data(owner_name, repo_name, branch_or_tag).await {
        Ok((minimum_zig_version, dependencies)) => (minimum_zig_version, dependencies),
        Err(_) => ("unknown".to_string(), Vec::new()),
    }
}
