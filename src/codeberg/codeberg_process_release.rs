use super::types;
use crate::codeberg::codeberg_data::ReleaseData;
use crate::codeberg::helper_functions::{get_build_zig_zon_data, get_readme_url};

pub async fn fetch_releases(owner_name: &str, repo_name: &str) -> Vec<ReleaseData> {
    let release_url =
        format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/releases");

    let client_res = reqwest::Client::new().get(&release_url).send().await;

    let client = match client_res {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if client.status() != reqwest::StatusCode::OK {
        return Vec::new();
    }

    let responce_as_json = match client.json::<types::releases_types::Root>().await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    // We can fetch releases concurrently
    let futures = responce_as_json.into_iter().map(|i| {
        let owner = owner_name.to_string();
        let repo = repo_name.to_string();
        async move {
            let details = match get_build_zig_zon_data(&owner, &repo, &i.tag_name, true).await {
                Ok(r) => r,
                Err(_) => (String::new(), Vec::new()),
            };

            let (readme_url, _) = get_readme_url(&owner, &repo, &i.tag_name, true, false).await;

            ReleaseData {
                tag_name: i.tag_name,
                is_prerelease: i.prerelease,
                published_at: i.published_at,
                minimum_zig_version: details.0,
                readme_url,
                dependencies: details.1,
            }
        }
    });

    futures::future::join_all(futures).await
}
