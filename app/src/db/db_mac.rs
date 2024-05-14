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
pub(crate) struct DbNoteLink {
    pub(crate) start_x: i64,
    pub(crate) start_y: i64,
    pub(crate) end_x: i64,
    pub(crate) end_y: i64,
    pub(crate) linked_note_id: i64,
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
    pub(crate) has_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotePatch {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) has_links: bool,
}

#[derive(Debug)]
pub(crate) struct DbMac;

impl DbMac {
    pub(crate) async fn save_note(
        db: &SqlitePool,
        body: String,
        title: String,
        has_links: bool,
    ) -> Result<()> {
        let result = sqlx::query!(
            "INSERT INTO notes (title, body, has_links) VALUES (?,?,?)",
            title,
            body,
            has_links
        )
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
        has_links: bool,
        id: i64,
    ) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE notes SET title=?, body=?, has_links=? WHERE id=?",
            title,
            body,
            has_links,
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
        let result = sqlx::query_as!(
            Note,
            "SELECT id, title, COALESCE(body, '') AS body, has_links FROM notes WHERE id=? ",
            id
        )
        .fetch_one(db)
        .await;

        match result {
            Ok(note) => Ok(note),
            Err(e) => Err(eyre!("Failed to load note: {:?}", e)),
        }
    }

    pub(crate) async fn load_note_links(
        db: &SqlitePool,
        parent_note_id: i64,
    ) -> Result<Vec<DbNoteLink>> {
        let result = sqlx::query_as!(
            DbNoteLink,
            "SELECT start_x, start_y, end_x, end_y, linked_note_id FROM links WHERE parent_note_id=?",
            parent_note_id
        )
        .fetch_all(db)
        .await;

        match result {
            Ok(links) => Ok(links),
            Err(e) => Err(eyre!("Failed to load note links: {:?}", e)),
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
