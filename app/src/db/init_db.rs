use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use crate::tui::errors::{create_db_error, DbError};

pub(crate) async fn create_db() -> Result<SqlitePool, DbError> {
    let opts = SqliteConnectOptions::new()
        .filename("notes.db")
        .create_if_missing(true);

    let conn = SqlitePool::connect_with(opts)
        .await
        .map_err(|e| create_db_error(e.to_string()))?;

    let _schema_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL UNIQUE,
            body TEXT,
            links TEXT
        );",
    )
    .execute(&conn)
    .await?;

    Ok(conn)
}
