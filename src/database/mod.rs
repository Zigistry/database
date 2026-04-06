use dotenv::dotenv;
use keyword_extraction::rake::{Rake, RakeParams};
use libsql::{Builder, Connection};
use std::env;

const START_FROM_SCRATCH: bool = false;

const DATABASE_SCHEMA: &str = include_str!("../../Database_SQL_Files/database_schema.sql");

pub fn truncate_to_char_limit(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect()
}

pub fn truncate_option_to_char_limit(value: Option<&str>, max_chars: usize) -> Option<String> {
    value.map(|v| truncate_to_char_limit(v, max_chars))
}

pub fn parse_lazy_flag(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("true")
}

pub fn utc_now_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn extract_keywords_from_text_and_return_as_string(text: &str, max_keywords: usize) -> String {
    let trimmed_text = text.trim();
    if trimmed_text.is_empty() {
        return String::new();
    }

    let rake = Rake::new(RakeParams::WithDefaults(
        trimmed_text,
        &crate::stop_words_in_eng,
    ));
    rake.get_ranked_keyword(max_keywords).join(" ")
}

pub fn merge_search_keywords(readme_keywords: &str, description: Option<&str>) -> String {
    let description_keywords = description
        .map(|text| extract_keywords_from_text_and_return_as_string(text, 50))
        .unwrap_or_default();

    let readme_trimmed = readme_keywords.trim();
    let description_keywords_trimmed = description_keywords.trim();

    if readme_trimmed == "" && description_keywords_trimmed == "" {
        return String::new();
    }

    if readme_trimmed == "" {
        return description_keywords_trimmed.to_string();
    }

    if description_keywords_trimmed == "" {
        return readme_trimmed.to_string();
    }

    format!("{readme_trimmed} {description_keywords_trimmed}")
}

pub async fn connect_to_database() -> Result<Connection, Box<dyn std::error::Error>> {
    dotenv().ok();
    let db = Builder::new_remote_replica(
        "./database.db",
        env::var("DATABASE_URL").expect("DATABASE_URL not found"),
        env::var("API_KEY").expect("API_KEY not found"),
    );
    let client = db.build().await.unwrap();
    let pool = client.connect().unwrap();
    if START_FROM_SCRATCH {
        pool.execute_batch(DATABASE_SCHEMA).await.unwrap();
    }
    Ok(pool)
}
