mod codeberg_process_release;
mod helper_functions;
pub mod types;

use chrono::{DateTime, Utc};

use std::str::FromStr;

use crate::codeberg::helper_functions::get_readme_url;
use crate::codeberg::types::types::Daum;
use crate::constants::ASYNC_LIMIT;
use crate::{CODEBERG_KEY, codeberg::helper_functions::get_build_zig_zon_data};
use chrono::{Days, Months, NaiveDateTime};
use codeberg_process_release::process_release;
use futures::{stream, stream::StreamExt};
use libsql::{Connection, params};

pub async fn process_last_15_minutes(query: &str, time_15_minutes_ago: NaiveDateTime) {
    let url = format!(
        "https://codeberg.org/api/v1/repos/search?q={query}&sort=updated&order=desc&limit=100&page=1"
    );
    let client = reqwest::Client::new();
    let responce = client
        .get(&url)
        .header("Authorization", &*CODEBERG_KEY)
        .send()
        .await
        .unwrap()
        .json::<types::types::Root>()
        .await
        .unwrap();

    let mut repos_to_actually_process = Vec::new();
    for repository in responce.data {
        if DateTime::parse_from_rfc3339(&repository.updated_at)
        .unwrap()
        .with_timezone(&Utc)
        .naive_utc() > time_15_minutes_ago {
            repos_to_actually_process.push(repository);
        } else {
            break;
        }
    }

    println!("{:?}", repos_to_actually_process);
}

pub async fn process_repo(repository: Daum, pool: &Connection) {
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
                            INSERT OR IGNORE INTO users
                                (id, platform, avatar_id, bio)
                            VALUES (?, ?, ?, ?)
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
                            INSERT OR IGNORE INTO repos
                                (id, avatar_id, owner, platform, description, issues_count, default_branch_name, fork_count
                                , stargazer_count, watchers_count, pushed_at, created_at, is_archived, is_disabled,
                                is_fork, license, primary_language, search_keywords)
                            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                        "#,
                        params![
                            repo_id.clone(),
                            repository
                                .owner
                                .avatar_url
                                .rsplit("/")
                                .next()
                                .unwrap()
                                .to_string(),
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
                            "Not Found", // Only for now
                            repository.language.clone(),
                            keywords
                        ]
                    )
                    .await
                    .unwrap();

    let rows_affected = transaction.execute(
                r#"
                    INSERT OR IGNORE INTO releases
                        (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
                    VALUES(?, ?, ?, ?, ?, ?)
                "#,
                params![
                    repo_id.clone(),
                    "__ZIGISTRY__DEFAULT__BRANCH__",
                    false,
                    repository.created_at.clone(),
                    build_zig_zon_data.0.clone(),
                    readme_url
                ],
            ).await.unwrap();

    let default_branch_release_id = if rows_affected > 0 {
        Some(transaction.last_insert_rowid())
    } else {
        None
    };

    match default_branch_release_id {
        Some(default_branch_release_id) => {
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
        }
        None => {
            println!("Got None for: {}", &repo_id);
        }
    }
    process_release(&repository.owner.login, &repository.name, &repo_id, &pool).await;
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
    pool: &Connection,
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

            let responce = client
                .get(&url)
                .header("Authorization", &*CODEBERG_KEY)
                .send()
                .await?
                .json::<types::types::Root>()
                .await?;

            if responce.data.is_empty() {
                break;
            }

            stream::iter(responce.data)
                .for_each_concurrent(ASYNC_LIMIT, |repository| async move {
                    process_repo(repository, pool).await;
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
    pool: &Connection,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_codeberg_repos(pool, "zig-package", start_date, end_date, step)
        .await
        .unwrap();
    fetch_all_codeberg_repos(pool, "zig", start_date, end_date, step)
        .await
        .unwrap();
    Ok(())
}
