mod config;
use chrono::{Days, Local, Months};
mod types;
use futures::future::join_all;
use std::{collections::HashMap, env};
use tokio::sync::Mutex;
mod constants;
mod custom_types;
mod helper_functions;
mod codeberg;
use crate::codeberg::codeberg_main;

use crate::{github::github_main, helper_functions::*};
use lazy_static::lazy_static;
mod codeberg_process_release;
mod bzz_stuff;
mod github;

type GenericErr = Box<dyn std::error::Error>;

lazy_static! {
    static ref KEY: String = "Bearer ".to_string()
        + &env::var("GH_API_KEY")
            .expect("GH_API_KEY not set")
            .to_string();
    static ref GLOBAL: Mutex<custom_types::Root> = Mutex::new(custom_types::Root {
        users: HashMap::new(),
        packages: HashMap::new(),
        programs: HashMap::new(),
    });
}

#[tokio::main]
async fn main() -> Result<(), GenericErr> {
    eprintln!("Starting");
    async {
        github_main();
        codeberg_main();
    }.await;
    // println!("{}", &GLOBAL.lock().await.packages.len());
    Ok(())
}
