use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(
    sqlx::
    FromRow,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]

pub(crate) struct Note {
    pub(crate) id: i64,
    pub(crate) note: String,
}

#[derive(Debug)]
pub(crate) struct NotesMac;

impl NotesMac {
    pub(crate) async fn save_note(db: &SqlitePool, note: String) -> Result<()> {
        let result = sqlx::query!("INSERT INTO notes (content) VALUES (?)", note)
            .execute(db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to save note: {:?}", e)),
        }
    }
}
