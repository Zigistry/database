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
use std::fs;
use std::{env, error::Error};

use crate::sections::fetch_repos_for_sections;

lazy_static! {
    static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
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
    match fetch_repos_for_sections(&pool).await {
        Ok(_) => {
            eprintln!("Sections completed successfully.");
        }
        Err(r) => {
            let json = serde_json::to_string(db!())?;
            eprintln!("{json}");
            eprintln!("While indexing sections gave this error:");
            eprintln!("{:#?}", r);
        }
    }
    calculate_dependents().await;
    let database_as_json = serde_json::to_string(db!())?;
    fs::write("database.json", database_as_json)?;
    Ok(())
}
