pub mod git_hub {
    pub async fn get_readme_url(repo_full_name: &str, branch_or_tag: &str) -> String {
        format!("https://github.com/{}/readme", branch_or_tag);
    }
    pub async fn get_build_zig_zon_data(repo_full_name: &str, branch_or_tag: &str) -> String {
        "".to_string()
    }
}

pub mod code_berg {}
