use sqlx::SqlitePool;

use super::types;
use crate::codeberg::helper_functions::{get_build_zig_zon_data, get_readme_url};
use crate::custom_types;
use std::collections::HashMap;

pub async fn process_release(
    owner_name: &str,
    repo_name: &str,
    repo_id: &str,
    pool: &SqlitePool,
) -> Result<HashMap<String, custom_types::Release>, Box<dyn std::error::Error>> {
    let release_url =
        format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/releases");

    let client = reqwest::Client::new().get(&release_url).send().await?;

    if client.status() != reqwest::StatusCode::OK {
        return Ok(HashMap::new());
    }

    let responce_as_json = client.json::<types::releases_types::Root>().await?;
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/README.md
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/build.zig.zon
    let mut all_releases = HashMap::new();
    for i in responce_as_json {
        let details = match get_build_zig_zon_data(&owner_name, &repo_name, &i.tag_name, true).await
        {
            Ok(r) => r,
            Err(_) => (String::new(), Vec::new()),
        };

        let this_specific_release_id: Option<i64> = sqlx::query_scalar(
            r#"
                INSERT OR IGNORE INTO releases
                    (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
                VALUES(?, ?, ?, ?, ?, ?)
                RETURNING id
            "#,
        )
        .bind(&repo_id)
        .bind(&i.tag_name)
        .bind(i.prerelease)
        .bind(i.published_at)
        .bind(details.0)
        .bind(get_readme_url(&owner_name, &repo_name, &i.tag_name, true).await)
        .fetch_optional(pool)
        .await
        .unwrap();
        match this_specific_release_id {
            Some(this_specific_release_id) => {
                for dependency in details.1.clone() {
                    sqlx::query(
                        r#"
                        INSERT INTO release_dependencies
                            (release_id, name, hash, lazy, url, path)
                        VALUES(?, ?, ?, ?, ?, ?)
                    "#,
                    )
                    .bind(this_specific_release_id)
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
                println!("Somehow {repo_id} didn't return.");
            }
        }
    }

    Ok(all_releases)
}
