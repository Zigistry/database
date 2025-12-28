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
mod github;

use chrono::Utc;
use lazy_static::lazy_static;
use std::{collections::HashMap, env, error::Error};
use tokio::sync::Mutex;

lazy_static! {
    static ref GITHUB_KEY: String =
        "Bearer ".to_string() + &env::var("GH_API_KEY").expect("GH_API_KEY not set");
    static ref CODEBERG_KEY: String =
        "token ".to_string() + &env::var("CB_API_KEY").expect("CB_API_KEY not set");
    static ref DATABASE: Mutex<custom_types::Root> = Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    });
}

#[macro_export]
macro_rules! db {
    () => {
        &mut *crate::DATABASE.lock().await
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("Starting");
    let timer_start = Utc::now();
    match codeberg::codeberg_main().await {
        Ok(_) => {
            eprintln!(
                "Codeberg completed successfully in {}minutes.",
                (Utc::now() - timer_start).num_minutes(),
            )
        }
        Err(r) => {
            let json = serde_json::to_string(db!())?;
            eprintln!("{json}");
            eprintln!("Codeberg gave this error:");
            eprintln!("{:#?}", r);
        }
    }
    let timer_start = Utc::now();
    match github::github_main().await {
        Ok(_) => {
            eprintln!(
                "GitHub completed successfully in {}minutes.",
                (Utc::now() - timer_start).num_minutes(),
            )
        }
        Err(r) => {
            let json = serde_json::to_string(db!())?;
            eprintln!("{json}");
            eprintln!("GitHub gave this error:");
            eprintln!("{:#?}", r);
        }
    }
    let json = serde_json::to_string(db!())?;
    println!("{json}");

    Ok(())
}
