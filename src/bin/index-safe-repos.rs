use libsql::{Connection, params};
use zigistry::{
    CODEBERG_KEY, GITHUB_KEY,
    constants::GH_GRAPH_QL_100_REPOS_FRAGMENT,
    constants::limits::{INDEX_SECTION_NAME_MAX_LEN, REPO_ID_MAX_LEN},
    database::{connect_to_database, truncate_to_char_limit},
};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let connection = connect_to_database().await.unwrap();

    loop {
        let mut rows_to_process = connection
            // I am doing order by, because when I will delete these, it will not delete something else,
            // when queries from database again.
            .query(
                "SELECT id FROM safe_to_index_new_repo ORDER BY id ASC LIMIT 50;",
                params![],
            )
            .await
            .unwrap();

        {
            while let Some(repo_row) = rows_to_process.next().await.unwrap() {
                let id: String = row.get(0).unwrap();
                let type_of_repo: String = row.get(1).unwrap();

                id.split('/')

                if id
            }
        }
        let rows_to_process = connection
            .query(
                "DELETE FROM safe_to_index_new_repo
         WHERE id IN (SELECT id FROM safe_to_index_new_repo ORDER BY id ASC LIMIT 50)
         RETURNING id, type_of_repo",
                params![],
            )
            .await
            .unwrap();
    }
}
