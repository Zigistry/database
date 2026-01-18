use crate::GITHUB_KEY;
use crate::github::process_repository;
use futures::stream;
use futures::stream::StreamExt;
use serde_json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use toml;
const TOML_CONTENT: &str = include_str!("../../sections.toml");
const GQL_CONTENT: &str = include_str!("../../gqlFiles/sections.gql");
// {"data":{"repository":
// I will be removing this part from the front of the responces
// this is the easiet way, because then I don't need the extra parser.
pub async fn fetch_repos_for_sections(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let parsed: HashMap<String, Vec<String>> =
        toml::from_str(TOML_CONTENT).expect("the toml is badly written.");
    stream::iter(parsed)
        .for_each_concurrent(2, |(key, value)| async move {
            let client = reqwest::Client::new();
            for library in value {
                let repo_id = library.to_lowercase();
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO index_sections
                        (section_name, repo_id)
                    VALUES(?, ?)
                "#,
                )
                .bind(&key)
                .bind(&repo_id)
                .execute(pool)
                .await
                .unwrap();

                let exists = sqlx::query("SELECT 1 FROM repos WHERE repo_id = ?")
                    .bind(&repo_id)
                    .fetch_optional(pool)
                    .await
                    .unwrap()
                    .is_some();

                if exists {
                    continue;
                }

                let mut repo_name_iter = repo_id.rsplit('/');
                let query_to_send = serde_json::json!({
                    "query": GQL_CONTENT,
                    "variables": {
                        "repo_name": repo_name_iter.next().expect("Wrong sections.toml file."),
                        "owner_name":repo_name_iter.next().expect("Wrong sections.toml file."),
                    }
                });
                let mut res2 = client
                    .post("https://api.github.com/graphql")
                    .header("Authorization", GITHUB_KEY.to_string())
                    .header("User-Agent", "zigistry.dev")
                    .json(&query_to_send)
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
                    .to_string();
                res2 = res2
                    .strip_prefix(r#"{"data":{"repository":"#)
                    .unwrap()
                    .strip_suffix("}}")
                    .expect(&res2)
                    .to_string();
                let res = match serde_json::from_str(&res2) {
                    Ok(res) => res,
                    Err(_) => {
                        eprintln!("It failed because I got this responce:{res2}");
                        eprintln!("The error: {}", res2);
                        panic!("Huge Problem.");
                    }
                };
                process_repository(&res, true, pool).await;
            }
        })
        .await;
    Ok(())
}
