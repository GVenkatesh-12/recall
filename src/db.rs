use crate::models::Note;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// Resolve the path for the database folder (~/.recall)
pub fn get_db_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().context("Could not determine user home directory")?;
    path.push(".recall");
    Ok(path)
}

/// Initialize connection to SQLite database and run migration schema setup
pub fn init_db() -> Result<Connection> {
    let db_dir = get_db_path()?;
    fs::create_dir_all(&db_dir)
        .with_context(|| format!("Failed to create database directory: {}", db_dir.display()))?;

    let db_path = db_dir.join("recall.db");
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open SQLite database at: {}", db_path.display()))?;

    // Automatically create the notes table if it does not exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .context("Failed to initialize database schema")?;

    Ok(conn)
}

/// Save a note text to the database
pub fn save_note(conn: &Connection, content: &str) -> Result<i64> {
    conn.execute("INSERT INTO notes (content) VALUES (?1)", [content])
        .context("Failed to save note to database")?;

    Ok(conn.last_insert_rowid())
}

/// List all notes in the database sorted by id DESC (most recent first)
pub fn list_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn
        .prepare("SELECT id, content, created_at FROM notes ORDER BY id DESC")
        .context("Failed to prepare select query")?;

    let note_iter = stmt
        .query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .context("Failed to execute select query")?;

    let mut notes = Vec::new();
    for note in note_iter {
        notes.push(note?);
    }
    Ok(notes)
}

/// Retrieve a note by its 0-based offset from the DESC sorted list of notes
pub fn get_note_by_offset(conn: &Connection, offset: usize) -> Result<Option<Note>> {
    let mut stmt = conn
        .prepare("SELECT id, content, created_at FROM notes ORDER BY id DESC LIMIT 1 OFFSET ?1")
        .context("Failed to prepare offset select query")?;

    let mut note_iter = stmt
        .query_map([offset], |row| {
            Ok(Note {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .context("Failed to execute offset select query")?;

    if let Some(note) = note_iter.next() {
        Ok(Some(note?))
    } else {
        Ok(None)
    }
}

/// Count the total number of notes currently in the database
pub fn count_notes(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .context("Failed to count notes in database")
}

/// Delete a note by its unique auto-increment database ID
pub fn delete_note_by_id(conn: &Connection, id: i64) -> Result<()> {
    let rows_affected = conn
        .execute("DELETE FROM notes WHERE id = ?1", [id])
        .context("Failed to execute delete query")?;

    if rows_affected == 0 {
        anyhow::bail!("Note with ID {} does not exist", id);
    }

    // If no notes remain, reset sqlite_sequence so autoincrement restarts from 1
    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
    if remaining == 0 {
        let _ = conn.execute("DELETE FROM sqlite_sequence WHERE name = 'notes'", []);
    }

    Ok(())
}

/// Update the content of an existing note by its unique database ID
pub fn update_note_content(conn: &Connection, id: i64, new_content: &str) -> Result<()> {
    let rows_affected = conn
        .execute(
            "UPDATE notes SET content = ?1 WHERE id = ?2",
            rusqlite::params![new_content, id],
        )
        .context("Failed to update note in database")?;

    if rows_affected == 0 {
        anyhow::bail!("Note with ID {} does not exist", id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_in_memory_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(conn)
    }

    #[test]
    fn test_save_and_list() -> Result<()> {
        let conn = init_in_memory_db()?;
        let id1 = save_note(&conn, "First note")?;
        let id2 = save_note(&conn, "Second note")?;
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let notes = list_notes(&conn)?;
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].content, "Second note"); // DESC order
        assert_eq!(notes[1].content, "First note");

        Ok(())
    }

    #[test]
    fn test_get_by_offset() -> Result<()> {
        let conn = init_in_memory_db()?;
        save_note(&conn, "Oldest")?;
        save_note(&conn, "Middle")?;
        save_note(&conn, "Newest")?;

        let note0 = get_note_by_offset(&conn, 0)?;
        assert!(note0.is_some());
        assert_eq!(note0.unwrap().content, "Newest");

        let note1 = get_note_by_offset(&conn, 1)?;
        assert!(note1.is_some());
        assert_eq!(note1.unwrap().content, "Middle");

        let note2 = get_note_by_offset(&conn, 2)?;
        assert!(note2.is_some());
        assert_eq!(note2.unwrap().content, "Oldest");

        let note3 = get_note_by_offset(&conn, 3)?;
        assert!(note3.is_none());

        Ok(())
    }

    #[test]
    fn test_count() -> Result<()> {
        let conn = init_in_memory_db()?;
        assert_eq!(count_notes(&conn)?, 0);

        save_note(&conn, "First")?;
        assert_eq!(count_notes(&conn)?, 1);

        save_note(&conn, "Second")?;
        assert_eq!(count_notes(&conn)?, 2);

        Ok(())
    }

    #[test]
    fn test_count_after_delete() -> Result<()> {
        // Regression test: deleting notes must not make the count keep increasing
        let conn = init_in_memory_db()?;
        save_note(&conn, "One")?;
        save_note(&conn, "Two")?;
        save_note(&conn, "Three")?;
        assert_eq!(count_notes(&conn)?, 3);

        // Delete one note (by its database id) and count again
        let notes = list_notes(&conn)?;
        delete_note_by_id(&conn, notes[0].id)?;
        assert_eq!(count_notes(&conn)?, 2);

        // Save again: count reflects live rows, not the ever-increasing rowid
        save_note(&conn, "Four")?;
        assert_eq!(count_notes(&conn)?, 3);

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<()> {
        let conn = init_in_memory_db()?;
        let id = save_note(&conn, "Delete me")?;
        assert!(get_note_by_offset(&conn, 0)?.is_some());
        delete_note_by_id(&conn, id)?;
        assert!(get_note_by_offset(&conn, 0)?.is_none());
        Ok(())
    }

    #[test]
    fn test_update() -> Result<()> {
        let conn = init_in_memory_db()?;
        let id = save_note(&conn, "Original content")?;
        update_note_content(&conn, id, "Updated content")?;

        let note = get_note_by_offset(&conn, 0)?.expect("Note should exist");
        assert_eq!(note.content, "Updated content");
        assert_eq!(note.id, id);
        Ok(())
    }

    #[test]
    fn test_delete_all_resets_sequence() -> Result<()> {
        let conn = init_in_memory_db()?;
        let id1 = save_note(&conn, "First")?;
        delete_note_by_id(&conn, id1)?;
        assert_eq!(count_notes(&conn)?, 0);

        // Next inserted note after empty table should get id 1 because sequence was cleared
        let id2 = save_note(&conn, "Fresh start")?;
        assert_eq!(id2, 1);
        Ok(())
    }
}
