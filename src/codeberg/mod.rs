mod codeberg_process_release;
mod helper_functions;
pub mod types;

use crate::CODEBERG_KEY;
use crate::codeberg::helper_functions::get_readme_url;
use crate::constants::ASYNC_LIMIT;
use crate::github::get_build_zig_zon_data;
use codeberg_process_release::process_release;
use futures::{stream, stream::StreamExt};
use sqlx::SqlitePool;

pub async fn fetch_all_codeberg_repos(
    pool: &SqlitePool,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_repos = vec![];
    let mut page = 1;

    let client = reqwest::Client::new();

    loop {
        let url = format!(
            "https://codeberg.org/api/v1/repos/search?q={query}&limit=50&page={page}&topic=true"
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

        for repo in responce.data {
            all_repos.push(repo);
        }
        page += 1;
    }

    eprintln!("All repos len: {}", all_repos.len());

    stream::iter(all_repos)
        .for_each_concurrent(ASYNC_LIMIT, |repository| async move {
            let user_id = format!("cb/{}", repository.owner.login).to_lowercase();
            let repo_id = format!("cb/{}/{}", repository.owner.login, repository.name).to_lowercase();
            sqlx::query(
                r#"
                    INSERT OR IGNORE INTO users
                        (id, platform, avatar_id, bio, followers, following, location, description, website_url)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&user_id)
            // I am using owner login name
            // for the avatar id because
            // it works and uses very low storage
            // as compaired to storing the entire
            // avatar url.
            .bind("github")
            .bind(repository.owner.login.clone())
            .bind(repository.owner.description.clone())
            .bind(
                repository
                    .owner
                    .followers_count
            )
            .bind(
                repository
                    .owner
                    .following_count
            )
            .bind(repository.owner.description.clone())
            .bind(repository.owner.website.clone())
            .execute(pool)
            .await
            .unwrap();

            let build_zig_zon_data = match get_build_zig_zon_data(&repository.owner.login, &repository.name, "HEAD").await
            {
                Ok(t) => t,
                Err(_) => (String::new(), Vec::new()),
            };

            sqlx::query(
        r#"
            INSERT OR IGNORE INTO repos
                (id, avatar_id, owner, platform, description, issues_count, default_branch_name, fork_count
                , stargazer_count, watchers_count, pushed_at, created_at, is_archived, is_disabled,
                is_fork, license, primary_language)
            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    ).bind(&repo_id)
    .bind(
          repository
            .owner
            .avatar_url
            .rsplit("/")
            .next()
            .unwrap()
            .to_string()
    )
   .bind(user_id)
    .bind("codeberg")
    .bind(repository.description.to_string())
    .bind(repository.open_issues_count)
    .bind(repository.default_branch.clone())
    .bind(repository.forks_count)
    .bind(repository.stars_count)
    .bind(repository.watchers_count)
    .bind(repository.updated_at.clone())
    .bind(repository.created_at.clone())
    .bind(repository.archived)
    .bind(repository.fork)
    .bind("Not Found") // Only for now
    .bind(repository.language.clone())
    .execute(pool).await.unwrap();

    let default_branch_release_id: Option<i64> = sqlx::query_scalar(
        r#"
            INSERT OR IGNORE INTO releases
                (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
            VALUES(?, ?, ?, ?, ?, ?)
            RETURNING id
        "#,
    )
    .bind(&repo_id)
    .bind("__ZIGISTRY__DEFAULT__BRANCH__")
    .bind(false)
    .bind(repository.created_at.clone())
    .bind(build_zig_zon_data.0.clone())
    .bind(
        get_readme_url(
            &repository.owner.login,
            repository.name.as_str(),
            &repository.default_branch,
            false
        ).await
    )
    .fetch_optional(pool)
    .await
    .unwrap();

 match default_branch_release_id {
        Some(default_branch_release_id) => {
            for dependency in build_zig_zon_data.1.clone() {
                sqlx::query(
                    r#"
                        INSERT INTO release_dependencies
                            (release_id, name, hash, lazy, url, path)
                        VALUES(?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(default_branch_release_id)
                .bind(dependency.name)
                .bind(dependency.hash)
                .bind(dependency.lazy)
                .bind(dependency.url)
                .bind(dependency.path)
                .execute(pool)
                .await
                .unwrap();
            }
        }
        None => {
            println!("Got None for: {}", &repo_id);
        }
    }
    process_release(&repository.owner.login, &repository.name,&repo_id, &pool)
                    .await
                    .unwrap_or_default();    
             if repository
                .topics             .contains(&"zig-package".to_string())
            {
sqlx::query(
            r#"
                 INSERT INTO packages
                    (repo_id)
                VALUES(?)
            "#,
        )
        .bind(repo_id)
        .execute(pool)
        .await
        .unwrap();
                            } else {
sqlx::query(
            r#"
                 INSERT INTO programs
                    (repo_id)
                VALUES(?)
            "#,
        )
        .bind(repo_id)
        .execute(pool)
        .await
        .unwrap();
                           }
        })
        .await;

    Ok(())
}

pub async fn codeberg_main(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_codeberg_repos(&pool, "zig-package")
        .await
        .unwrap();
    fetch_all_codeberg_repos(&pool, "zig").await.unwrap();
    Ok(())
}
