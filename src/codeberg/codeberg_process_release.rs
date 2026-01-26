use libsql::{Connection, params};

use super::types;
use crate::codeberg::helper_functions::{get_build_zig_zon_data, get_readme_url};

pub async fn process_release(owner_name: &str, repo_name: &str, repo_id: &str, pool: &Connection) {
    let release_url =
        format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/releases");

    let client = reqwest::Client::new()
        .get(&release_url)
        .send()
        .await
        .unwrap();

    if client.status() != reqwest::StatusCode::OK {
        return;
    }

    let responce_as_json = client.json::<types::releases_types::Root>().await.unwrap();
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/README.md
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/build.zig.zon
    for i in responce_as_json {
        let details = match get_build_zig_zon_data(&owner_name, &repo_name, &i.tag_name, true).await
        {
            Ok(r) => r,
            Err(_) => (String::new(), Vec::new()),
        };

        let mut rows = pool
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
                    repo_id,
                    i.tag_name.clone(),
                    i.prerelease,
                    i.published_at,
                    details.0,
                    get_readme_url(&owner_name, &repo_name, &i.tag_name, true, false)
                        .await
                        .0,
                ],
            )
            .await
            .unwrap();

        let this_specific_release_id: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();

        // Clear old dependencies before inserting new ones to ensure they are up to date
        pool.execute(
            "DELETE FROM release_dependencies WHERE release_id = ?",
            params![this_specific_release_id],
        )
        .await
        .unwrap();

        for dependency in details.1.clone() {
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
    }
}
