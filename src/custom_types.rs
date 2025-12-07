use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub name: String,
    pub avatar_url: String,
    pub html_url: String,
    pub type_field: String, // afaik, this can be of type "User" or "Organization"
}


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Repo {
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub users: HashMap<String, User>,
    pub packages: HashMap<String, Repo>,
    pub programs: HashMap<String, Repo>,
}
