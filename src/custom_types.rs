use std::collections::HashMap;

pub struct Asset {
    pub download_url: String,
    pub size: i64,
    pub content_type: String,
}
/// This is all the data I get from build.zig.zonn
pub struct Dependency {
    pub hash: String,
    pub lazy: bool,
    pub url: String,
    pub path: String,
}
pub struct Release {
    pub is_prerelease: bool,
    pub published_at: String,
    pub release_assets: HashMap<String, Asset>,
    pub dependencies: HashMap<String, Dependency>,
    pub minimum_zig_version: String,
    pub readme_url: String,
}

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
    pub head_branch: Release,
    pub primary_language: String,
}

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
// I will have this for user.json
pub struct UsersJsonRoot {
    /// Here the String will be:
    /// gh/rohanvashisht1234
    /// cb/rohanvashisht1234
    pub users: HashMap<String, User>,
}
// Now, for packages.json
pub struct PackagesJsonRoot {
    /// Here the String will be:
    /// gh/rohanvashisht1234/zorsig
    /// cb/rohanvashisht1234/repo_name
    pub users: HashMap<String, Repo>,
}
// Now, for programs.json
pub struct ProgramsJsonRoot {
    /// Here the String will be:
    /// gh/rohanvashisht1234/zorsig
    /// cb/rohanvashisht1234/repo_name
    pub users: HashMap<String, Repo>,
}
