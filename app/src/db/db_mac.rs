use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, SqlitePool};

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
    pub(crate) parent_note_id: i64,
    pub(crate) textarea_id: i64,
    pub(crate) textarea_row: i64,
    pub(crate) start_col: i64,
    pub(crate) end_col: i64,
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
        title: String,
        body: String,
        has_links: bool,
    ) -> Result<i64> {
        let result = sqlx::query!(
            "INSERT INTO notes (title, body, has_links) VALUES (?,?,?) RETURNING id",
            title,
            body,
            has_links
        )
        .fetch_one(db)
        .await;

        match result {
            Ok(row) => Ok(row.id),
            Err(e) => Err(eyre!("Failed to save note: {:?}", e)),
        }
    }

    pub(crate) async fn save_links(
        db: &SqlitePool,
        links: Vec<DbNoteLink>,
        parent_note_id: i64,
    ) -> Result<()> {
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO links 
                (textarea_id, textarea_row, start_col, end_col, parent_note_id, linked_note_id) ",
        );

        query_builder.push_values(links.into_iter(), |mut b, link| {
            b.push_bind(link.textarea_id)
                .push_bind(link.textarea_row)
                .push_bind(link.start_col)
                .push_bind(link.end_col)
                .push_bind(parent_note_id)
                .push_bind(link.linked_note_id);
        });

        let query = query_builder.build();

        let result = query.execute(db).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to save links: {:?}", e)),
        }
    }

    pub(crate) async fn update_note(
        db: &SqlitePool,
        title: String,
        body: String,
        has_links: bool,
        id: i64,
    ) -> Result<i64> {
        let result = sqlx::query!(
            "UPDATE notes SET title=?, body=?, has_links=? WHERE id=? RETURNING id",
            title,
            body,
            has_links,
            id
        )
        .fetch_one(db)
        .await;

        match result {
            Ok(row) => {
                let id = row.id.expect("Updated note should be in database");
                Ok(id)
            }
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
            "SELECT
                parent_note_id, textarea_id, textarea_row, start_col, end_col, linked_note_id 
            FROM 
                links 
            WHERE 
                parent_note_id=?",
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

    pub(crate) async fn delete_link(db: &SqlitePool, parent_note_id: i64, textarea_id: i64) -> Result<()> {
        let result = sqlx::query!("
            DELETE FROM links 
            WHERE 
                parent_note_id=$1 
            AND 
                textarea_id=$2",
            parent_note_id,
            textarea_id
        )
        .execute(db)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to delete link: {:?}", e)),
        }
    }
}
