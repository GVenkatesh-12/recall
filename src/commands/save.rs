use crate::db;
use crate::ui;
use anyhow::Result;
use rusqlite::Connection;

/// Handle saving a command to the SQLite database
pub fn handle(conn: &Connection, content: &str) -> Result<()> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Command content cannot be empty.");
    }

    db::save_note(conn, trimmed)?;
    let total = db::count_notes(conn)?;
    ui::print_success(&format!("Saved note #{}", total));
    Ok(())
}
