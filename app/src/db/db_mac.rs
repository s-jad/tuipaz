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
pub(crate) struct NoteIdentifier {
    pub(crate) id: i64,
    pub(crate) title: String,
}

impl ToString for NoteIdentifier {
    fn to_string(&self) -> String {
        self.title.clone()
    }
}

#[derive(
    sqlx::
    FromRow,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
pub(crate) struct Note {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) body: Option<String>,
    pub(crate) links: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotePatch {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) links: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DbMac;

impl DbMac {
    pub(crate) async fn save_note(db: &SqlitePool, body: String, title: String) -> Result<()> {
        let result = sqlx::query!("INSERT INTO notes (title, body) VALUES (?,?)", title, body)
            .execute(db)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to save note: {:?}", e)),
        }
    }

    pub(crate) async fn update_note(
        db: &SqlitePool,
        title: String,
        body: String,
        id: i64,
    ) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE notes SET title=?, body=? WHERE id=?",
            title,
            body,
            id
        )
        .execute(db)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to save note: {:?}", e)),
        }
    }

    pub(crate) async fn load_note(db: &SqlitePool, id: i64) -> Result<Note> {
        //panic!("title: {title:?}");
        let result = sqlx::query_as!(
            Note,
            "SELECT id, title, COALESCE(body, '') AS body, COALESCE(links, '') AS links FROM notes WHERE id=? ",
            id
        )
        .fetch_one(db)
        .await;

        // panic!("load_note result: {result:?}");

        match result {
            Ok(note) => Ok(note),
            Err(e) => Err(eyre!("Failed to load note: {:?}", e)),
        }
    }

    pub(crate) async fn load_note_identifiers(db: &SqlitePool) -> Result<Vec<NoteIdentifier>> {
        let result = sqlx::query_as!(NoteIdentifier, "SELECT id, title FROM notes")
            .fetch_all(db)
            .await;

        match result {
            Ok(notes) => Ok(notes),
            Err(e) => Err(eyre!("Failed to load note identifiers: {:?}", e)),
        }
    }
}
