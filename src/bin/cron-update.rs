use chrono::Utc;
use libsql::{Connection, params};
use std::error::Error;
use std::sync::Arc;
use zigistry::{
    GITHUB_KEY, constants::GH_GRAPH_QL_100_REPOS_FRAGMENT, database, github, github::types::Node,
};

#[derive(Clone, Debug)]
struct NeedsUpdateRow {
    id: String,
    type_of_repo: String,
}

fn parse_github_repo_id(repo_id: &str) -> Option<(String, String)> {
    let mut parts = repo_id.split('/');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("gh"), Some(owner), Some(name), None) if !owner.is_empty() && !name.is_empty() => {
            Some((owner.to_string(), name.to_string()))
        }
        _ => None,
    }
}

fn escape_graphql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn build_batch_query(rows: &[NeedsUpdateRow]) -> String {
    let mut query = String::from("query {\n");
    for (repo_idx, row) in rows.iter().enumerate() {
        let Some((owner, repo)) = parse_github_repo_id(&row.id) else {
            continue;
        };
        query.push_str(&format!(
            "  repo_{repo_idx}: repository(owner: \"{}\", name: \"{}\") {{ ...RepoFields }}\n",
            escape_graphql_string(&owner),
            escape_graphql_string(&repo),
        ));
    }
    query.push_str("}\n");
    query.push_str(GH_GRAPH_QL_100_REPOS_FRAGMENT);
    query
}

async fn make_update_rows_easy(
    connection: &Connection,
) -> Result<Vec<NeedsUpdateRow>, Box<dyn Error + Send + Sync>> {
    let mut rows = connection
        .query(
            "SELECT id, type_of_repo FROM needs_updates ORDER BY id",
            params![],
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(NeedsUpdateRow {
            id: row.get(0)?,
            type_of_repo: row.get(1)?,
        });
    }
    Ok(out)
}

async fn process_chunk(
    chunk: &[NeedsUpdateRow],
    connection: Arc<Connection>,
    client: &reqwest::Client,
) -> Result<(), Box<dyn Error>> {
    if chunk.is_empty() {
        return Ok(());
    }

    let query_to_send = serde_json::json!({
        "query": build_batch_query(chunk),
    });

    let response = client
        .post("https://api.github.com/graphql")
        .header("User-Agent", "zigistry")
        .header("Authorization", &*GITHUB_KEY)
        .json(&query_to_send)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("gql returned {}", response.status()).into());
    }

    let response_body = response.text().await?;
    let response_json: serde_json::Value = serde_json::from_str(&response_body)?;
    if let Some(errors) = response_json.get("errors") {
        eprintln!("gh gql errors: {errors}");
    }

    let Some(data_obj) = response_json
        .get("data")
        .and_then(|value| value.as_object())
    else {
        return Ok(());
    };

    let mut collected = Vec::new();
    for (repo_idx, row) in chunk.iter().enumerate() {
        if parse_github_repo_id(&row.id).is_none() {
            continue;
        }

        let alias = format!("repo_{repo_idx}");
        let Some(repo_value) = data_obj.get(&alias) else {
            continue;
        };
        if repo_value.is_null() {
            continue;
        }

        let node: Node = match serde_json::from_value(repo_value.clone()) {
            Ok(node) => node,
            Err(error) => {
                eprintln!("Skipping {} because parse failed: {}", row.id, error);
                continue;
            }
        };

        let is_package = row.type_of_repo.eq_ignore_ascii_case("package");
        let data = github::get_repo_data(&node, is_package, client).await;
        collected.push((row.id.clone(), data));
    }

    if collected.is_empty() {
        return Ok(());
    }

    let transaction = connection.transaction().await?;
    for (repo_id, data) in collected {
        github::persist_repo_data(&transaction, data).await;
        transaction
            .execute("DELETE FROM needs_updates WHERE id = ?", params![repo_id])
            .await?;
    }
    transaction.commit().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = Arc::new(database::connect_to_database().await?);
    let started_at = Utc::now();
    let client = reqwest::Client::new();

    match make_update_rows_easy(pool.as_ref()).await {
        Ok(rows) => {
            for chunk in rows.chunks(100) {
                if let Err(error) = process_chunk(chunk, Arc::clone(&pool), &client).await {
                    eprintln!("process_chunk failed: {error}");
                }
            }
        }
        Err(error) => {
            eprintln!("failed to read needs_updates: {error}");
        }
    }

    eprintln!(
        "cron-update finished in {} minutes.",
        (Utc::now() - started_at).num_minutes(),
    );
    Ok(())
}
