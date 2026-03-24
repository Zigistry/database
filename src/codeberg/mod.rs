mod codeberg_data;
mod codeberg_process_release;
mod helper_functions;
pub mod types;
use std::sync::Arc;

use crate::codeberg::codeberg_data::RepoData;
use crate::codeberg::helper_functions::{
    get_build_zig_zon_data, get_latest_commit_hash, has_zig_in_top_languages,
};
use crate::codeberg::types::types::Daum;
use crate::constants::ASYNC_LIMIT;
use crate::constants::limits;
use crate::database::{parse_lazy_flag, truncate_to_char_limit, utc_now_timestamp};
use crate::{CODEBERG_KEY, codeberg::helper_functions::get_readme_url};
use codeberg_process_release::fetch_releases;
use futures::{stream, stream::StreamExt};
use libsql::{Connection, Transaction, params};

pub async fn get_repo_data(repository: Daum) -> RepoData {
    let user_id = format!("cb/{}", repository.owner.login).to_lowercase();
    let repo_id = format!("cb/{}/{}", repository.owner.login, repository.name).to_lowercase();
    let latest_commit_hash =
        get_latest_commit_hash(&repository.owner.login, &repository.name).await;
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
        latest_commit_hash,
        readme_url,
        readme_keywords: keywords,
        build_zig_zon_version: build_zig_zon_data.0,
        build_zig_zon_dependencies: build_zig_zon_data.1,
        releases,
    }
}

pub async fn send_repo_data_to_database(transaction: &Transaction, data: RepoData) {
    let RepoData {
        repository,
        user_id,
        repo_id,
        latest_commit_hash,
        readme_url,
        readme_keywords,
        build_zig_zon_version,
        build_zig_zon_dependencies,
        releases,
    } = data;

    let repo_id = truncate_to_char_limit(&repo_id, limits::REPO_ID_MAX_LEN);
    let user_id = truncate_to_char_limit(&user_id, limits::USER_ID_MAX_LEN);
    let platform = truncate_to_char_limit("codeberg", limits::PLATFORM_MAX_LEN);
    let avatar_id = truncate_to_char_limit(
        repository
            .owner
            .avatar_url
            .rsplit('/')
            .next()
            .unwrap_or(repository.owner.login.as_str()),
        limits::USER_AVATAR_ID_MAX_LEN,
    );
    let owner_id = truncate_to_char_limit(&user_id, limits::REPO_OWNER_MAX_LEN);
    let user_bio = Some(truncate_to_char_limit(
        &repository.owner.description,
        limits::USER_BIO_MAX_LEN,
    ));
    let description = Some(truncate_to_char_limit(
        &repository.description,
        limits::REPO_DESCRIPTION_MAX_LEN,
    ));
    let default_branch_name = truncate_to_char_limit(
        &repository.default_branch,
        limits::REPO_DEFAULT_BRANCH_MAX_LEN,
    );
    let latest_commit_hash =
        truncate_to_char_limit(&latest_commit_hash, limits::REPO_COMMIT_HASH_MAX_LEN);
    let license = truncate_to_char_limit("-", limits::REPO_LICENSE_MAX_LEN);
    let primary_language =
        truncate_to_char_limit(&repository.language, limits::REPO_PRIMARY_LANGUAGE_MAX_LEN);
    let database_updated_at = utc_now_timestamp();
    let is_package = repository.topics.iter().any(|topic| topic == "zig-package");

    let (user_insert_result, repo_insert_result) = tokio::join!(
        transaction.execute(
            r#"
            INSERT INTO users
                (id, platform, avatar_id, bio)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                platform = excluded.platform,
                avatar_id = excluded.avatar_id,
                bio = excluded.bio
            "#,
            params![user_id.clone(), platform.clone(), avatar_id, user_bio],
        ),
        transaction.execute(
            r#"
            INSERT INTO repos
                (id, owner, platform, description, issues_count, default_branch_name, fork_count,
                 stargazer_count, watchers_count, pushed_at, created_at, is_archived, is_disabled,
                 is_fork, license, primary_language, latest_commit_hash, last_updated_in_this_database)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
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
                license = excluded.license,
                primary_language = excluded.primary_language,
                latest_commit_hash = excluded.latest_commit_hash,
                last_updated_in_this_database = excluded.last_updated_in_this_database
            "#,
            params![
                repo_id.clone(),
                owner_id,
                platform,
                description,
                repository.open_issues_count,
                default_branch_name,
                repository.forks_count,
                repository.stars_count,
                repository.watchers_count,
                repository.updated_at.clone(),
                repository.created_at.clone(),
                repository.archived,
                repository.archived,
                repository.fork,
                license,
                primary_language,
                latest_commit_hash,
                database_updated_at,
            ]
        ),
    );
    user_insert_result.unwrap();
    repo_insert_result.unwrap();
    transaction
        .execute(
            r#"DELETE FROM repo_search WHERE repo_id = ?"#,
            params![repo_id.clone()],
        )
        .await
        .unwrap();
    transaction
        .execute(
            r#"INSERT INTO repo_search (repo_id, keywords) VALUES (?, ?)"#,
            params![repo_id.clone(), readme_keywords],
        )
        .await
        .unwrap();

    transaction
        .execute(
            "DELETE FROM repo_topics WHERE repo_id = ?",
            params![repo_id.clone()],
        )
        .await
        .unwrap();

    let mut topics: Vec<String> = repository
        .topics
        .iter()
        .map(|topic| truncate_to_char_limit(topic, limits::TOPIC_MAX_LEN))
        .filter(|topic| !topic.is_empty())
        .collect();

    let mut seen = std::collections::HashSet::new();
    topics.retain(|topic| seen.insert(topic.clone()));

    for topic in topics {
        transaction
            .execute(
                r#"INSERT OR IGNORE INTO repo_topics (repo_id, topic) VALUES (?, ?)"#,
                params![repo_id.clone(), topic],
            )
            .await
            .unwrap();
    }

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
                truncate_to_char_limit(
                    "__ZIGISTRY__DEFAULT__BRANCH__",
                    limits::RELEASE_VERSION_MAX_LEN
                ),
                false,
                repository.created_at.clone(),
                truncate_to_char_limit(
                    &build_zig_zon_version,
                    limits::RELEASE_MIN_ZIG_VERSION_MAX_LEN
                ),
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

    for dependency in build_zig_zon_dependencies {
        transaction
            .execute(
                r#"
                    INSERT INTO release_dependencies
                        (release_id, name, hash, is_lazy, url, path)
                    VALUES(?, ?, ?, ?, ?, ?)
                "#,
                params![
                    default_branch_release_id,
                    truncate_to_char_limit(
                        &dependency.name,
                        limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                    ),
                    truncate_to_char_limit(
                        &dependency.hash,
                        limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                    ),
                    i64::from(parse_lazy_flag(&dependency.lazy)),
                    truncate_to_char_limit(
                        &dependency.url,
                        limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                    ),
                    truncate_to_char_limit(
                        &dependency.path,
                        limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                    ),
                ],
            )
            .await
            .unwrap();
    }

    // Perist releases
    for r in releases {
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
                    truncate_to_char_limit(&r.tag_name, limits::RELEASE_VERSION_MAX_LEN),
                    r.is_prerelease,
                    r.published_at,
                    truncate_to_char_limit(
                        &r.minimum_zig_version,
                        limits::RELEASE_MIN_ZIG_VERSION_MAX_LEN
                    ),
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
                            (release_id, name, hash, is_lazy, url, path)
                        VALUES(?, ?, ?, ?, ?, ?)
                    "#,
                    params![
                        this_specific_release_id,
                        truncate_to_char_limit(
                            &dependency.name,
                            limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                        ),
                        truncate_to_char_limit(
                            &dependency.hash,
                            limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                        ),
                        i64::from(parse_lazy_flag(&dependency.lazy)),
                        truncate_to_char_limit(
                            &dependency.url,
                            limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                        ),
                        truncate_to_char_limit(
                            &dependency.path,
                            limits::RELEASE_DEPENDENCY_FIELD_MAX_LEN
                        ),
                    ],
                )
                .await
                .unwrap();
        }
    }

    if is_package {
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
        for attempt_count in 0..5 {
            match client
                .get(&url)
                .header("Authorization", &*CODEBERG_KEY)
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        eprintln!("Codeberg status: {}", resp.status());
                        let wait_secs = if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            resp.headers()
                                .get("Retry-After")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(60)
                        } else {
                            2u64.pow(attempt_count)
                        };
                        eprintln!("Waiting {} seconds before retry...", wait_secs);
                        tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
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
                                tokio::time::sleep(std::time::Duration::from_secs(
                                    2u64.pow(attempt_count),
                                ))
                                .await;
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read response body: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(
                                2u64.pow(attempt_count),
                            ))
                            .await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to send request: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt_count)))
                        .await;
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
            .map(|repository| async move {
                if !has_zig_in_top_languages(&repository.owner.login, &repository.name).await {
                    return None;
                }
                Some(get_repo_data(repository).await)
            })
            .buffer_unordered(ASYNC_LIMIT)
            .for_each(|data| async {
                if let Some(data) = data {
                    send_repo_data_to_database(&transaction, data).await;
                }
            })
            .await;

        transaction.commit().await.unwrap();
        page += 1;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
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

pub async fn fetch_all_codeberg_repos_cron_updating_part(
    pool: Arc<Connection>,
    query: &str,
    is_package: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut page = 1;
    let client = reqwest::Client::new();
    loop {
        let url = format!(
            "https://codeberg.org/api/v1/repos/search?q={query}&limit=100&page={page}&topic=true",
        );

        eprintln!("Processing cron: {}", url);

        let mut responce = Option::None;
        for number_of_tries_done in 0..5 {
            match client
                .get(&url)
                .header("Authorization", &*crate::CODEBERG_KEY)
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        eprintln!("cb status: {}", resp.status());
                        let wait_secs = if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            resp.headers()
                                .get("Retry-After")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(60)
                        } else {
                            2u64.pow(number_of_tries_done)
                        };
                        eprintln!("Waiting {} seconds before trying again...", wait_secs);
                        tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
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
                                eprintln!("failed to parse json: {}", e);
                                eprintln!("cb body small: {}", snippet);
                                tokio::time::sleep(std::time::Duration::from_secs(
                                    2u64.pow(number_of_tries_done),
                                ))
                                .await;
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read response body: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(
                                2u64.pow(number_of_tries_done),
                            ))
                            .await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to send request: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(
                        2u64.pow(number_of_tries_done),
                    ))
                    .await;
                }
            }
        }

        let responce = responce.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch data from cb after 5 retries",
            )
        })?;

        if responce.data.is_empty() {
            break;
        }

        let repo_type = if is_package { "package" } else { "program" };
        let transaction = pool.transaction().await.unwrap();

        stream::iter(responce.data)
            .map(|repository| async move {
                if !crate::codeberg::helper_functions::has_zig_in_top_languages(&repository.owner.login, &repository.name).await {
                    return None;
                }

                let repo_id = format!("cb/{}/{}", repository.owner.login, repository.name).to_lowercase();
                let latest_commit_hash = crate::codeberg::helper_functions::get_latest_commit_hash(&repository.owner.login, &repository.name).await;
                Some((repository, repo_id, latest_commit_hash))
            })
            .buffer_unordered(crate::constants::ASYNC_LIMIT)
            .for_each(|data| async {
                if let Some((repository, repo_id, latest_commit_hash)) = data {
                    let mut existing_rows = transaction
                        .query(
                            "SELECT latest_commit_hash FROM repos WHERE id = ? LIMIT 1",
                            params![repo_id.clone()],
                        )
                        .await.unwrap();

                    if let Some(row) = existing_rows.next().await.unwrap() {
                        let existing_commit_hash: String = row.get(0).unwrap();
                        if existing_commit_hash != latest_commit_hash {
                            transaction
                                .execute(
                                    "INSERT OR IGNORE INTO needs_updates (id, type_of_repo) VALUES (?, ?)",
                                    params![repo_id, repo_type],
                                )
                                .await.unwrap();
                        }
                        return;
                    }

                    let mut banned_repos = transaction
                        .query(
                            "SELECT 1 FROM banned_user_list WHERE id IN (?, ?) LIMIT 1",
                            params![format!("cb/{}", repository.owner.login).to_lowercase(), repository.owner.login.to_lowercase()],
                        )
                        .await.unwrap();

                    if banned_repos.next().await.unwrap().is_some() {
                        return;
                    }

                    transaction
                        .execute(
                            "INSERT OR IGNORE INTO index_new_repo (id, type_of_repo) VALUES (?, ?)",
                            params![repo_id, repo_type],
                        )
                        .await.unwrap();
                }
            }).await;

        transaction.commit().await.unwrap();
        page += 1;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}

pub async fn codeberg_main_cron(pool: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_codeberg_repos_cron_updating_part(pool.clone(), "zig-package", true).await?;
    fetch_all_codeberg_repos_cron_updating_part(pool.clone(), "zig", false).await?;
    Ok(())
}

pub async fn run_cron_update_once(pool: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut rows = pool
        .query(
            "SELECT id FROM needs_updates WHERE id LIKE 'cb/%'",
            params![],
        )
        .await
        .unwrap();
    let mut repos_that_need_update = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        let id: String = row.get(0).unwrap();
        repos_that_need_update.push(id);
    }

    if repos_that_need_update.is_empty() {
        return Ok(());
    }

    for id in repos_that_need_update {
        let parts: Vec<&str> = id.split('/').collect();
        if parts.len() != 3 {
            continue;
        }
        let owner = parts[1];
        let repo = parts[2];

        let url = format!("https://codeberg.org/api/v1/repos/{owner}/{repo}");
        let mut optional_response = None;
        for attempt_count in 0..5 {
            match client
                .get(&url)
                .header("Authorization", &*crate::CODEBERG_KEY)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        optional_response = Some(resp);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt_count)))
                        .await;
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt_count)))
                        .await;
                }
            }
        }

        if let Some(resp) = optional_response {
            if let Ok(json_body) = resp.text().await {
                if let Ok(daum) = serde_json::from_str::<types::types::Daum>(&json_body) {
                    let repo_data = get_repo_data(daum).await;
                    let transaction = pool.transaction().await.unwrap();
                    send_repo_data_to_database(&transaction, repo_data).await;
                    transaction
                        .execute("DELETE FROM needs_updates WHERE id = ?", params![id])
                        .await
                        .unwrap();
                    transaction.commit().await.unwrap();
                } else {
                    eprintln!("failed to parse cb json for {id}");
                }
            }
        }
    }

    Ok(())
}
