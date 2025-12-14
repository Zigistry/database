// Code berg types
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub ok: bool,
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub id: i64,
    pub owner: Owner,
    pub name: String,
    #[serde(rename = "full_name")]
    pub full_name: String,
    pub description: String,
    pub empty: bool,
    pub private: bool,
    pub fork: bool,
    pub template: bool,
    pub parent: Value,
    pub mirror: bool,
    pub size: i64,
    pub language: String,
    #[serde(rename = "languages_url")]
    pub languages_url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub url: String,
    pub link: String,
    #[serde(rename = "ssh_url")]
    pub ssh_url: String,
    #[serde(rename = "clone_url")]
    pub clone_url: String,
    #[serde(rename = "original_url")]
    pub original_url: String,
    pub website: String,
    #[serde(rename = "stars_count")]
    pub stars_count: u32,
    #[serde(rename = "forks_count")]
    pub forks_count: u32,
    #[serde(rename = "watchers_count")]
    pub watchers_count: u32,
    #[serde(rename = "open_issues_count")]
    pub open_issues_count: u32,
    #[serde(rename = "open_pr_counter")]
    pub open_pr_counter: u32,
    #[serde(rename = "release_counter")]
    pub release_counter: u32,
    #[serde(rename = "default_branch")]
    pub default_branch: String,
    pub archived: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "archived_at")]
    pub archived_at: String,
    pub permissions: Permissions,
    #[serde(rename = "has_issues")]
    pub has_issues: bool,
    #[serde(rename = "internal_tracker")]
    pub internal_tracker: Option<InternalTracker>,
    #[serde(rename = "has_wiki")]
    pub has_wiki: bool,
    #[serde(rename = "wiki_branch")]
    pub wiki_branch: Option<String>,
    #[serde(rename = "globally_editable_wiki")]
    pub globally_editable_wiki: bool,
    #[serde(rename = "has_pull_requests")]
    pub has_pull_requests: bool,
    #[serde(rename = "has_projects")]
    pub has_projects: bool,
    #[serde(rename = "has_releases")]
    pub has_releases: bool,
    #[serde(rename = "has_packages")]
    pub has_packages: bool,
    #[serde(rename = "has_actions")]
    pub has_actions: bool,
    #[serde(rename = "ignore_whitespace_conflicts")]
    pub ignore_whitespace_conflicts: bool,
    #[serde(rename = "allow_merge_commits")]
    pub allow_merge_commits: bool,
    #[serde(rename = "allow_rebase")]
    pub allow_rebase: bool,
    #[serde(rename = "allow_rebase_explicit")]
    pub allow_rebase_explicit: bool,
    #[serde(rename = "allow_squash_merge")]
    pub allow_squash_merge: bool,
    #[serde(rename = "allow_fast_forward_only_merge")]
    pub allow_fast_forward_only_merge: bool,
    #[serde(rename = "allow_rebase_update")]
    pub allow_rebase_update: bool,
    #[serde(rename = "default_delete_branch_after_merge")]
    pub default_delete_branch_after_merge: bool,
    #[serde(rename = "default_merge_style")]
    pub default_merge_style: String,
    #[serde(rename = "default_allow_maintainer_edit")]
    pub default_allow_maintainer_edit: bool,
    #[serde(rename = "default_update_style")]
    pub default_update_style: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    pub internal: bool,
    #[serde(rename = "mirror_interval")]
    pub mirror_interval: String,
    #[serde(rename = "object_format_name")]
    pub object_format_name: String,
    #[serde(rename = "mirror_updated")]
    pub mirror_updated: String,
    #[serde(rename = "repo_transfer")]
    pub repo_transfer: Value,
    pub topics: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
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
    pub followers_count: u32,
    #[serde(rename = "following_count")]
    pub following_count: u32,
    #[serde(rename = "starred_repos_count")]
    pub starred_repos_count: u32,
    pub username: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub admin: bool,
    pub push: bool,
    pub pull: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalTracker {
    #[serde(rename = "enable_time_tracker")]
    pub enable_time_tracker: bool,
    #[serde(rename = "allow_only_contributors_to_track_time")]
    pub allow_only_contributors_to_track_time: bool,
    #[serde(rename = "enable_issue_dependencies")]
    pub enable_issue_dependencies: bool,
}
