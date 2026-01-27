use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use zigistry::{
    GITHUB_KEY,
    database::connect_to_database,
    github::{process_repository, types::Node},
};

#[derive(Debug, Deserialize)]
struct ManualAdditions {
    extras: Vec<String>,
    sections: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    manual_additions: ManualAdditions,
}

const MY_GQL_QUERY: &str = include_str!("../../GitHub_GQL_API_Files/single_repo.gql");

async fn process_repo(repo: &str, client: &reqwest::Client, connection: Arc<libsql::Connection>) {
    let mut repo_name_iter = repo.split('/');
    let owner = repo_name_iter.next().unwrap();
    let name = repo_name_iter.next().unwrap();

    let query_to_send = serde_json::json!({
        "query": MY_GQL_QUERY,
        "variables": {
            "repo_name": name,
            "owner_name":owner,
        }
    });
    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", "Bearer ".to_string() + &*GITHUB_KEY)
        .json(&query_to_send)
        .send()
        .await
        .unwrap();
    let response_body = response.text().await.unwrap();
    let response_json: Node = serde_json::from_str(&response_body).unwrap();
    process_repository(&response_json, true, &connection, &client).await;
}

#[tokio::main]
pub async fn main() {
    let toml_content = std::fs::read_to_string("sections-and-manual-addition.toml").unwrap();
    let config: Config = toml::from_str(&toml_content).unwrap();

    let client = reqwest::Client::new();

    let connection = Arc::new(connect_to_database().await.unwrap());

    for repo in config.manual_additions.extras {
        process_repo(&repo, &client, connection.clone()).await;
    }
    for repo in config.manual_additions.sections.values() {
        for repo in repo {
            process_repo(&repo, &client, connection.clone()).await;
        }
    }
}
