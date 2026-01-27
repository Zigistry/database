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
pub mod dependents_calculator;
pub mod github;
pub mod sections;

use lazy_static::lazy_static;
use std::env;
use stop_words::{LANGUAGE, get};

lazy_static! {
    pub static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    pub static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
    pub static ref stop_words_in_eng: Vec<String> = get(LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();
}
