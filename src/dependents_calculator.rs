use sqlx::SqlitePool;

const DEPENDENTS_CALCULATOR_QUERY: &str =
    include_str!("../Database_SQL_Files/dependents_calculator.sql");

pub async fn calculate_dependents(pool: &SqlitePool) {
    // I will now find the gh/thingy/thingy pattern
    // Ex: github.com/zigzap/zap
    sqlx::query(DEPENDENTS_CALCULATOR_QUERY)
        .execute(pool)
        .await
        .unwrap();
}
