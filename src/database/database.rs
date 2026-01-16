use sqlx::Executor;
use sqlx::{self, Pool, Sqlite};

use crate::databaseSchema::INIT_REPOS;

pub fn init_database(db: Pool<Sqlite>) {
    db.execute(INIT_USERS);
}
