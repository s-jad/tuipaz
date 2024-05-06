use color_eyre::eyre::eyre;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

use crate::tui::errors::DbError;

pub(crate) async fn create_db(db_url: PathBuf) -> Result<SqlitePool, DbError> {
    let db_url_str = db_url
        .to_str()
        .ok_or_else(|| create_db_error("Failed to convert db_url to db_url_str".to_string()))?;

    let db = SqlitePool::connect(&db_url_str)
        .await
        .map_err(|e| create_db_error(e.to_string()))?;

    Ok(db)
}

fn create_db_error(msg: String) -> DbError {
    let err = eyre!(msg);
    DbError::from(err)
}
