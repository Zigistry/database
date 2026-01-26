use serde::Deserialize;
use std::collections::HashMap;
use zigistry::github::process_repository;

#[derive(Debug, Deserialize)]
struct Config {
    manual_additions: Vec<String>,
    sections: HashMap<String, Vec<String>>,
}
pub fn main() {
    let toml_content = std::fs::read_to_string("sections.toml").unwrap();
    let config: Config = toml::from_str(&toml_content).unwrap();

    for repo in config.manual_additions {
        process_repository(repo, is_package, pool);
    }
    println!("{:?}", config);
}
