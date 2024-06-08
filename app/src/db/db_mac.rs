use color_eyre::eyre::{eyre, Result};
use log::info;
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

    pub(crate) async fn delete_note(db: &SqlitePool, note_id: i64) -> Result<()> {
        let delete_links_result = sqlx::query!("DELETE FROM links WHERE parent_note_id=? OR linked_note_id=?", note_id, note_id)
           .execute(db)
           .await;
    
        match delete_links_result{
            Ok(_) => {
                let delete_note_result = sqlx::query!("DELETE FROM notes WHERE id=?", note_id).execute(db).await;

                match delete_note_result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(eyre!("Failed to delete note: {:?}", e))

                }
            },
            Err(e) => Err(eyre!("Failed to delete links attached to deleted note: {:?}", e))
        }
    }

    pub(crate) async fn update_links(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, 
        db: &SqlitePool,
        links: Vec<DbNoteLink>,
        parent_note_id: i64,
    ) -> Result<()> {
        for link in links {
            let result = sqlx::query!(
                "UPDATE links 
                SET textarea_row =?, start_col =?, end_col =?
                WHERE parent_note_id =? AND linked_note_id =? AND textarea_id =?;",
                link.textarea_row, 
                link.start_col, 
                link.end_col, 
                parent_note_id, 
                link.linked_note_id, 
                link.textarea_id,
            )
            .execute(db)
            .await;

            if let Err(e) = result {
                return Err(eyre!("Failed to update link: {:?}", e))
            }

        }
        Ok(())
    }

    pub(crate) async fn save_links(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, 
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

    pub(crate) async fn delete_links(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, 
        db: &SqlitePool, 
        link_identifiers: Vec<(i64, i64)>
    ) -> Result<()> {
        let where_clause = link_identifiers.iter().map(|&(parent_note_id, textarea_id)| {
            format!("(parent_note_id ={} AND textarea_id ={})", parent_note_id, textarea_id)
        }).collect::<Vec<_>>().join(" OR ");

        let query_str = format!(r#"DELETE FROM links WHERE {}"#, where_clause);
        
        let result = sqlx::query(&query_str)
            .execute(db)  
            .await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Failed to delete link: {:?}", e)),
        }
    }
}
