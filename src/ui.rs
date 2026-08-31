use colored::Colorize;

/// Print a success message prefixed with a green checkmark
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message prefixed with a red cross to stderr
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// Print a warning message prefixed with a yellow warning triangle
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Format headings in cyan and bold
pub fn format_heading(msg: &str) -> String {
    msg.cyan().bold().to_string()
}

/// Format the note list numbers in gray/dim
pub fn format_number(num: usize) -> String {
    num.to_string().cyan().to_string()
}

/// Format the divider line
pub fn print_divider() {
    println!("{}", "─".repeat(50).dimmed());
}

/// Truncate very long notes intelligently.
/// Takes the first line of the note. If the first line is longer than `limit`,
/// truncates it and appends "...". If the note has multiple lines, appends "..."
/// to indicate there is more content.
pub fn truncate_note(content: &str, limit: usize) -> String {
    let mut lines = content.lines();
    if let Some(first_line) = lines.next() {
        let has_more_lines = lines.next().is_some();
        if first_line.len() > limit {
            format!("{}...", &first_line[..limit])
        } else if has_more_lines {
            format!("{}...", first_line)
        } else {
            first_line.to_string()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_note() {
        assert_eq!(truncate_note("Short note", 20), "Short note");
        assert_eq!(
            truncate_note("A very long note that exceeds limit", 10),
            "A very lon..."
        );
        assert_eq!(truncate_note("Line one\nLine two", 20), "Line one...");
        assert_eq!(
            truncate_note("Line one that is long\nLine two", 5),
            "Line ..."
        );
        assert_eq!(truncate_note("", 20), "");
    }
}
