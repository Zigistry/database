pub mod types;
use crate::bzz_stuff::{parse, tokenize};
use crate::constants::{GH_GRAPH_QL_QUERY, POSSIBLE_README_FILE_NAMES};
use crate::{GITHUB_KEY, custom_types};
use chrono::{Days, Months, NaiveDateTime};
use futures::stream;
use futures::stream::StreamExt;
use keyword_extraction::rake::{Rake, RakeParams};
use libsql::Connection;
use libsql::params;
use std::error::Error;

const EMPTY_REPLY: &str =
    r#"{"data":{"search":{"pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[]}}}"#;

pub async fn process_repository(repository: &types::Node, is_package: bool, pool: &Connection) {
    let user_id = format!("gh/{}", repository.owner.login).to_lowercase();
    let repo_id = format!("gh/{}/{}", repository.owner.login, repository.name).to_lowercase();
    // println!("Processing User: {}", repository.owner.login);
    pool.execute(
        r#"
            INSERT OR IGNORE INTO users
                (id, platform, avatar_id, bio)
            VALUES (?, ?, ?, ?)
        "#,
        params![
            user_id.clone(),
            "github",
            // I am using owner login name
            // for the avatar id because
            // it works and uses very low storage
            // as compaired to storing the entire
            // avatar url.
            repository.owner.login.clone(),
            repository.owner.bio.clone()
        ],
    )
    .await
    .unwrap();

    let default_branch_name = repository
        .default_branch_ref
        .clone()
        .unwrap_or_default()
        .name
        .clone();

    let build_zig_zon_data = get_build_zig_zon_data_wrapper(
        &repository.owner.login,
        repository.name.as_str(),
        if default_branch_name.is_empty() {
            "HEAD"
        } else {
            default_branch_name.as_ref()
        },
    )
    .await;

    let (readme_url, readme_keywords) = match get_readme_url_and_keywords(
        &repository.owner.login,
        repository.name.as_str(),
        if default_branch_name.is_empty() {
            "HEAD"
        } else {
            default_branch_name.as_ref()
        },
        true,
    )
    .await
    {
        Some(url) => url,
        _ => ("404 unable to find readme.".to_string(), String::new()),
    };

    pool.execute(
        r#"
            INSERT OR IGNORE INTO repos
                (id, avatar_id, owner, platform, description, issues_count, default_branch_name, fork_count
                , stargazer_count, watchers_count, pushed_at, created_at, is_archived, is_disabled,
                is_fork, license, primary_language, search_keywords)
            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            repo_id.clone(),
            user_id.clone(),
            "github",
            repository.description.clone(),
            repository.issues.total_count,
            repository.default_branch_ref.clone().unwrap_or_default().name,
            repository.fork_count,
            repository.stargazer_count,
            repository.watchers.total_count,
            repository.pushed_at.clone(),
            repository.created_at.clone(),
            repository.is_archived,
            repository.is_disabled,
            repository.is_fork,
            repository.license_info.clone().unwrap_or_default().spdx_id,
            repository.primary_language.clone().unwrap_or_default().name,
            readme_keywords
        ],
    )
    .await.unwrap();
    // eprintln!("Processing Repository: {}", repository.name);

    let default_branch_release_id: u64 = pool
        .execute(
            r#"
            INSERT OR IGNORE INTO releases
                (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
            VALUES(?, ?, ?, ?, ?, ?)
            RETURNING id
        "#,
            params![
                repo_id.clone(),
                "__ZIGISTRY__DEFAULT__BRANCH__",
                false,
                repository.created_at.clone(),
                build_zig_zon_data.0.clone(),
                readme_url.clone(),
            ],
        )
        .await
        .unwrap();
    if default_branch_release_id != 0 {
        for dependency in build_zig_zon_data.1.clone() {
            pool.execute(
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
    } else {
        println!("Got None for: {}", &repo_id);
    }

    for release in &repository.releases.nodes {
        let (readme_url, _) = match get_readme_url_and_keywords(
            &repository.owner.login,
            &repository.name,
            &release.tag_name,
            false,
        )
        .await
        {
            Some(url) => url,
            _ => ("404 unable to find readme.".to_string(), String::new()),
        };
        let bzz_results = get_build_zig_zon_data_wrapper(
            &repository.owner.login,
            &repository.name,
            &release.tag_name,
        )
        .await;

        let this_specific_release_id: u64 = pool
            .execute(
                r#"
            INSERT OR IGNORE INTO releases
                (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
            VALUES(?, ?, ?, ?, ?, ?)
        "#,
                params![
                    repo_id.clone(),
                    release.tag_name.clone(),
                    release.is_prerelease,
                    release.published_at.clone(),
                    bzz_results.0.clone(),
                    readme_url,
                ],
            )
            .await
            .unwrap();
        if this_specific_release_id != 0 {
            for dependency in bzz_results.1.clone() {
                pool.execute(
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
        } else {
            println!("Somehow {repo_id} didn't return.");
        }
    }
    if is_package {
        pool.execute(
            r#"
                 INSERT OR IGNORE INTO packages
                    (repo_id)
                VALUES(?)
            "#,
            params![repo_id],
        )
        .await
        .unwrap();
    } else {
        pool.execute(
            r#"
                 INSERT OR IGNORE INTO programs
                    (repo_id)
                VALUES(?)
            "#,
            params![repo_id],
        )
        .await
        .unwrap();
    }
}

pub async fn process_query(
    query: &str,
    is_package: bool,
    pool: &Connection,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Yes, 8th Feb 2016, Zig release date.
    let start = start_date;
    let end = end_date;
    let mut lower = start;
    let mut upper = start.checked_add_days(Days::new(step)).unwrap();
    let client = reqwest::Client::new();
    loop {
        let mut nodes = Vec::new();
        eprintln!("Now processing:{lower}..{upper}");
        let mut has_next = true;
        let mut next: Option<String> = None;
        while has_next {
            let query_to_send = serde_json::json!({
                "query": GH_GRAPH_QL_QUERY,
                "variables": {
                    "query": format!("topic:{query} created:{lower}..{upper}"),
                    "next_value": next
                }
            });

            // We'll retry the request + body read a few times in case of transient network issues.
            let mut retry_count = 0usize;
            let text = loop {
                if retry_count > 8 {
                    panic!("Tried {} times, still problem.", retry_count);
                }

                match client
                    .post("https://api.github.com/graphql")
                    .header("Authorization", GITHUB_KEY.to_string())
                    .header("User-Agent", "zigistry.dev")
                    .json(&query_to_send)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            eprintln!("problem: {}", resp.status());
                            retry_count += 1;
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            continue;
                        }
                        match resp.text().await {
                            Ok(body) => break body,
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
            };

            if text == EMPTY_REPLY {
                has_next = false;
                continue;
            }
            let mut res2: types::Root = match serde_json::from_str(&text) {
                Ok(t) => t,
                Err(t) => {
                    eprintln!("Got this response:");
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
                process_repository(&node, is_package, pool).await;
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

pub async fn github_main(
    pool: &Connection,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    step: u64,
) -> Result<(), Box<dyn Error>> {
    process_query("zig-package", true, pool, start_date, end_date, step)
        .await
        .unwrap();
    process_query("zig", false, &pool, start_date, end_date, step)
        .await
        .unwrap();
    Ok(())
}

pub async fn get_readme_url_and_keywords(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    process_keywords: bool,
) -> Option<(String, String)> {
    let name =
        format!("https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/");

    let client = reqwest::Client::new();
    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let mine = name.to_string() + readme_file_name;
        let res = match client.head(&mine).send().await {
            Ok(t) => t,
            Err(_) => {
                print!("skipping readme {owner_name}/{repo_name}");
                continue;
            }
        };
        if res.status().is_success() {
            if process_keywords {
                let res = match client.get(&mine).send().await {
                    Ok(t) => t,
                    Err(_) => {
                        print!("skipping readme {owner_name}/{repo_name}");
                        continue;
                    }
                };
                if res.status().is_success() {
                    let content = match res.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!(
                                "Failed to read README body for {owner_name}/{repo_name}: {e}"
                            );
                            continue;
                        }
                    };
                    let rake = Rake::new(RakeParams::WithDefaults(
                        &content,
                        &crate::stop_words_in_eng,
                    ));
                    // Afaik, 200 keywords is overkill.
                    let keywords = rake.get_ranked_keyword(200);
                    let keyword_string = keywords.join(" ");
                    return Option::from((mine, keyword_string));
                }
            }
            return Option::from((mine, String::new()));
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
