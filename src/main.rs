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
mod codeberg_process_release;
mod constants;
mod custom_types;
mod github;
mod helper_functions;
mod types;

use lazy_static::lazy_static;
use std::{collections::HashMap, env};
use tokio::sync::Mutex;

lazy_static! {
    static ref KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    static ref GLOBAL: Mutex<custom_types::Root> = Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting");
    // github_main().await.unwrap();
    codeberg::codeberg_main().await;
    // println!("{}", &GLOBAL.lock().await.packages.len());
    Ok(())
}
