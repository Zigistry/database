pub mod github_data;
pub mod types;
use crate::bzz_stuff::{parse, tokenize};
use crate::constants::{GH_GRAPH_QL_QUERY, POSSIBLE_README_FILE_NAMES};
use crate::github::github_data::{ReleaseData, RepoData};
use crate::github::types::Node;
use crate::{GITHUB_KEY, custom_types};
use chrono::{Days, NaiveDateTime};
use futures::stream;
use futures::stream::StreamExt;
use keyword_extraction::rake::{Rake, RakeParams};
use libsql::{Connection, Transaction, params};
use std::error::Error;
use std::sync::Arc;

const EMPTY_REPLY: &str =
    r#"{"data":{"search":{"pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[]}}}"#;

pub async fn process_last_15_minutes(
    connection: Arc<Connection>,
    query: String,
    is_package: bool,
    time_15_minutes_ago: NaiveDateTime,
) {
    // 30, so that this is fail safe. And coveres all previous nproblems.
    let client = Arc::new(reqwest::Client::new());
    let mut has_next = true;
    let mut next: Option<String> = None;

    while has_next {
        let query_to_send = serde_json::json!({
            "query": GH_GRAPH_QL_QUERY,
            "variables":  {
                "query": format!("topic:{query} pushed:>{}", time_15_minutes_ago.format("%Y-%m-%dT%H:%M:%SZ")),
                "next_value": next
            }
        });

        let text = fetch_with_retry(&client, query_to_send).await;

        if text == EMPTY_REPLY {
            has_next = false;
            continue;
        }

        let res2: types::Root = match serde_json::from_str(&text) {
            Ok(t) => t,
            Err(t) => {
                panic!("Got this problem: {t}");
            }
        };

        has_next = res2.data.search.page_info.has_next_page;
        next = Option::from(res2.data.search.page_info.end_cursor);
        let process_nodes = res2.data.search.nodes;

        // Increased concurrency to 20 (was 5)
        // Maybe I can increate it later.

        let transaction = connection.transaction().await.unwrap();
        stream::iter(process_nodes.clone())
            .map(|node| {
                let cli = Arc::clone(&client);
                async move { get_repo_data(&node, is_package, &cli).await }
            })
            .buffer_unordered(5)
            .for_each(|data| async {
                persist_repo_data(&transaction, data).await;
            })
            .await;

        transaction.commit().await.unwrap();

    }
}

pub async fn get_repo_data(
    repository: &Node,
    is_package: bool,
    client: &reqwest::Client,
) -> RepoData {
    let user_id = format!("gh/{}", repository.owner.login).to_lowercase();
    let repo_id = format!("gh/{}/{}", repository.owner.login, repository.name).to_lowercase();

    let default_branch_name = repository
        .default_branch_ref
        .clone()
        .unwrap_or_default()
        .name
        .clone();

    let branch = if default_branch_name.is_empty() {
        "HEAD"
    } else {
        default_branch_name.as_ref()
    };

    let (build_zig_zon_data, (readme_url, readme_keywords)) = tokio::join!(
        get_build_zig_zon_data_wrapper(&repository.owner.login, &repository.name, branch, client),
        get_readme_url_and_keywords(
            &repository.owner.login,
            &repository.name,
            branch,
            true,
            client
        )
    );

    let (readme_url, readme_keywords) = match (readme_url, readme_keywords) {
        (Some(url), Some(kw)) => (url, kw),
        (Some(url), None) => (url, String::new()),
        _ => ("404 unable to find readme. ".to_string(), String::new()),
    };

    let releases_iter = repository.releases.nodes.iter();
    let releases_futures = releases_iter.map(|release| {
        let owner = repository.owner.login.clone();
        let name = repository.name.clone();
        let tag = release.tag_name.clone();
        let release_clone = release.clone();
        let cli = client.clone();

        async move {
            let (readme_url, _) =
                match get_readme_url_and_keywords(&owner, &name, &tag, false, &cli).await {
                    (Some(url), _) => (url, String::new()),
                    _ => ("404 unable to find readme.".to_string(), String::new()),
                };

            let bzz_results = get_build_zig_zon_data_wrapper(&owner, &name, &tag, &cli).await;

            ReleaseData {
                tag_name: release_clone.tag_name,
                is_prerelease: release_clone.is_prerelease,
                published_at: release_clone.published_at,
                minimum_zig_version: bzz_results.0,
                readme_url,
                dependencies: bzz_results.1,
            }
        }
    });

    let releases = futures::future::join_all(releases_futures).await;

    RepoData {
        repository: repository.clone(),
        is_package,
        user_id,
        repo_id,
        readme_url,
        readme_keywords,
        build_zig_zon_version: build_zig_zon_data.0,
        build_zig_zon_dependencies: build_zig_zon_data.1,
        releases,
    }
}

pub async fn persist_repo_data(transaction: &Transaction, data: RepoData) {
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
                "github",
                repository.owner.login.clone(),
                repository.owner.bio.clone()
            ],
        )
        .await
        .unwrap();

    transaction.execute(
        r#"
            INSERT INTO repos
                (id, avatar_id, owner, platform, description, issues_count, default_branch_name, fork_count
                , stargazer_count, watchers_count, pushed_at, created_at, is_archived, is_disabled,
                is_fork, minimum_zig_version, license, primary_language)
            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                license = excluded.license,
                primary_language = excluded.primary_language
        "#,
        params![
            data.repo_id.clone(),
            repository.owner. login.clone(),
            data.user_id.clone(),
            "github",
            repository.description.clone(),
            repository.issues. total_count,
            repository.default_branch_ref.clone().unwrap_or_default().name,
            repository.fork_count,
            repository.stargazer_count,
            repository.watchers. total_count,
            repository. pushed_at.clone(),
            repository.created_at.clone(),
            repository.is_archived,
            repository.is_disabled,
            repository.is_fork,
            data.build_zig_zon_version.clone(),
            repository
                .license_info
                .as_ref()
                .and_then(|l| Some(l.spdx_id.clone()))
                .unwrap_or_else(|| "-".to_string()),
            repository.primary_language.clone().unwrap_or_default().name,
        ],
    )
    .await.unwrap();

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
            params![
                data.repo_id.clone(),
                repository
                    .repository_topics
                    .edges
                    .iter()
                    .map(|element| element.node.topic.name.clone())
                    .collect::<Vec<String>>()
                    .join(",")
                    .clone()
            ],
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
                data.readme_url.clone(),
            ],
        )
        .await
        .unwrap();

    let default_branch_release_id: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
    drop(rows);

    // Clear old dependencies before inserting new ones to ensure they are up to date
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
                    dependency.path,
                ],
            )
            .await
            .unwrap();
    }

    // Just sending processed data to the transaction.
    for release_data in data.releases {
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
                    release_data.tag_name,
                    release_data.is_prerelease,
                    release_data.published_at,
                    release_data.minimum_zig_version,
                    release_data.readme_url,
                ],
            )
            .await
            .unwrap();

        let this_specific_release_id: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        drop(rows);

        // Clear old dependencies before inserting new ones to ensure they are up to date
        transaction
            .execute(
                "DELETE FROM release_dependencies WHERE release_id = ?",
                params![this_specific_release_id],
            )
            .await
            .unwrap();

        for dependency in release_data.dependencies {
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

    if data.is_package {
        transaction
            .execute(
                r#"
                 INSERT OR IGNORE INTO packages
                    (repo_id)
                VALUES(?)
            "#,
                params![data.repo_id],
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
                params![data.repo_id],
            )
            .await
            .unwrap();
    }
}

pub async fn process_query(
    query: &str,
    is_package: bool,
    pool: Arc<Connection>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = start_date;
    let end = end_date;
    let mut lower = start;
    let mut upper = start.checked_add_days(Days::new(step)).unwrap();
    let client = Arc::new(reqwest::Client::new());

    loop {
        let mut nodes = Vec::new();
        eprintln!("Now processing:{lower}..{upper}");
        let mut has_next = true;
        let mut next: Option<String> = None;

        while has_next {
            let query_to_send = serde_json::json!({
                "query": GH_GRAPH_QL_QUERY,
                "variables":  {
                    "query": format!("topic:{query} created:{}..{}", lower.format("%Y-%m-%dT%H:%M:%SZ"), upper.format("%Y-%m-%dT%H:%M:%SZ")),
                    "next_value": next
                }
            });

            let text = fetch_with_retry(&client, query_to_send).await;

            if text == EMPTY_REPLY {
                has_next = false;
                continue;
            }

            let mut res2: types::Root = match serde_json::from_str(&text) {
                Ok(t) => t,
                Err(t) => {
                    eprintln!("Got this response:  {text}");
                    panic!("Got this problem: {t}");
                }
            };

            eprintln!("{:#?}", res2.data.search.page_info.has_next_page);
            has_next = res2.data.search.page_info.has_next_page;
            next = Option::from(res2.data.search.page_info.end_cursor);
            nodes.append(&mut res2.data.search.nodes);
        }

        // Increased concurrency from 100 to handle the parallel work better

        let transaction = pool.transaction().await.unwrap();
        stream::iter(&nodes)
            .map(|node| {
                let cli = Arc::clone(&client);
                async move { get_repo_data(node, is_package, &cli).await }
            })
            .buffer_unordered(50)
            .for_each(|data| async {
                persist_repo_data(&transaction, data).await;
            })
            .await;

        transaction.commit().await.unwrap();

        lower = upper;
        upper = lower.checked_add_days(Days::new(step)).unwrap();
        if lower > end {
            break;
        }
    }
    return Ok(());
}

pub async fn github_main(
    pool: Arc<Connection>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn Error>> {
    process_query(
        "zig-package",
        true,
        Arc::clone(&pool),
        start_date,
        end_date,
        step,
    )
    .await
    .unwrap();
    process_query("zig", false, pool, start_date, end_date, step)
        .await
        .unwrap();
    Ok(())
}

pub async fn get_readme_url_and_keywords(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    process_keywords: bool,
    client: &reqwest::Client,
) -> (Option<String>, Option<String>) {
    let name =
        format!("https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/");

    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let url = name.to_string() + readme_file_name;

        // Try GET directly instead of HEAD first - fewer requests
        match client.get(&url).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    if process_keywords {
                        match res.text().await {
                            Ok(content) => {
                                let rake = Rake::new(RakeParams::WithDefaults(
                                    &content,
                                    &crate::stop_words_in_eng,
                                ));
                                let mut keywords = rake.get_ranked_keyword(200);
                                keywords.push(owner_name.to_string());
                                keywords.push(repo_name.to_string());
                                let keyword_string = keywords.join(" ");
                                return (Some(url), Some(keyword_string));
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to read README body for {owner_name}/{repo_name}: {e}"
                                );
                                continue;
                            }
                        }
                    }
                    return (Some(url), None);
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
    (None, None)
}

pub async fn get_build_zig_zon_data(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    client: &reqwest::Client,
) -> Result<(String, Vec<custom_types::Dependency>), Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/build.zig.zon"
    );
    let text = client.get(&url).send().await?.text().await?;

    let tokens = tokenize(text.chars())?;
    let parsed = parse(tokens.into_iter())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}

async fn get_build_zig_zon_data_wrapper(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    client: &reqwest::Client,
) -> (String, Vec<custom_types::Dependency>) {
    match get_build_zig_zon_data(owner_name, repo_name, branch_or_tag, client).await {
        Ok((minimum_zig_version, dependencies)) => (minimum_zig_version, dependencies),
        Err(_) => {
            eprintln!(
                "Parser wasn't able to parse:  https://github.com/{}/{}",
                owner_name, repo_name
            );
            ("unknown".to_string(), Vec::new())
        }
    }
}

// Helper to reduce code duplication
async fn fetch_with_retry(client: &reqwest::Client, query_to_send: serde_json::Value) -> String {
    let mut retry_count = 0usize;
    loop {
        if retry_count > 8 {
            panic!("Tried {} times, still problem.", retry_count);
        }

        match client
            .post("https://api.github.com/graphql")
            .header("Authorization", GITHUB_KEY.to_string())
            .header("User-Agent", "zigistry. dev")
            .json(&query_to_send)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    eprintln!("problem:  {}", resp.status());
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                match resp.text().await {
                    Ok(body) => return body,
                    Err(e) => {
                        eprintln!("problem: {e}");
                        retry_count += 1;
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("GitHub Error: {e}");
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        }
    }
}
