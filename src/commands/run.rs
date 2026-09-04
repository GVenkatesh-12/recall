use crate::db;
use crate::ui;
use anyhow::{Context, Result};
use inquire::Confirm;
use rusqlite::Connection;

/// Handle executing a saved command by its displayed index
pub fn handle(conn: &Connection, index: usize, yes: bool) -> Result<()> {
    if index == 0 {
        ui::print_error("Invalid command number. Numbers start from 1.");
        return Ok(());
    }

    let offset = index - 1;
    let note_opt = db::get_note_by_offset(conn, offset)?;

    match note_opt {
        Some(note) => {
            ui::print_copy_box(&format!("Run Command #{}", index), &note.content);
            println!();

            let should_run = if yes {
                true
            } else {
                Confirm::new(&format!("Execute command #{}?", index))
                    .with_default(true)
                    .prompt()
                    .unwrap_or(false)
            };

            if !should_run {
                ui::print_warning("Execution cancelled.");
                return Ok(());
            }

            ui::print_info(&format!("Running: {}", note.content));
            println!();

            execute_command(&note.content)?;
        }
        None => {
            ui::print_error(&format!(
                "Command #{} does not exist. Run 'recall' to see your commands.",
                index
            ));
        }
    }

    Ok(())
}

/// Execute a shell command string in a child process with inherited standard streams
pub fn execute_command(cmd_str: &str) -> Result<()> {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut child = std::process::Command::new(shell)
            .arg("-c")
            .arg(cmd_str)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn shell to run '{}'", cmd_str))?;

        let status = child
            .wait()
            .with_context(|| format!("Failed to wait for process '{}'", cmd_str))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    #[cfg(windows)]
    {
        let mut child = std::process::Command::new("cmd")
            .arg("/C")
            .arg(cmd_str)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn cmd.exe to run '{}'", cmd_str))?;

        let status = child
            .wait()
            .with_context(|| format!("Failed to wait for process '{}'", cmd_str))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}
