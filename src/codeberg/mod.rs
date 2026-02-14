mod codeberg_data;
mod codeberg_process_release;
mod helper_functions;
pub mod types;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::codeberg::codeberg_data::RepoData;
use crate::codeberg::helper_functions::get_build_zig_zon_data;
use crate::codeberg::types::types::Daum;
use crate::constants::ASYNC_LIMIT;
use crate::{CODEBERG_KEY, codeberg::helper_functions::get_readme_url};
use chrono::NaiveDateTime;
use codeberg_process_release::fetch_releases;
use futures::{stream, stream::StreamExt};
use libsql::{Connection, Transaction, params};

pub async fn get_repo_data(repository: Daum) -> RepoData {
    let user_id = format!("cb/{}", repository.owner.login).to_lowercase();
    let repo_id = format!("cb/{}/{}", repository.owner.login, repository.name).to_lowercase();
    let (readme_url, keywords) = get_readme_url(
        &repository.owner.login,
        repository.name.as_str(),
        &repository.default_branch,
        false,
        true,
    )
    .await;

    let build_zig_zon_data = match get_build_zig_zon_data(
        &repository.owner.login,
        &repository.name,
        "HEAD",
        false,
    )
    .await
    {
        Ok(t) => t,
        Err(_) => (String::new(), Vec::new()),
    };

    let releases = fetch_releases(&repository.owner.login, &repository.name).await;

    RepoData {
        repository,
        user_id,
        repo_id,
        readme_url,
        readme_keywords: keywords,
        build_zig_zon_version: build_zig_zon_data.0,
        build_zig_zon_dependencies: build_zig_zon_data.1,
        releases,
    }
}

pub async fn send_repo_data_to_database(transaction: &Transaction, data: RepoData) {
    let repository = data.repository;

    transaction
        .execute(
            r#"
                        INSERT INTO users
                            (id, platform, avatar_id, bio)
                        VALUES (?, ?, ?, ?)
                        ON CONFLICT(id) DO UPDATE SET
                            platform = excluded.platform,
                            avatar_id = excluded.avatar_id,
                            bio = excluded.bio
                "#,
            params![
                data.user_id.clone(),
                "codeberg",
                // I am using owner login name
                // for the avatar id because
                // it works and uses very low storage
                // as compaired to storing the entire
                // avatar url.
                repository
                    .owner
                    .avatar_url
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .to_string(),
                repository.owner.description.clone()
            ],
        )
        .await
        .unwrap();

    transaction.execute(
        r#"
            INSERT INTO repos (id, avatar_id, owner, platform, description, issues_count,
                default_branch_name, fork_count, stargazer_count, watchers_count, pushed_at, created_at,
                is_archived, is_disabled, is_fork, minimum_zig_version, license, primary_language)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                avatar_id = excluded.avatar_id,
                owner = excluded.owner,
                platform = excluded.platform,
                description = excluded.description,
                issues_count = excluded.issues_count,
                default_branch_name = excluded.default_branch_name,
                fork_count = excluded.fork_count,
                stargazer_count = excluded.stargazer_count,
                watchers_count = excluded.watchers_count,
                pushed_at = excluded.pushed_at,
                created_at = excluded.created_at,
                is_archived = excluded.is_archived,
                is_disabled = excluded.is_disabled,
                is_fork = excluded.is_fork,
                minimum_zig_version = excluded.minimum_zig_version,
                license = excluded.license,
                primary_language = excluded.primary_language;
        "#,
        params![
            data.repo_id.clone(),
            repository.owner.avatar_url.rsplit("/").next().unwrap().to_string(),
            data.user_id,
            "codeberg",
            repository.description.to_string(),
            repository.open_issues_count,
            repository.default_branch.clone(),
            repository.forks_count,
            repository.stars_count,
            repository.watchers_count,
            repository.updated_at.clone(),
            repository.created_at.clone(),
            repository.archived,
            repository.archived,
            repository.fork,
            data.build_zig_zon_version.clone(),
            "-",
            repository.language.clone(),
        ]
    )
    .await
    .unwrap();

    transaction
        .execute(
            r#"
                INSERT OR REPLACE INTO repo_search (repo_id, keywords) VALUES (?, ?)
            "#,
            params![data.repo_id.clone(), data.readme_keywords],
        )
        .await
        .unwrap();

    transaction
        .execute(
            r#"
                INSERT OR REPLACE INTO repo_topics (repo_id, topic) VALUES (?, ?)
            "#,
            params![data.repo_id.clone(), repository.topics.join(",").clone()],
        )
        .await
        .unwrap();

    let mut rows = transaction
        .query(
            r#"
                INSERT INTO releases
                    (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
                VALUES(?, ?, ?, ?, ?, ?)
                ON CONFLICT(repo_id, version) DO UPDATE SET
                    is_prerelease = excluded.is_prerelease,
                    published_at = excluded.published_at,
                    minimum_zig_version = excluded.minimum_zig_version,
                    readme_url = excluded.readme_url
                RETURNING id
            "#,
            params![
                data.repo_id.clone(),
                "__ZIGISTRY__DEFAULT__BRANCH__",
                false,
                repository.created_at.clone(),
                data.build_zig_zon_version.clone(),
                data.readme_url
            ],
        )
        .await
        .unwrap();

    let default_branch_release_id: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();

    transaction
        .execute(
            "DELETE FROM release_dependencies WHERE release_id = ?",
            params![default_branch_release_id],
        )
        .await
        .unwrap();

    for dependency in data.build_zig_zon_dependencies {
        transaction
            .execute(
                r#"
                    INSERT INTO release_dependencies
                        (release_id, name, hash, lazy, url, path)
                    VALUES(?, ?, ?, ?, ?, ?)
                "#,
                params![
                    default_branch_release_id,
                    dependency.name,
                    dependency.hash,
                    dependency.lazy,
                    dependency.url,
                    dependency.path
                ],
            )
            .await
            .unwrap();
    }

    // Perist releases
    for r in data.releases {
        let mut rows = transaction
            .query(
                r#"
                INSERT INTO releases
                    (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
                VALUES(?, ?, ?, ?, ?, ?)
                ON CONFLICT(repo_id, version) DO UPDATE SET
                    is_prerelease = excluded.is_prerelease,
                    published_at = excluded.published_at,
                    minimum_zig_version = excluded.minimum_zig_version,
                    readme_url = excluded.readme_url
                RETURNING id
            "#,
                params![
                    data.repo_id.clone(),
                    r.tag_name,
                    r.is_prerelease,
                    r.published_at,
                    r.minimum_zig_version,
                    r.readme_url,
                ],
            )
            .await
            .unwrap();

        let this_specific_release_id: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();

        transaction
            .execute(
                "DELETE FROM release_dependencies WHERE release_id = ?",
                params![this_specific_release_id],
            )
            .await
            .unwrap();

        for dependency in r.dependencies {
            transaction
                .execute(
                    r#"
                        INSERT INTO release_dependencies
                            (release_id, name, hash, lazy, url, path)
                        VALUES(?, ?, ?, ?, ?, ?)
                    "#,
                    params![
                        this_specific_release_id,
                        dependency.name,
                        dependency.hash,
                        dependency.lazy,
                        dependency.url,
                        dependency.path,
                    ],
                )
                .await
                .unwrap();
        }
    }

    if repository.topics.contains(&"zig-package".to_string()) {
        transaction
            .execute(
                r#"
                    INSERT OR IGNORE INTO packages
                        (repo_id)
                    VALUES(?)
                "#,
                params![data.repo_id.clone()],
            )
            .await
            .unwrap();
    } else {
        transaction
            .execute(
                r#"
                    INSERT OR IGNORE INTO programs
                        (repo_id)
                    VALUES(?)
                "#,
                params![data.repo_id.clone()],
            )
            .await
            .unwrap();
    }
}

pub async fn fetch_all_codeberg_repos(
    pool: Arc<Connection>,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut page = 1;
    let client = reqwest::Client::new();
    loop {
        let url = format!(
            "https://codeberg.org/api/v1/repos/search?q={query}&limit=100&page={page}&topic=true",
        );

        eprintln!("Processing: {}", url);

        let mut responce = Option::None;
        for _ in 0..5 {
            match client
                .get(&url)
                .header("Authorization", &*CODEBERG_KEY)
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        eprintln!("Codeberg status: {}", resp.status());
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }

                    match resp.text().await {
                        Ok(body) => match serde_json::from_str::<types::types::Root>(&body) {
                            Ok(val) => {
                                responce = Some(val);
                                break;
                            }
                            Err(e) => {
                                let snippet: String = body.chars().take(300).collect();
                                eprintln!("Failed to parse JSON: {}", e);
                                eprintln!("Codeberg body (truncated): {}", snippet);
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read response body: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to send request: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }

        let responce = responce.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch data from Codeberg after 5 retries",
            )
        })?;

        if responce.data.is_empty() {
            break;
        }

        let transaction = pool.transaction().await.unwrap();
        stream::iter(responce.data)
            .map(|repository| get_repo_data(repository))
            .buffer_unordered(ASYNC_LIMIT)
            .for_each(|data| async {
                send_repo_data_to_database(&transaction, data).await;
            })
            .await;

        transaction.commit().await.unwrap();
        page += 1;
    }

    Ok(())
}

pub async fn codeberg_main(pool: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_codeberg_repos(pool.clone(), "zig-package")
        .await
        .unwrap();
    fetch_all_codeberg_repos(pool.clone(), "zig").await.unwrap();
    Ok(())
}
