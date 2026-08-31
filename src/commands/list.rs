use crate::db;
use crate::ui;
use anyhow::Result;
use rusqlite::Connection;

/// Handle listing all notes
pub fn handle(conn: &Connection) -> Result<()> {
    let notes = db::list_notes(conn)?;

    if notes.is_empty() {
        ui::print_warning("No notes found.");
        println!("Use: recall -s \"your note\"");
        return Ok(());
    }

    println!("\n {}", ui::format_heading("Recall Notes"));
    ui::print_divider();
    println!();

    let pad_width = notes.len().to_string().len();
    for (idx, note) in notes.iter().enumerate() {
        let display_num = idx + 1;
        // Truncate at 80 characters or first newline
        let truncated = ui::truncate_note(&note.content, 80);
        println!(
            " {:>pad_width$}  {}",
            ui::format_number(display_num),
            truncated
        );
    }

    println!();
    ui::print_divider();
    println!(
        "Total: {} note{}",
        notes.len(),
        if notes.len() == 1 { "" } else { "s" }
    );

    Ok(())
}
