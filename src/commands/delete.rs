use anyhow::Result;
use rusqlite::Connection;
use crate::db;
use crate::ui;
use inquire::Confirm;

/// Handle deleting a note by its displayed index
pub fn handle(conn: &Connection, index: usize, force: bool) -> Result<()> {
    if index == 0 {
        ui::print_error("Invalid note number. Numbers start from 1.");
        return Ok(());
    }
    
    let offset = index - 1;
    let note_opt = db::get_note_by_offset(conn, offset)?;
    
    match note_opt {
        Some(note) => {
            let should_delete = if force {
                true
            } else {
                Confirm::new("Delete this note?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false)
            };
            
            if should_delete {
                db::delete_note_by_id(conn, note.id)?;
                ui::print_success(&format!("Deleted note #{}", index));
            } else {
                println!("Deletion cancelled.");
            }
        }
        None => {
            ui::print_error(&format!("Note #{} does not exist", index));
        }
    }
    
    Ok(())
}
