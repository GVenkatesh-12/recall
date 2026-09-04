use crate::db;
use crate::ui;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Handle saving the last executed command from shell history
pub fn handle(conn: &Connection) -> Result<()> {
    match find_last_command() {
        Some(cmd) => {
            let trimmed = cmd.trim();
            db::save_note(conn, trimmed)?;
            let total = db::count_notes(conn)?;

            ui::print_success(&format!("Saved last command as #{}", total));
            println!();
            ui::print_copy_box(&format!("Command #{}", total), trimmed);
            println!();

            let is_bash = std::env::var("SHELL")
                .map(|s| s.ends_with("bash"))
                .unwrap_or(false);
            if is_bash && std::env::var("RECALL_WRAPPER").is_err() {
                ui::print_info(
                    "Tip: In Bash, add 'eval \"$(recall init bash)\"' to ~/.bashrc to capture commands instantly.",
                );
                println!();
            }
            Ok(())
        }
        None => {
            ui::print_error("Could not find any recent command in your shell history.");
            ui::print_info(
                "Ensure your shell history is written to disk, or save manually using: recall -s \"<command>\"",
            );
            Ok(())
        }
    }
}

/// Find the last executed command across active shell histories
pub fn find_last_command() -> Option<String> {
    let candidate_files = detect_history_files();

    for path in candidate_files {
        if let Ok(Some(cmd)) = parse_history_file(&path) {
            return Some(cmd);
        }
    }

    None
}

/// Detect possible shell history files, sorted by newest modification time first
fn detect_history_files() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Check $HISTFILE environment variable
    if let Ok(histfile) = std::env::var("HISTFILE") {
        let p = PathBuf::from(histfile);
        if p.is_file() {
            candidates.push(p);
        }
    }

    // 2. Check standard shell history files in home directory
    if let Some(home) = dirs::home_dir() {
        let defaults = [
            home.join(".bash_history"),
            home.join(".zsh_history"),
            home.join(".local/share/fish/fish_history"),
            home.join(".history"),
            home.join(".ash_history"),
            home.join(".sh_history"),
        ];

        for path in defaults {
            if path.is_file() && !candidates.contains(&path) {
                candidates.push(path);
            }
        }
    }

    // Sort by modification time (most recent first)
    candidates.sort_by(|a, b| {
        let mtime_a = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mtime_b = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        mtime_b.cmp(&mtime_a)
    });

    candidates
}

/// Parse a given history file and return the last non-recall command
pub fn parse_history_file(path: &Path) -> Result<Option<String>> {
    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        // Read lines, tolerating non-UTF8 characters in zsh/bash history
        match line {
            Ok(l) => lines.push(l),
            Err(_) => continue,
        }
    }

    // Parse commands from lines based on file type / content
    let commands = extract_commands_from_lines(&lines);

    // Search backwards for the first non-recall command
    for cmd in commands.into_iter().rev() {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() && !is_recall_command(trimmed) {
            return Ok(Some(trimmed.to_string()));
        }
    }

    Ok(None)
}

/// Extract clean command strings from raw history lines (supports Bash, Zsh, and Fish)
pub fn extract_commands_from_lines(lines: &[String]) -> Vec<String> {
    let mut commands = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 1. Check Fish format: "- cmd: ..."
        if let Some(rest) = trimmed.strip_prefix("- cmd:") {
            let cmd = rest.trim();
            if !cmd.is_empty() {
                commands.push(cmd.to_string());
            }
            continue;
        }

        // Fish metadata lines
        if trimmed.starts_with("when:")
            || trimmed.starts_with("paths:")
            || trimmed.starts_with("- /")
        {
            continue;
        }

        // 2. Check Zsh extended format: ": 1693849200:0;git status"
        if trimmed.starts_with(':')
            && let Some(idx) = trimmed.find(';')
        {
            let cmd = trimmed[idx + 1..].trim();
            if !cmd.is_empty() {
                commands.push(cmd.to_string());
            }
            continue;
        }

        // 3. Check Bash timestamp comment: "#1693849200"
        if trimmed.starts_with('#') && trimmed[1..].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // 4. Standard plain command line
        commands.push(trimmed.to_string());
    }

    commands
}

/// Determine if a command string is an invocation of recall itself
pub fn is_recall_command(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return true;
    }

    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    first_word == "recall"
        || first_word.ends_with("/recall")
        || trimmed.starts_with("cargo run")
        || trimmed.starts_with("history")
        || trimmed.starts_with("builtin history")
        || trimmed == "clear"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_recall_command() {
        assert!(is_recall_command("recall"));
        assert!(is_recall_command("recall -l"));
        assert!(is_recall_command("recall -s 'hello'"));
        assert!(is_recall_command("./recall 1"));
        assert!(is_recall_command("/home/user/.local/bin/recall run 2"));
        assert!(is_recall_command("cargo run -- -l"));
        assert!(is_recall_command("clear"));
        assert!(is_recall_command("history"));
        assert!(is_recall_command("history -a"));
        assert!(is_recall_command("builtin history -a"));

        assert!(!is_recall_command("git status"));
        assert!(!is_recall_command("docker compose up -d"));
        assert!(!is_recall_command("ffmpeg -i input.mov output.mp4"));
    }

    #[test]
    fn test_extract_bash_history() {
        let lines = vec![
            "#1693849100".to_string(),
            "ls -la".to_string(),
            "#1693849200".to_string(),
            "git commit -m \"fix: issue\"".to_string(),
            "recall -l".to_string(),
        ];

        let cmds = extract_commands_from_lines(&lines);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "ls -la");
        assert_eq!(cmds[1], "git commit -m \"fix: issue\"");
        assert_eq!(cmds[2], "recall -l");
    }

    #[test]
    fn test_extract_zsh_extended_history() {
        let lines = vec![
            ": 1693849100:0;echo 'hello world'".to_string(),
            ": 1693849200:1;cargo test --all".to_string(),
            ": 1693849300:0;recall".to_string(),
        ];

        let cmds = extract_commands_from_lines(&lines);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "echo 'hello world'");
        assert_eq!(cmds[1], "cargo test --all");
        assert_eq!(cmds[2], "recall");
    }

    #[test]
    fn test_extract_fish_history() {
        let lines = vec![
            "- cmd: curl https://example.com".to_string(),
            "  when: 1693849100".to_string(),
            "- cmd: recall -l".to_string(),
            "  when: 1693849200".to_string(),
        ];

        let cmds = extract_commands_from_lines(&lines);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "curl https://example.com");
        assert_eq!(cmds[1], "recall -l");
    }
}
