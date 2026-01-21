use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, error::Error};
use zigistry::codeberg;
use zigistry::database;
use zigistry::dependents_calculator::calculate_dependents;
use zigistry::github::{self, process_repository};
use zigistry::sections::fetch_repos_for_sections;

#[derive(Debug, Deserialize)]
struct Config {
    manual_additions: Vec<String>,
    sections: HashMap<String, Vec<String>>,
}
pub fn main() {
    let toml_content = std::fs::read_to_string("sections.toml").unwrap();
    let config: Config = toml::from_str(&toml_content).unwrap();

    for repo in config.manual_additions {
        process_repository(repository, is_package, pool);
    }
    println!("{:?}", config);
}
