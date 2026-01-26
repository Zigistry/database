use keyword_extraction::rake::{Rake, RakeParams};

use crate::bzz_stuff;
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::custom_types;

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
                    let keywords = rake.get_ranked_keyword(200);
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
