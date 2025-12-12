use std::collections::HashMap;
use std::error::Error;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

use crate::bzz_stuff::parse;
use crate::constants::POSSIBLE_README_FILE_NAMES;
use crate::custom_types;
use crate::custom_types::Dependency;
use lazy_static::lazy_static;
lazy_static! {
    static ref KEY2:String = String::new();
}

pub async fn process_release(owner_name: String, repo_name: String) -> Result<custom_types::Release, Box<dyn std::error::Error>> {
    let release_url = format!(
        "https://codeberg.org/api/v1/repos/{}/{}/releases",
        owner_name, repo_name
    );
    let client = reqwest::Client::new()
        .get(&release_url)
        .bearer_auth(KEY2.to_string())
        .send()
        .await?
        .json::<Root>()
        .await?;
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/README.md
    // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/build.zig.zon
    for i in client {
        let bzz_link = format!("https://codeberg.org/{owner_name}/{repo_name}/raw/tag/{}/build.zig.zon", i.tag_name);
        let bzz_stuff = reqwest::Client::new()
        .get(&bzz_link)
        .send()
        .await?
        .text()
        .await?;

        let compiled_release = custom_types::Release {
            dependencies: vec![],
            is_prerelease:i.prerelease,
            published_at:i.published_at,
            release_assets:HashMap::new(),
            minimum_zig_version:,
            readme_url: get_readme_url(&owner_name, &repo_name, &i.tag_name, true).await,
        };

    }
    
    Ok()
}

pub type Root = Vec<Root2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root2 {
    pub id: i64,
    #[serde(rename = "tag_name")]
    pub tag_name: String,
    #[serde(rename = "target_commitish")]
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "tarball_url")]
    pub tarball_url: String,
    #[serde(rename = "zipball_url")]
    pub zipball_url: String,
    #[serde(rename = "hide_archive_links")]
    pub hide_archive_links: bool,
    #[serde(rename = "upload_url")]
    pub upload_url: String,
    pub draft: bool,
    pub prerelease: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "published_at")]
    pub published_at: String,
    pub author: Author,
    pub assets: Vec<Value>,
    #[serde(rename = "archive_download_count")]
    pub archive_download_count: ArchiveDownloadCount,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: i64,
    pub login: String,
    #[serde(rename = "login_name")]
    pub login_name: String,
    #[serde(rename = "source_id")]
    pub source_id: i64,
    #[serde(rename = "full_name")]
    pub full_name: String,
    pub email: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub language: String,
    #[serde(rename = "is_admin")]
    pub is_admin: bool,
    #[serde(rename = "last_login")]
    pub last_login: String,
    pub created: String,
    pub restricted: bool,
    pub active: bool,
    #[serde(rename = "prohibit_login")]
    pub prohibit_login: bool,
    pub location: String,
    pub pronouns: String,
    pub website: String,
    pub description: String,
    pub visibility: String,
    #[serde(rename = "followers_count")]
    pub followers_count: i64,
    #[serde(rename = "following_count")]
    pub following_count: i64,
    #[serde(rename = "starred_repos_count")]
    pub starred_repos_count: i64,
    pub username: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDownloadCount {
    pub zip: i64,
    #[serde(rename = "tar_gz")]
    pub tar_gz: i64,
}


pub async fn get_readme_url(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag:bool,
) -> String {
    let url = if is_tag {
        format!(
            "https://codeberg.org/{owner_name}/{repo_name}/raw/tag/{branch_or_tag}/"
        )
    } else {
        format!(
            "https://codeberg.org/{owner_name}/{repo_name}/raw/branch/{branch_or_tag}/"
        )
    };

    let client = reqwest::Client::new();

    for readme_file_name in POSSIBLE_README_FILE_NAMES {
        let mine = url.to_string() + readme_file_name;
        let res = client.head(&mine).send().await.unwrap();
        if res.status().is_success() {
            return mine.to_string();
        }
    }

    String::new()
}




pub async fn get_build_zig_zon_data(
    owner_name: &str,
    repo_name: &str,
    branch_or_tag: &str,
    is_tag:bool,
) -> Result<(String, Vec<Dependency>), Box<dyn Error>> {
    let url = if is_tag {
        format!(
            "https://codeberg.org/{owner_name}/{repo_name}/raw/tag/{branch_or_tag}/build.zig.zon"
        )
    } else {
        // https://codeberg.org/FObersteiner/zdt/raw/tag/v0.8.2-zig_0.15/README.md
        format!(
            "https://codeberg.org/{owner_name}/{repo_name}/raw/branch/{branch_or_tag}/build.zig.zon"
        )
    };
   
    let client = reqwest::Client::new();
    let text = client.get(&url).send().await?.text().await?;

    let tokens = crate::bzz_stuff::tokenize(&mut text.chars().collect::<Vec<_>>().into_iter().peekable())?;
    let parsed = parse(&mut tokens.into_iter().peekable())?;

    Ok((parsed.minimum_zig_version, parsed.dependencies))
}

