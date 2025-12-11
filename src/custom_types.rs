use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct Asset {
    pub download_url: String,
    pub size: i64,
    pub content_type: String,
}
/// This is all the data I get from build.zig.zonn
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Dependency {
    pub name: String,
    pub hash: String,
    pub lazy: String,
    pub url: String,
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Release {
    pub is_prerelease: bool,
    pub published_at: String,
    pub release_assets: HashMap<String, Asset>,
    pub dependencies: Vec<Dependency>,
    pub minimum_zig_version: String,
    pub readme_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Repo {
    pub description: String,
    pub issues_count: u32,
    pub default_branch: String,
    pub fork_count: u32,
    pub stargazer_count: u32,
    pub watchers_count: u32,
    pub pushed_at: String,
    pub created_at: String,
    pub is_archived: bool,
    pub is_disabled: bool,
    pub is_fork: bool,
    pub license: String,
    pub repository_topics: Vec<String>,
    pub releases: HashMap<String, Release>,
    // the forexample main branch or master branch.
    // the best way I can say is HEAD branch.
    pub default_branch_information: Release,
    pub primary_language: String,
}

#[derive(Debug)]
pub struct User {
    pub avatar_url: String,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub followers: u32,
    pub following: u32,
    pub location: Option<String>,
    pub description: Option<String>,
    pub website_url: Option<String>,
}

pub struct Root {
    /// Here the String will be:
    /// gh/rohanvashisht1234
    /// cb/rohanvashisht1234
    pub users: HashMap<String, User>,
    /// Here the String will be:
    /// gh/rohanvashisht1234/zorsig
    /// cb/rohanvashisht1234/repo_name
    pub packages: HashMap<String, Repo>,
    /// Here the String will be:
    /// gh/rohanvashisht1234/zorsig
    /// cb/rohanvashisht1234/repo_name
    pub programs: HashMap<String, Repo>,
}
