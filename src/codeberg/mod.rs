mod codeberg_process_release;
pub mod types;

use crate::{CODEBERG_KEY, DATABASE, custom_types};
use futures::future;
use std::collections::HashMap;
use std::error::Error;

pub async fn fetch_all_codeberg_repos(query: &str) -> Result<(), Box<dyn Error>> {
    let mut all_repos = vec![];
    let mut page = 1;

    let client = reqwest::Client::new();

    loop {
        let url =
            format!("https://codeberg.org/api/v1/repos/search?q={query}&page={page}&topic=true");

        eprintln!("Processing: {}", url);
        let res = client
            .get(&url)
            // .header("Authorization", &**CODEBERG_KEY)
            .send()
            .await?
            .text()
            .await?;
        let responce = serde_json::from_str::<types::types::Root>(&res)?;
        if responce.data.is_empty() {
            break;
        }
        for repo in responce.data {
            all_repos.push(repo);
        }
        page += 1;
    }

    eprintln!("All repos len: {}", all_repos.len());

    let mut futures_all = vec![];

    for repo in all_repos {
        futures_all.push(async move {
            let user_name = format!("cb/{}", repo.owner.login);
            if !(DATABASE.lock().await.users.contains_key(&user_name)) {
                let user = custom_types::User {
                    avatar_url: repo.owner.avatar_url,
                    company: Some(String::new()),
                    followers: repo.owner.followers_count,
                    following: repo.owner.following_count,
                    location: Some(repo.owner.location),
                    description: Some(repo.owner.description.clone()),
                    bio: Some(repo.owner.description),
                    website_url: Some(repo.owner.website),
                };
                DATABASE.lock().await.users.insert(user_name.clone(), user);
            }

            let releases = codeberg_process_release::process_release(repo.owner.login, repo.name)
                .await
                .unwrap_or_default();

            let repo_resultant = custom_types::Repo {
                created_at: repo.created_at.clone(),
                description: repo.description.into(),
                issues_count: repo.open_issues_count,
                default_branch: repo.default_branch,
                fork_count: repo.forks_count,
                stargazer_count: repo.stars_count,
                watchers_count: repo.watchers_count,
                pushed_at: repo.updated_at,
                is_archived: repo.archived,
                is_disabled: repo.archived,
                is_fork: repo.fork,
                license: String::from("Not found"),
                repository_topics: repo.topics,
                primary_language: repo.language,
                default_branch_information: custom_types::Release {
                    is_prerelease: false,
                    published_at: repo.created_at,
                    release_assets: HashMap::new(),
                    dependencies: vec![],
                    minimum_zig_version: String::new(),
                    readme_url: String::new(),
                },
                releases: releases,
            };
            // https://codeberg.org/:owner/:repo/releases.rss
            if repo_resultant
                .repository_topics
                .contains(&"zig-package".to_string())
            {
                DATABASE
                    .lock()
                    .await
                    .packages
                    .insert(user_name.clone(), repo_resultant);
            } else {
                DATABASE
                    .lock()
                    .await
                    .programs
                    .insert(user_name.clone(), repo_resultant);
            }
        });
    }

    futures_all.chunks().map(|chunk| async {
        future::join_all(chunk).await;
    })   
    
    Ok(())
}

pub async fn compute_default_branch_information() -> Result<(), Box<dyn Error>> {
    
    Ok(())
}

pub async fn codeberg_main() -> Result<(), Box<dyn Error>> {
    fetch_all_codeberg_repos("zig-package").await.unwrap();
    fetch_all_codeberg_repos("zig").await.unwrap();
    eprintln!("{}", &DATABASE.lock().await.users.len());
    Ok(())
}
