use keyword_extraction::rake::{Rake, RakeParams};

use crate::CODEBERG_KEY;
use crate::bzz_stuff;
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::constants::limits;
use crate::custom_types;
use crate::database::truncate_to_char_limit;

pub async fn get_readme_url(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag: bool,
    process_keywords: bool,
) -> (String, String) {
    let branch_or_tag_value = if is_tag { "tag" } else { "branch" };
    let url = format!(
        "https://codeberg.org/{owner_name}/{repo_name}/raw/{branch_or_tag_value}/{branch_or_tag}/"
    );

    let client = reqwest::Client::new();

    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let readme_possible_url = url.clone() + readme_file_name;

        let responce = match client.head(&readme_possible_url).send().await {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Problem:{err}");
                continue;
            }
        };
        if responce.status().is_success() {
            if process_keywords {
                let res = match client.get(&readme_possible_url).send().await {
                    Ok(t) => t,
                    Err(_) => {
                        print!("skipping readme {owner_name}/{repo_name}");
                        continue;
                    }
                };
                if res.status().is_success() {
                    let rake = Rake::new(RakeParams::WithDefaults(
                        &res.text().await.unwrap(),
                        &crate::stop_words_in_eng,
                    ));
                    // Afaik, 200 keywords is overkill.
                    let mut keywords = rake.get_ranked_keyword(200);
                    keywords.push(owner_name.to_string());
                    keywords.push(repo_name.to_string());
                    let keyword_string = keywords.join(" ");
                    return (readme_possible_url, keyword_string);
                }
            }
            return (readme_possible_url, String::new());
        }
    }

    (String::new(), String::new())
}

pub async fn get_build_zig_zon_data(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag: bool,
) -> Result<(String, Vec<custom_types::Dependency>), Box<dyn std::error::Error>> {
    let branch_or_tag_value = if is_tag { "tag" } else { "branch" };
    let url = format!(
        "https://codeberg.org/{owner_name}/{repo_name}/raw/{branch_or_tag_value}/{branch_or_tag}/build.zig.zon"
    );

    let client = reqwest::Client::new();
    let text = client.get(&url).send().await?.text().await?;

    let tokens = bzz_stuff::tokenize(text.chars())?;
    let parsed = bzz_stuff::parse(tokens.into_iter())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}

pub async fn get_latest_commit_hash(owner_name: &str, repo_name: &str) -> String {
    let url = format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/commits?limit=1");

    let response = match reqwest::Client::new()
        .get(url)
        .header("Authorization", &*CODEBERG_KEY)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response,
        _ => return "unknown".to_string(),
    };

    let response_json = match response.json::<serde_json::Value>().await {
        Ok(value) => value,
        Err(_) => return "unknown".to_string(),
    };

    response_json
        .get(0)
        .and_then(|commit| commit.get("sha"))
        .and_then(|sha| sha.as_str())
        .map(str::trim)
        .filter(|sha| !sha.is_empty())
        .map(|sha| truncate_to_char_limit(sha, limits::REPO_COMMIT_HASH_MAX_LEN))
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn has_zig_in_top_languages(owner_name: &str, repo_name: &str) -> bool {
    let url = format!("https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/languages");

    let response = match reqwest::Client::new()
        .get(url)
        .header("Authorization", &*CODEBERG_KEY)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response,
        _ => return false,
    };

    let response_json = match response.json::<serde_json::Value>().await {
        Ok(value) => value,
        Err(_) => return false,
    };

    let mut languages: Vec<(&str, u64)> = response_json
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(name, bytes)| bytes.as_u64().map(|size| (name.as_str(), size)))
                .collect()
        })
        .unwrap_or_default();

    languages.sort_by(|a, b| b.1.cmp(&a.1));
    languages
        .into_iter()
        .take(10)
        .any(|(name, _)| name.eq_ignore_ascii_case("zig"))
}
