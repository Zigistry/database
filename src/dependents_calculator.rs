use libsql::Connection;

const DEPENDENTS_CALCULATOR_QUERY: &str =
    include_str!("../Database_SQL_Files/dependents_calculator.sql");

pub async fn calculate_dependents(pool: &Connection) {
    // I will now find the gh/thingy/thingy pattern
    // Ex: github.com/zigzap/zap
    pool.execute(DEPENDENTS_CALCULATOR_QUERY, ()).await.unwrap();
}
