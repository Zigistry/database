pub mod bzz_stuff;
pub mod codeberg;
pub mod constants;
pub mod custom_types;
pub mod database;
pub mod dependents_calculator;
pub mod github;
pub mod sections;

use chrono::{NaiveDate, Utc};
use dependents_calculator::calculate_dependents;
use lazy_static::lazy_static;
use std::{env, error::Error};
use stop_words::{LANGUAGE, get};

lazy_static! {
    static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
    static ref stop_words_in_eng: Vec<String> = get(LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();
}
