use anyhow::{Context, Result};
use rusqlite::Connection;
use crate::db;
use crate::clipboard;
use crate::ui;

/// Handle copying a note to the clipboard by its displayed 1-based index
pub fn handle(conn: &Connection, index: usize) -> Result<()> {
    if index == 0 {
        ui::print_error("Invalid note number. Numbers start from 1.");
        return Ok(());
    }
    
    let offset = index - 1;
    let note_opt = db::get_note_by_offset(conn, offset)?;
    
    match note_opt {
        Some(note) => {
            clipboard::copy_to_clipboard(&note.content)
                .context("Clipboard operation failed")?;
            
            ui::print_success("Copied to clipboard\n");
            println!("{}", note.content);
        }
        None => {
            ui::print_error(&format!("Note #{} does not exist", index));
        }
    }
    
    Ok(())
}
