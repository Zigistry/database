mod codeberg_process_release;
mod helper_functions;
pub mod types;

use crate::codeberg::helper_functions::get_readme_url;
use crate::constants::ASYNC_LIMIT;
use crate::github::get_build_zig_zon_data;
use crate::{custom_types, db};
use codeberg_process_release::process_release;
use futures::{stream, stream::StreamExt};
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

        let responce = client
            .get(&url)
            .send()
            .await?
            .json::<types::types::Root>()
            .await?;

        if responce.data.is_empty() {
            break;
        }

        for repo in responce.data {
            all_repos.push(repo);
        }
        page += 1;
    }

    eprintln!("All repos len: {}", all_repos.len());

    stream::iter(all_repos)
        .for_each_concurrent(ASYNC_LIMIT, |repo| async move {
            let user_name = format!("cb/{}", repo.owner.login);

            if !db!().users.contains_key(&user_name) {
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
                db!().users.insert(user_name.clone(), user);
            }

            let impdata = match get_build_zig_zon_data(&repo.owner.login, &repo.name, "HEAD").await
            {
                Ok(t) => t,
                Err(e) => (String::new(), Vec::new()),
            };
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
                    dependencies: impdata.1,
                    minimum_zig_version: impdata.0,
                    readme_url: get_readme_url(&repo.owner.login, &repo.name, "HEAD", false).await,
                },
                releases: process_release(&repo.owner.login, &repo.name)
                    .await
                    .unwrap_or_default(),
            };

            if repo_resultant
                .repository_topics
                .contains(&"zig-package".to_string())
            {
                db!().packages.insert("cb".to_string() + &repo.full_name, repo_resultant);
            } else {
                db!().programs.insert("cb/".to_string() + &repo.full_name, repo_resultant);
            }
        })
        .await;

    Ok(())
}

pub async fn compute_default_branch_information() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub async fn codeberg_main() -> Result<(), Box<dyn Error>> {
    fetch_all_codeberg_repos("zig-package").await.unwrap();
    fetch_all_codeberg_repos("zig").await.unwrap();
    Ok(())
}
