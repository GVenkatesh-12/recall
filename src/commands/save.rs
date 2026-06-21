use anyhow::Result;
use rusqlite::Connection;
use crate::db;
use crate::ui;

/// Handle saving a note to the SQLite database
pub fn handle(conn: &Connection, content: &str) -> Result<()> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Note content cannot be empty.");
    }
    
    let id = db::save_note(conn, trimmed)?;
    ui::print_success(&format!("Saved note #{}", id));
    Ok(())
}
