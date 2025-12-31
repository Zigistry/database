mod api_responce_types;
use crate::GITHUB_KEY;
use crate::custom_types;
use crate::db;
use crate::github::process_query;
use crate::github::process_repository;
use crate::github::types;
use serde_json;
use std::collections::HashMap;
use toml;

const TOML_CONTENT: &str = include_str!("../../sections.toml");
const GQL_CONTENT: &str = include_str!("../../gqlFiles/sections.gql");

pub async fn fetch_repos_for_sections() -> Result<(), Box<dyn std::error::Error>> {
    let parsed: HashMap<String, Vec<String>> =
        toml::from_str(TOML_CONTENT).expect("the toml is badly written.");
    for (k, v) in parsed {
        db!().index_sections.insert(k, v.clone());
        let client = reqwest::Client::new();
        for library in v {
            if db!().packages.contains_key(&library) {
                continue;
            }
            let mut repo_name_iter = library.rsplit("/");
            let query_to_send = serde_json::json!({
                "query": GQL_CONTENT,
                "variables": {
                    "query": format!("repo_name:{} owner_name:{}", repo_name_iter.next().expect("Wrong sections.toml file."), repo_name_iter.next().expect("Wrong sections.toml file.")),
                }
            });
            let res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", GITHUB_KEY.to_string())
                .header("User-Agent", "zigistry.dev")
                .json(&query_to_send)
                .send()
                .await?
                .json::<crate::github::types::Node>()
                .await?;
            process_repository(&res, true).await;
        }
    }
    Ok(())
}
