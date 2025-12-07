use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub avatar_url: String,
    pub profile_link: String,
    pub type_field: String,
    pub followers: i64,
    pub following: i64,
    pub email: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub company: Value,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Dependency {
    pub name: String,
    pub hash: String,
    pub lazy: bool,
    pub url: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tag {
    pub readme_url: String,
    pub last_updated: String,
    pub minimum_zig_version: String,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Repo {
    pub topics: Vec<String>,
    pub stars_count: u32,
    pub forks_count: u32,
    pub watchers_count: u32,
    pub pr_count: u32,
    pub forked: bool,
    pub issues: u32,
    pub description: String,
    pub dependents: Vec<String>,
    pub unstable_current_repo_head_branch: Tag,
    pub tags: HashMap<String, Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub users: HashMap<String, User>,
    pub packages: HashMap<String, Repo>,
    pub programs: HashMap<String, Repo>,
}
