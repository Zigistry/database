use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub type Root = Vec<Root2>;
