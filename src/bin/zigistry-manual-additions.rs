use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use zigistry::{
    GITHUB_KEY,
    database::connect_to_database,
    github::{get_repo_data, persist_repo_data, types::Node},
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

async fn process_repo_from_string(
    repo: &str,
    client: &reqwest::Client,
    connection: Arc<libsql::Connection>,
) {
    let mut repo_name_iter = repo.split('/');
    let _ = repo_name_iter.next().unwrap(); // this is the provider like gh or cb, ONLY gh is supported for now.
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
        .header("User-Agent", "zigistry")
        .header("Authorization", &*GITHUB_KEY)
        .json(&query_to_send)
        .send()
        .await
        .unwrap();
    let response_body = response.text().await.unwrap();

    println!("Got this: {}", response_body);

    let resbody = response_body
        .strip_prefix("{\"data\":{\"repository\":")
        .unwrap()
        .strip_suffix("}}")
        .unwrap();
    let response_json: Node = serde_json::from_str(&resbody).unwrap();
    if !zigistry::github::has_zig_in_top_languages(&response_json) {
        eprintln!(
            "Skipping {} - Zig is not in the top 10 languages.",
            repo.to_lowercase()
        );
        return;
    }
    let data = zigistry::github::get_repo_data(&response_json, true, &client).await;
    let transaction = connection.transaction().await.unwrap();
    zigistry::github::persist_repo_data(&transaction, data).await;
    transaction.commit().await.unwrap();
}

#[tokio::main]
pub async fn main() {
    let toml_content = std::fs::read_to_string("sections-and-manual-addition.toml").unwrap();
    let config: Config = toml::from_str(&toml_content).unwrap();

    let client = reqwest::Client::new();

    let connection = Arc::new(connect_to_database().await.unwrap());

    for repo in config.manual_additions.extras {
        process_repo_from_string(&repo, &client, connection.clone()).await;
    }
    for (section, repos) in config.manual_additions.sections {
        for repo in repos {
            process_repo_from_string(&repo, &client, connection.clone()).await;
            connection
                .execute(
                    "INSERT OR REPLACE INTO index_sections (section_name, repo_id) VALUES (?, ?)",
                    (section.clone().to_lowercase(), repo.clone().to_lowercase()),
                )
                .await
                .expect(&format!("FAILED ON: {repo}"));
        }
    }
}
