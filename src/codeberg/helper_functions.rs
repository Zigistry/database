use crate::CODEBERG_KEY;
use crate::bzz_stuff;
use crate::constants::limits;
use crate::custom_types;
use crate::database::truncate_to_char_limit;
use regex::Regex;

pub async fn get_readme_url(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag: bool,
    fetch_content: bool,
    directory_files: &str,
) -> (String, String) {
    if directory_files.is_empty() {
        return (String::new(), String::new());
    }

    let branch_or_tag_value = if is_tag { "tag" } else { "branch" };
    let base_url = format!(
        "https://codeberg.org/{owner_name}/{repo_name}/raw/{branch_or_tag_value}/{branch_or_tag}/"
    );

    let client = reqwest::Client::new();
    // https://regex101.com/?regex=%28%3Fi%29%5Ereadme%5B%5Cw.%5D*%24&testString=readme.md%0Areadme.rs%0AREADME.md%0AReadme.md%0Areadme%2F%0Areadme_something%2F%0A&flags=gm&flavor=pcre2&delimiter=%2F
    let readme_regex = Regex::new(r"(?i)^readme[\w.]*$").unwrap();
    let readme_file_name = directory_files
        .lines()
        .find(|line| readme_regex.is_match(line.trim()));

    match readme_file_name {
        Some(name) => {
            let readme_url = base_url + name;
            if fetch_content {
                let res = match client.get(&readme_url).send().await {
                    Ok(t) if t.status().is_success() => t,
                    _ => return (readme_url, String::new()),
                };
                match res.text().await {
                    Ok(content) => (readme_url, content),
                    Err(_) => (readme_url, String::new()),
                }
            } else {
                (readme_url, String::new())
            }
        }
        None => (String::new(), String::new()),
    }
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

pub async fn fetch_root_folder_directory_files(
    client: &reqwest::Client,
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
) -> String {
    let url = format!(
        "https://codeberg.org/api/v1/repos/{owner_name}/{repo_name}/contents?ref={branch_or_tag}"
    );

    let response = match client
        .get(&url)
        .header("Authorization", &*CODEBERG_KEY)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp,
        _ => return String::new(),
    };

    let response_json: Vec<serde_json::Value> = match response.json().await {
        Ok(val) => val,
        Err(_) => return String::new(),
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for thing in response_json {
        let name = match thing["name"].as_str() {
            Some(name) => name.to_string(),
            None => continue,
        };
        let kind = match thing["type"].as_str() {
            Some(kind) => kind,
            None => continue,
        };

        match kind {
            "dir" => directories.push(name),
            "file" => files.push(name),
            _ => {}
        }
    }

    let dirs_string = directories.join("\n");
    let files_string = files.join("\n");
    let join_both_strings = dirs_string + "\n\n" + &files_string;
    join_both_strings
}
