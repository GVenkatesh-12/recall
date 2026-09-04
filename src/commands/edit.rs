use crate::db;
use crate::ui;
use anyhow::{Context, Result};
use inquire::Text;
use rusqlite::Connection;

/// Handle editing a command by its displayed index (1-based)
pub fn handle(conn: &Connection, index: usize, new_content: Option<String>) -> Result<()> {
    if index == 0 {
        ui::print_error("Invalid note number. Numbers start from 1.");
        return Ok(());
    }

    let offset = index - 1;
    let note_opt = db::get_note_by_offset(conn, offset)?;

    match note_opt {
        Some(note) => {
            let updated = match new_content {
                Some(text) => text,
                None => Text::new("Edit command:")
                    .with_default(&note.content)
                    .prompt()
                    .context("Prompt cancelled")?,
            };

            let trimmed = updated.trim();
            if trimmed.is_empty() {
                ui::print_error("Command content cannot be empty.");
                return Ok(());
            }

            db::update_note_content(conn, note.id, trimmed)?;
            ui::print_success(&format!("Updated note #{}", index));
        }
        None => {
            ui::print_error(&format!("Note #{} does not exist", index));
        }
    }

    Ok(())
}
