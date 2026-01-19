/*!
 * SPDX-License-Identifier: AGPL-3.0-only WITH LicenseRef-Zigistry-Database-Permission
 *
 * Copyright (c) 2025 Rohan Vashisht
 *
 * This software is licensed under the GNU Affero General Public License v3.0.
 * The database content under `zigistry/database` is subject to additional terms.
 *
 *      ______       _     _
 *     |__  (_) __ _(_)___| |_ _ __ _   _
 *       / /| |/ _` | / __| __| '__| | | |
 *      / /_| | (_| | \__ \ |_| |  | |_| |
 *     /____|_|\__, |_|___/\__|_|   \__, |
 *             |___/                |___/
 *
 *
 * See LICENSE and LICENSE-ADDITIONAL in the project root directory for full details.
 */
mod bzz_stuff;
mod codeberg;
mod constants;
mod custom_types;
mod database;
mod dependents_calculator;
mod github;
mod sections;

use chrono::Utc;
use dependents_calculator::calculate_dependents;
use lazy_static::lazy_static;
use std::{env, error::Error};
use stop_words::{LANGUAGE, get};

use crate::sections::fetch_repos_for_sections;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = database::init_database().await.unwrap();
    eprintln!("Starting");
    let timer_start = Utc::now();
    codeberg::codeberg_main(&pool).await.unwrap();
    eprintln!(
        "Codeberg completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );

    github::github_main(&pool).await.unwrap();
    eprintln!(
        "GitHub completed successfully in {}minutes.",
        (Utc::now() - timer_start).num_minutes(),
    );
    let timer_start = Utc::now();
    fetch_repos_for_sections(&pool).await.unwrap();
    eprintln!(
        "Sections completed successfully in {} minutes",
        (Utc::now() - timer_start).num_minutes()
    );
    calculate_dependents(&pool).await;
    database::wrap_up(&pool).await.unwrap();
    Ok(())
}
