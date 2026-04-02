use libsql::params;
use zigistry::{
    CODEBERG_KEY, GITHUB_KEY, constants::GH_GRAPH_QL_100_REPOS_FRAGMENT,
    database::connect_to_database,
};

fn make_repo_query(owner_name: &str, repo_name: &str) -> String {
    format!(
        "query {{\n  repository(owner: \"{}\", name: \"{}\") {{ ...RepoFields }}\n}}\n{}",
        owner_name.replace('\\', "\\\\").replace('"', "\\\""),
        repo_name.replace('\\', "\\\\").replace('"', "\\\""),
        GH_GRAPH_QL_100_REPOS_FRAGMENT // Actually, this is a fragment I can use for 1 repo as well.
    )
}

async fn process_codeberg_repo(
    owner_name: &str,
    repo_name: &str,
    type_of_repo: &str,
    client: &reqwest::Client,
) {
    todo!();
}
async fn process_github_repo(
    owner_name: &str,
    repo_name: &str,
    type_of_repo: &str,
    client: &reqwest::Client,
) {
    let query = make_repo_query(owner_name, repo_name);
    let body_content = serde_json::json!({ "query": query });
    let res = client
        .post("https://api.github.com/graphql")
        .header("User-Agent", "zigistry")
        .header("Authorization", &*GITHUB_KEY)
        .json(&body_content)
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        return;
    }

    let json_text = res.text().await.unwrap();

    println!("{json_text}");
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let connection = connect_to_database().await.unwrap();

    loop {
        let mut rows_to_process = connection
            .query(
                "SELECT id, type_of_repo FROM safe_to_index_new_repo",
                params![],
            )
            .await
            .unwrap();

        {
            while let Some(repo_row) = rows_to_process.next().await.unwrap() {
                let id: String = repo_row.get(0).unwrap();
                let type_of_repo: String = repo_row.get(1).unwrap();

                let mut parts = id.split('/');

                let platform = parts.next().unwrap();
                let owner_name = parts.next().unwrap();
                let repo_name = parts.next().unwrap();

                if platform == "gh" {
                    process_github_repo(owner_name, repo_name, &type_of_repo, &client).await;
                } else if platform == "cb" {
                    process_codeberg_repo(owner_name, repo_name, &type_of_repo, &client).await;
                } else {
                    panic!("got an unknown platform. {}", id)
                }
            }
        }
        let rows_to_process = connection
            .query(
                "DELETE FROM safe_to_index_new_repo
         WHERE id IN (SELECT id FROM safe_to_index_new_repo)
         RETURNING id, type_of_repo",
                params![],
            )
            .await
            .unwrap();
    }
}
