use colored::Colorize;

/// Print a success message prefixed with a green checkmark
pub fn print_success(msg: &str) {
    println!("  {} {}", "✔".bright_green().bold(), msg);
}

/// Print an error message prefixed with a red cross to stderr
pub fn print_error(msg: &str) {
    eprintln!("  {} {}", "✖".bright_red().bold(), msg);
}

/// Print a warning message prefixed with a yellow warning triangle
pub fn print_warning(msg: &str) {
    println!("  {} {}", "▲".bright_yellow().bold(), msg);
}

/// Print an info message prefixed with an info icon
pub fn print_info(msg: &str) {
    println!("  {} {}", "ℹ".bright_cyan().bold(), msg);
}

/// Format headings in cyan and bold
#[allow(dead_code)]
pub fn format_heading(msg: &str) -> String {
    msg.bright_cyan().bold().to_string()
}

/// Format the note list numbers in bright cyan
#[allow(dead_code)]
pub fn format_number(num: usize) -> String {
    format!("#{:02}", num).bright_cyan().bold().to_string()
}

/// Format a divider line
#[allow(dead_code)]
pub fn print_divider() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Print a styled card box for displaying a command (e.g. on copy)
pub fn print_copy_box(title: &str, content: &str) {
    let width: usize = 63;
    let title_display = format!(" {} ", title);
    let border_len = width.saturating_sub(title_display.chars().count() + 4);

    println!(
        "  {}{}{}{}",
        "╭──".dimmed(),
        title_display.bright_cyan().bold(),
        "─".repeat(border_len).dimmed(),
        "╮".dimmed()
    );
    for line in content.lines() {
        let line_len = line.chars().count();
        let pad = width.saturating_sub(line_len + 5);
        println!(
            "  {}   {}{}{}",
            "│".dimmed(),
            line.bright_white(),
            " ".repeat(pad),
            "│".dimmed()
        );
    }
    println!("  {}", format!("╰{}╯", "─".repeat(width - 2)).dimmed());
}

/// Parse SQLite CURRENT_TIMESTAMP ("YYYY-MM-DD HH:MM:SS" UTC) and return a friendly relative time
pub fn format_relative_time(created_at: &str) -> String {
    if let Some(sec) = parse_utc_timestamp(created_at) {
        if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            let now_sec = now.as_secs();
            if now_sec >= sec {
                let diff = now_sec - sec;
                if diff < 60 {
                    return "just now".to_string();
                } else if diff < 3600 {
                    return format!("{}m ago", diff / 60);
                } else if diff < 86400 {
                    return format!("{}h ago", diff / 3600);
                } else if diff < 86400 * 30 {
                    return format!("{}d ago", diff / 86400);
                } else if diff < 86400 * 365 {
                    return format!("{}mo ago", diff / (86400 * 30));
                } else {
                    return format!("{}y ago", diff / (86400 * 365));
                }
            }
        }
    }
    // Fallback to simple date (YYYY-MM-DD) if available
    created_at
        .split_whitespace()
        .next()
        .unwrap_or(created_at)
        .to_string()
}

fn parse_utc_timestamp(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();
    if date_parts.len() != 3 || time_parts.len() < 2 {
        return None;
    }
    let (y, m, d) = (date_parts[0], date_parts[1], date_parts[2]);
    let h = time_parts[0];
    let min = time_parts[1];
    let sec = if time_parts.len() > 2 {
        time_parts[2]
    } else {
        0
    };

    let days = days_from_civil(y as i64, m as u32, d as u32)?;
    if days < 0 {
        return None;
    }
    Some((days as u64) * 86400 + h * 3600 + min * 60 + sec)
}

fn days_from_civil(mut y: i64, m: u32, d: u32) -> Option<i64> {
    if m < 1 || m > 12 || d < 1 || d > 31 {
        return None;
    }
    y -= (m <= 2) as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe as i64 - 719468)
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

    #[test]
    fn test_format_relative_time_smoke() {
        // A timestamp from far past should render in years or months
        let rel = format_relative_time("2020-01-01 00:00:00");
        assert!(rel.ends_with("y ago") || rel.ends_with("mo ago"));

        // Invalid timestamp should return fallback
        let bad = format_relative_time("invalid-date");
        assert_eq!(bad, "invalid-date");
    }
}
