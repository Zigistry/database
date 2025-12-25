use super::types;
use crate::bzz_stuff::{parse, tokenize};
use crate::codeberg::helper_functions::{get_build_zig_zon_data, get_readme_url};
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::custom_types;
use std::collections::HashMap;

pub async fn process_release(
    owner_name: &str,
    repo_name: &str,
) -> Result<HashMap<String, custom_types::Release>, Box<dyn std::error::Error>> {
    let release_url =
        format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/releases");

    let client = reqwest::Client::new().get(&release_url).send().await?;

    if client.status() != reqwest::StatusCode::OK {
        return Ok(HashMap::new());
    }

    let responce_as_json = client.json::<types::releases_types::Root>().await?;
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/README.md
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/build.zig.zon
    let mut all_releases = HashMap::new();
    for i in responce_as_json {
        let details = match get_build_zig_zon_data(&owner_name, &repo_name, &i.tag_name, true).await
        {
            Ok(r) => r,
            Err(_) => (String::new(), Vec::new()),
        };

        let compiled_release = custom_types::Release {
            dependencies: details.1,
            is_prerelease: i.prerelease,
            published_at: i.published_at,
            release_assets: HashMap::new(),
            // the issue is that, Idk the type of aset
            // wasn't able to find a single repo with
            // some actual asset, hence, I am leaving this
            // as "Value" only, putting a normal hash new
            // I will figure this out after first release.
            // the code is already spoilt, need to very
            // complicated at this point.
            minimum_zig_version: details.0,
            readme_url: get_readme_url(&owner_name, &repo_name, &i.tag_name, true).await,
        };
        all_releases.insert(i.tag_name, compiled_release);
    }

    Ok(all_releases)
}
