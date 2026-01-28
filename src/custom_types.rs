use serde_with::skip_serializing_none;
use std::collections::HashMap;

/// This is all the data I get from build.zig.zonn
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "h")]
    pub hash: String,
    #[serde(rename = "l")]
    pub lazy: String,
    #[serde(rename = "u")]
    pub url: String,
    #[serde(rename = "p")]
    pub path: String,
}

#[skip_serializing_none]
#[derive(Debug, serde::Serialize, Default)]
pub struct Release {
    #[serde(rename = "h")]
    pub is_prerelease: bool,
    #[serde(rename = "p")]
    pub published_at: String,
    #[serde(rename = "d")]
    pub dependencies: Vec<Dependency>,
    #[serde(rename = "m")]
    pub minimum_zig_version: String,
    #[serde(rename = "r")]
    pub readme_url: String,
}

#[skip_serializing_none]
#[derive(Debug, serde::Serialize)]
pub struct Repo {
    #[serde(rename = "a")]
    pub avatar_id: String,
    #[serde(rename = "d")]
    pub description: Option<String>,
    #[serde(rename = "i")]
    pub issues_count: u32,
    #[serde(rename = "db")]
    pub default_branch: String,
    #[serde(rename = "fc")]
    pub fork_count: u32,
    #[serde(rename = "s")]
    pub stargazer_count: u32,
    #[serde(rename = "w")]
    pub watchers_count: u32,
    #[serde(rename = "p")]
    pub pushed_at: String,
    #[serde(rename = "c")]
    pub created_at: String,
    #[serde(rename = "ia")]
    pub is_archived: bool,
    #[serde(rename = "id")]
    pub is_disabled: bool,
    #[serde(rename = "if")]
    pub is_fork: bool,
    #[serde(rename = "l")]
    pub license: String,
    #[serde(rename = "t")]
    pub repository_topics: Vec<String>,
    #[serde(rename = "r")]
    pub releases: HashMap<String, Release>,
    // the forexample main branch or master branch.
    // the best way I can say is HEAD branch.
    #[serde(rename = "dbi")]
    pub default_branch_information: Release,
    #[serde(rename = "pl")]
    pub primary_language: String,
    #[serde(rename = "dts")]
    pub dependents: Vec<String>,
}

#[skip_serializing_none]
#[derive(Debug, serde::Serialize)]
pub struct User {
    #[serde(rename = "a")]
    pub avatar_id: String,
    #[serde(rename = "b")]
    pub bio: Option<String>,
    #[serde(rename = "c")]
    pub company: Option<String>,
    #[serde(rename = "f")]
    pub followers: u32,
    #[serde(rename = "fg")]
    pub following: u32,
    #[serde(rename = "l")]
    pub location: Option<String>,
    #[serde(rename = "d")]
    pub description: Option<String>,
    #[serde(rename = "w")]
    pub website_url: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, serde::Serialize)]
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
    /// This is only for the packages section
    /// display on the index page.
    pub index_sections: HashMap<String, Vec<String>>,
}

// OK, From here the discussion begins of what will
// be the future of Zigistry's database.
// Since the past ~2 years, I have been using json as the primary
// way, even though it works very cool, the issue is that json
// is not something I can conveniently use for Database management.
// I mean, its not the best for dbms. The best solution I can think
// of is using sqlite. SQL lite.
// The thing is that the search
// implementation and everything from json generation is just too much.
// also, the concept of having asset url and asset download didn't really
// make much of a sence.
// So, the main issue comes to when:
// This database is currently 6 MB, and me using clone in rust makes the memory
// usage much worse. Using SQL lite would make things in a way that a huge
// amount of memory would not be used.
// Currently its just 5000 repos, still its taking an hour and
// a half to index all this. Now, maybe switching to SQL lite would make stuff
// atleast easier to maintain, increasing a huge amount of possibility to improve.
// Hence, I am defining the sql schema I can think of:
// Ok, so a single table would be there for all repos.
