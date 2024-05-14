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
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            title TEXT NOT NULL UNIQUE,
            body TEXT,
            has_links BOOL NOT NULL
        );",
    )
    .execute(&conn)
    .await?;

    // Create the links table if it doesn't exist
    let _links_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            start_x INTEGER NOT NULL,
            start_y INTEGER NOT NULL,
            end_x INTEGER NOT NULL,
            end_y INTEGER NOT NULL,
            text TEXT NOT NULL,
            parent_note_id INTEGER NOT NULL,
            linked_note_id INTEGER NOT NULL,
            FOREIGN KEY(parent_note_id) REFERENCES notes(id),
            FOREIGN KEY(linked_note_id) REFERENCES notes(id)
        );",
    )
    .execute(&conn)
    .await?;

    Ok(conn)
}
