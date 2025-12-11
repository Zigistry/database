pub mod git_hub {
    use crate::bzz_stuff::{parse, tokenize};
    use crate::constants::POSSIBLE_README_URLS;
    use crate::custom_types::Dependency;
    use std::error::Error;

    pub async fn get_readme_url(
        owner_name: &str,
        repo_name: &str,
        branch_or_tag: &str,
    ) -> Option<String> {
        let name =
            format!("https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/");

        let client = reqwest::Client::new();
        for i in POSSIBLE_README_URLS {
            let mine = name.to_string() + i;
            let res = client.head(&mine).send().await.unwrap();
            if res.status().is_success() {
                return Option::from(mine);
            }
        }
        None
    }
    
    pub async fn get_build_zig_zon_data(
        owner_name: &str,
        repo_name: &str,
        branch_or_tag: &str,
    ) -> Result<(String, Vec<Dependency>), Box<dyn Error>> {
        let url = format!(
            "https://raw.githubusercontent.com/{owner_name}/{repo_name}/{branch_or_tag}/build.zig.zon"
        );
        let client = reqwest::Client::new();
        let text = client.get(&url).send().await?.text().await?;

        let tokens = tokenize(&mut text.chars().collect::<Vec<_>>().into_iter().peekable())?;
        let parsed = parse(&mut tokens.into_iter().peekable())?;

        Ok((parsed.minimum_zig_version, parsed.dependencies))
    }
}

pub mod code_berg {}
