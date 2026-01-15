use sqlx::Executor;
use sqlx::{self, Pool, Sqlite};

pub fn init_database(db: Pool<Sqlite>) {
    db.execute("");
}
