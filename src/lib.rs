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

pub mod bzz_stuff;
pub mod codeberg;
pub mod constants;
pub mod custom_types;
pub mod database;
pub mod github;

use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    pub static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
}

pub fn keyword_extraction(
    readme_content: &str,
    description: &str,
    owner_name: &str,
    repo_name: &str,
) {
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(include_str!("./readme_extraction/extraction.py"))
        .env("DOC", format!("{readme_content} {description}"))
        .output()
        .unwrap();

    let text = String::from_utf8_lossy(&output.stdout);

    println!("{text}");
}
