use crate::db;
use crate::ui;
use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;

/// Handle listing all notes
pub fn handle(conn: &Connection) -> Result<()> {
    let notes = db::list_notes(conn)?;
    let pkg_version = env!("CARGO_PKG_VERSION");

    if notes.is_empty() {
        println!();
        println!(
            "  {}  {}",
            "✦ RECALL".bright_cyan().bold(),
            format!("v{}", pkg_version).dimmed()
        );
        println!(
            "  {}",
            "╭─────────────────────────────────────────────────────────────╮".dimmed()
        );
        println!(
            "  {}                                                             {}",
            "│".dimmed(),
            "│".dimmed()
        );
        println!(
            "  {}   {}                             {}",
            "│".dimmed(),
            "No commands stored yet.".bright_white().bold(),
            "│".dimmed()
        );
        println!(
            "  {}                                                             {}",
            "│".dimmed(),
            "│".dimmed()
        );
        println!(
            "  {}   {}                                               {}",
            "│".dimmed(),
            "Quick Start:".bright_yellow().bold(),
            "│".dimmed()
        );
        println!(
            "  {}     {}    Save a command        {}",
            "│".dimmed(),
            "recall -s \"docker compose up -d\"".bright_white(),
            "│".dimmed()
        );
        println!(
            "  {}     {}                     Copy command #1       {}",
            "│".dimmed(),
            "recall 1".bright_white(),
            "│".dimmed()
        );
        println!(
            "  {}     {}                   Edit command #1       {}",
            "│".dimmed(),
            "recall -e 1".bright_white(),
            "│".dimmed()
        );
        println!(
            "  {}     {}                     Update recall CLI     {}",
            "│".dimmed(),
            "recall update".bright_white(),
            "│".dimmed()
        );
        println!(
            "  {}                                                             {}",
            "│".dimmed(),
            "│".dimmed()
        );
        println!(
            "  {}",
            "╰─────────────────────────────────────────────────────────────╯".dimmed()
        );
        println!();
        return Ok(());
    }

    let count_text = format!(
        "{} {}",
        notes.len(),
        if notes.len() == 1 {
            "command"
        } else {
            "commands"
        }
    );

    let header_prefix_len = 2 + 8 + 2 + format!("v{}", pkg_version).len();
    let header_pad = 65usize.saturating_sub(header_prefix_len);

    println!();
    println!(
        "  {}  {} {:>width$}",
        "✦ RECALL".bright_cyan().bold(),
        format!("v{}", pkg_version).dimmed(),
        count_text.bright_magenta(),
        width = header_pad
    );
    println!(
        "  {}",
        "╭─────────────────────────────────────────────────────────────╮".dimmed()
    );

    let pad_width = if notes.len() >= 100 { 3 } else { 2 };
    for (idx, note) in notes.iter().enumerate() {
        let display_num = idx + 1;
        let time_str = ui::format_relative_time(&note.created_at);
        // Truncate command to fit cleanly within box
        let truncated = ui::truncate_note(&note.content, 40);

        let num_str = format!("#{:0width$}", display_num, width = pad_width);
        let cmd_len = truncated.chars().count();
        let time_len = time_str.chars().count();
        // Exact inner width is 51 characters for (cmd + spaces + time)
        let space_available = 51usize.saturating_sub(cmd_len + time_len);
        let spaces = " ".repeat(space_available.max(2));

        println!(
            "  {}   {}  {}{}{}  {}",
            "│".dimmed(),
            num_str.bright_cyan().bold(),
            truncated.bright_white(),
            spaces,
            time_str.dimmed(),
            "│".dimmed()
        );
    }

    println!(
        "  {}",
        "╰─────────────────────────────────────────────────────────────╯".dimmed()
    );
    println!(
        "  {}",
        "• recall <id> copy  •  recall -s add  •  recall -e edit  •  recall -d delete".dimmed()
    );
    println!();

    Ok(())
}
