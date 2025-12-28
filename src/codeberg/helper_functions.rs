use crate::bzz_stuff;
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::custom_types;

pub async fn get_readme_url(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag: bool,
) -> String {
    let branch_or_tag_value = if is_tag { "tag" } else { "branch" };
    let url = format!(
        "https://codeberg.org/{owner_name}/{repo_name}/raw/{branch_or_tag_value}/{branch_or_tag}/"
    );

    let client = reqwest::Client::new();

    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let readme_possible_url = url.clone() + readme_file_name;

        let responce = client.head(&readme_possible_url).send().await.unwrap();
        if responce.status().is_success() {
            return readme_possible_url;
        }
    }

    String::new()
}

pub async fn get_build_zig_zon_data(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag: bool,
) -> Result<(String, Vec<custom_types::Dependency>), Box<dyn std::error::Error>> {
    let branch_or_tag_value = if is_tag { "tag" } else { "branch" };
    let url = format!(
        "https://codeberg.org/{owner_name}/{repo_name}/raw/{branch_or_tag_value}/{branch_or_tag}/"
    );

    let client = reqwest::Client::new();
    let text = client.get(&url).send().await?.text().await?;

    let tokens = bzz_stuff::tokenize(text.chars())?;
    let parsed = bzz_stuff::parse(tokens.into_iter())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}
