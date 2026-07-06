use libsql::{Connection, params};
use std::error::Error;
use zigistry::{codeberg, database, github, keyword_extraction};

struct Repo {
    id: String,
    description: String,
    branch: String,
}

fn parse_repo_id(id: String) -> Result<(String, String, String), Box<dyn Error>> {
    let parts: Vec<&str> = id.split('/').collect();
    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

async fn get_repos(conn: &Connection) -> Result<Vec<Repo>, Box<dyn Error>> {
    let mut rows = conn
        .query(
            "SELECT id, COALESCE(description, ''), default_branch_name FROM repos",
            params![],
        )
        .await?;

    let mut repos = Vec::new();

    while let Some(row) = rows.next().await? {
        repos.push(Repo {
            id: row.get(0)?,
            description: row.get(1)?,
            branch: row.get(2)?,
        });
    }

    Ok(repos)
}

async fn get_readme(
    platform: String,
    owner: String,
    name: String,
    branch: String,
    client: &reqwest::Client,
) -> String {
    let branch = if branch.trim().is_empty() {
        "HEAD".to_string()
    } else {
        branch
    };

    match platform.as_str() {
        "gh" => {
            let (_, content) =
                github::get_readme_url_and_content(&owner, &name, &branch, true, client).await;
            content.unwrap_or_default()
        }
        "cb" => {
            let directory_files = codeberg::helper_functions::fetch_root_folder_directory_files(
                client, &owner, &name, &branch,
            )
            .await;
            let (_, content) = codeberg::helper_functions::get_readme_url(
                &owner,
                &name,
                &branch,
                false,
                true,
                &directory_files,
            )
            .await;
            content
        }
        _ => panic!("database id is not in correct format."),
    }
}

async fn save_keywords(
    conn: &Connection,
    repo_id: String,
    keywords: String,
) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM repo_search WHERE repo_id = ?",
        params![repo_id.clone()],
    )
    .await?;
    conn.execute(
        "INSERT INTO repo_search (repo_id, keywords) VALUES (?, ?)",
        params![repo_id, keywords],
    )
    .await?;
    Ok(())
}

async fn handle_repo(
    conn: &Connection,
    client: &reqwest::Client,
    repo: &Repo,
) -> Result<(), Box<dyn Error>> {
    let (platform, owner, name) = parse_repo_id(repo.id.clone()).unwrap();
    let readme = get_readme(
        platform,
        owner.clone(),
        name.clone(),
        repo.branch.clone(),
        client,
    )
    .await;

    let keywords = keyword_extraction(
        readme.as_str(),
        &repo.description,
        name.as_str(),
        owner.as_str(),
    )
    .await?;

    save_keywords(conn, repo.id.clone(), keywords).await?;

    println!("indexed {}", repo.id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = database::connect_to_database().await?;
    let client = reqwest::Client::new();

    let repos = get_repos(&conn).await?;

    for repo in repos {
        handle_repo(&conn, &client, &repo).await.unwrap();
    }

    println!("COMPLETED!!!!!");

    Ok(())
}
