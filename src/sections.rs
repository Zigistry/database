use std::collections::HashMap;

use crate::db;
use reqwest::header::FROM;
use serde::{Deserialize, Serialize};
use toml;
const TOML_CONTENT: &str = include_str!("../sections.toml");

#[derive(Debug, Deserialize)]
struct TomlType {
    #[serde(rename = "INDEX_PAGE_SECTION_TOPIC_URLS")]
    index_page_section_topic_urls: HashMap<String, Vec<String>>,
}

pub async fn fetch_repos_for_sections() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = toml::from_str::<TomlType>(TOML_CONTENT).expect("the toml is badly written.");
    for (k, v) in parsed.index_page_section_topic_urls {
        let value = HashMap::new();
        let client = reqwest::Client::new();
        for library in v {
            if db!().packages.contains_key(library) {

            }
            let res = client.post(library).header("Authorization", crate::GITHUB_KEY).send().await?.json<`;
            res.
        }
        db!().index_sections.insert(k);
    }
    Ok(())
}
