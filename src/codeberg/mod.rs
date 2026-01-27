mod codeberg_process_release;
mod helper_functions;
pub mod types;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::codeberg::helper_functions::get_readme_url;
use crate::codeberg::types::types::Daum;
use crate::constants::ASYNC_LIMIT;
use crate::{CODEBERG_KEY, codeberg::helper_functions::get_build_zig_zon_data};
use chrono::{Days, Months, NaiveDateTime};
use codeberg_process_release::process_release;
use futures::{stream, stream::StreamExt};
use libsql::{Connection, params};

pub async fn process_last_15_minutes(
    query: &str,
    time_15_minutes_ago: NaiveDateTime,
    pool: Arc<Connection>,
) {
    let url = format!(
        "https://codeberg.org/api/v1/repos/search?q={query}&sort=updated&order=desc&limit=100&page=1"
    );
    let client = reqwest::Client::new();
    let mut responce = Option::None;
    for _ in 0..5 {
        match client
            .get(&url)
            .header("Authorization", &*CODEBERG_KEY)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<types::types::Root>().await {
                Ok(val) => {
                    responce = Some(val);
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            },
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    let responce = responce.expect("Codeberg failed after 5 retries :(");

    let mut repos_to_actually_process = Vec::new();
    for repository in &responce.data {
        if DateTime::parse_from_rfc3339(&repository.updated_at)
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc()
            > time_15_minutes_ago
        {
            repos_to_actually_process.push(repository);
        } else {
            break;
        }
    }
    println!(
        "GOING TO PROCESS: {} many repos",
        repos_to_actually_process.len()
    );
    stream::iter(repos_to_actually_process.clone())
        .for_each_concurrent(5, |repo| {
            let value = pool.clone();
            println!("Prociessing: {}", &repo.clone().name);
            async move {
                process_repo(repo.clone(), value).await;
            }
        })
        .await;
    println!("{:?}", repos_to_actually_process.clone());
}

pub async fn process_repo(repository: Daum, pool: Arc<Connection>) {
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
    let transaction = pool.transaction().await.unwrap();
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
                user_id.clone(),
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
            repo_id.clone(),
            repository.owner.avatar_url.rsplit("/").next().unwrap().to_string(),
            user_id,
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
            build_zig_zon_data.0.clone(),
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
            params![repo_id.clone(), keywords],
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
                repo_id.clone(),
                "__ZIGISTRY__DEFAULT__BRANCH__",
                false,
                repository.created_at.clone(),
                build_zig_zon_data.0.clone(),
                readme_url
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

    for dependency in build_zig_zon_data.1.clone() {
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
    process_release(
        &repository.owner.login,
        &repository.name,
        &repo_id,
        &transaction,
    )
    .await;
    if repository.topics.contains(&"zig-package".to_string()) {
        transaction
            .execute(
                r#"
                    INSERT OR IGNORE INTO packages
                        (repo_id)
                    VALUES(?)
                "#,
                params![repo_id.clone()],
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
                params![repo_id.clone()],
            )
            .await
            .unwrap();
    }
    transaction.commit().await.unwrap();
}

pub async fn fetch_all_codeberg_repos(
    pool: Arc<Connection>,
    query: &str,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut page = 1;
    let client = reqwest::Client::new();
    let start = start_date;
    let end = end_date;
    let mut lower = start;
    let mut upper = start.checked_add_days(Days::new(step)).unwrap();
    loop {
        loop {
            let url = format!(
                "https://codeberg.org/api/v1/repos/search?q={query}&limit=100&page={page}&topic=true&start_date={lower}&end_date={upper}",
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
                    Ok(resp) => match resp.json::<types::types::Root>().await {
                        Ok(val) => {
                            responce = Some(val);
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        }
                    },
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

            let pool = pool.clone();
            stream::iter(responce.data)
                .for_each_concurrent(ASYNC_LIMIT, move |repository| {
                    let pool = pool.clone();
                    async move {
                        process_repo(repository, pool).await;
                    }
                })
                .await;
            page += 1;
        }

        lower = upper;
        upper = lower.checked_add_months(Months::new(6)).unwrap();
        if lower > end {
            break;
        }
    }

    Ok(())
}

pub async fn codeberg_main(
    pool: Arc<Connection>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_codeberg_repos(pool.clone(), "zig-package", start_date, end_date, step)
        .await
        .unwrap();
    fetch_all_codeberg_repos(pool.clone(), "zig", start_date, end_date, step)
        .await
        .unwrap();
    Ok(())
}
