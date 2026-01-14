pub mod types;
use crate::bzz_stuff::{parse, tokenize};
use crate::constants::{GH_GRAPH_QL_QUERY, POSSIBLE_README_FILE_NAMES};
use crate::{GITHUB_KEY, custom_types, db};
use chrono::{Months, NaiveDate, Utc};
use futures::stream;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
const EMPTY_REPLY: &str =
    r#"{"data":{"search":{"pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[]}}}"#;

pub async fn process_repository(repository: &types::Node, is_package: bool) {
    let user_name = format!("gh/{}", repository.owner.login).to_lowercase();
    if !db!().users.contains_key(&user_name) {
        // println!("Processing User: {}", repository.owner.login);
        let user_resultant = custom_types::User {
            avatar_id: repository.owner.login.clone(),
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
        db!().users.insert(user_name, user_resultant);
    }
    // eprintln!("Processing Repository: {}", repository.name);
    let mut repository_resultant = custom_types::Repo {
        avatar_id: repository.owner.login.clone(),
        dependents: vec![],
        description: repository.description.clone(),
        issues_count: repository.issues.total_count,
        default_branch: repository.default_branch_ref.name.to_string(),
        fork_count: repository.fork_count,
        stargazer_count: repository.stargazer_count,
        watchers_count: repository.watchers.total_count,
        pushed_at: repository.pushed_at.clone(),
        created_at: repository.created_at.to_string(),
        is_archived: repository.is_archived,
        is_disabled: repository.is_disabled,
        is_fork: repository.is_fork,
        license: repository.license_info.clone().unwrap_or_default().spdx_id,
        repository_topics: repository
            .repository_topics
            .edges
            .iter()
            .map(|e| e.node.topic.name.clone())
            .collect(),
        primary_language: repository.primary_language.clone().unwrap_or_default().name,
        default_branch_information: custom_types::Release {
            is_prerelease: false,
            published_at: repository.created_at.clone(),
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
                _ => "404 unable to find readme.".to_string(),
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
    for release in &repository.releases.nodes {
        let readme_url =
            get_readme_url(&repository.owner.login, &repository.name, &release.tag_name).await;
        let bzz_results =
            get_build_zig_zon_data(&repository.owner.login, &repository.name, &release.tag_name)
                .await;

        repository_resultant.releases.insert(
            release.tag_name.clone(),
            custom_types::Release {
                is_prerelease: release.is_prerelease,
                published_at: release.published_at.clone(),
                dependencies: match &bzz_results {
                    Ok((_, dependencies)) => dependencies.to_vec(),
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
                    _ => String::new(),
                },
            },
        );
    }
    if is_package {
        db!().packages.insert(
            format!("gh/{}/{}", repository.owner.login, repository.name).to_lowercase(),
            repository_resultant,
        );
    } else {
        db!().programs.insert(
            format!("gh/{}/{}", repository.owner.login, repository.name).to_lowercase(),
            repository_resultant,
        );
    }
}

pub async fn process_query(
    query: &str,
    is_package: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Yes, 8th Feb 2016, Zig release date.
    let start = NaiveDate::from_ymd_opt(2016, 2, 8).unwrap();
    let end = Utc::now().date_naive();
    let mut lower = start;
    let mut upper = start.checked_add_months(Months::new(6)).unwrap();
    let client = reqwest::Client::new();
    let mut nodes = Vec::new();
    loop {
        eprintln!("Now processing:{lower}..{upper}");
        let mut has_next = true;
        let mut next: Option<String> = None;
        let mut asd = 1;
        while has_next {
            eprintln!("HAS NEXT! {asd}");
            asd += 1;
            let query_to_send = serde_json::json!({
                "query": GH_GRAPH_QL_QUERY,
                "variables": {
                    "query": format!("topic:{query} created:{lower}..{upper}"),
                    "next_value": next
                }
            });
            let res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", GITHUB_KEY.to_string())
                .header("User-Agent", "zigistry.dev")
                .json(&query_to_send)
                .send()
                .await?;

            let text = res.text().await?;
            if text == EMPTY_REPLY {
                has_next = false;
                continue;
            }
            let mut res2: types::Root = match serde_json::from_str(&text) {
                Ok(t) => t,
                Err(t) => {
                    eprintln!("Got this responce:");
                    eprintln!("{text}");
                    panic!("Got this problem: {t}");
                }
            };
            eprintln!("{:#?}", res2.data.search.page_info.has_next_page);
            has_next = res2.data.search.page_info.has_next_page;
            next = Option::from(res2.data.search.page_info.end_cursor);
            nodes.append(&mut res2.data.search.nodes);
        }

        stream::iter(&nodes)
            .for_each_concurrent(100, |node| async move {
                process_repository(&node, is_package).await;
            })
            .await;
        lower = upper;
        upper = lower.checked_add_months(Months::new(6)).unwrap();
        if lower > end {
            break;
        }
    }
    return Ok(());
}

pub async fn github_main() -> Result<(), Box<dyn Error>> {
    process_query("zig-package", true).await.unwrap();
    process_query("zig", false).await.unwrap();
    Ok(())
}

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
) -> Result<(String, Vec<custom_types::Dependency>), Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/build.zig.zon"
    );
    let client = reqwest::Client::new();
    let text = client.get(&url).send().await?.text().await?;

    let tokens = tokenize(text.chars())?;
    let parsed = parse(tokens.into_iter())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}

// I am doing this because I don't want so much messy
// code in my github_main
async fn get_build_zig_zon_data_wrapper(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
) -> (String, Vec<custom_types::Dependency>) {
    match get_build_zig_zon_data(owner_name, repo_name, branch_or_tag).await {
        Ok((minimum_zig_version, dependencies)) => (minimum_zig_version, dependencies),
        Err(_) => {
            eprintln!(
                "Parser wasn't able to parse: https://github.com/{}/{}",
                owner_name, repo_name
            );
            ("unknown".to_string(), Vec::new())
        }
    }
}
