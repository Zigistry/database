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

        let this_specific_release_id: u64 = pool
            .execute(
                r#"
                INSERT OR IGNORE INTO releases
                    (repo_id, version, is_prerelease, published_at, minimum_zig_version, readme_url)
                VALUES(?, ?, ?, ?, ?, ?)
            "#,
                params![
                    &repo_id,
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

        if this_specific_release_id != 0 {
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
        } else {
            println!("Somehow {repo_id} didn't return.");
        }
    }
}
