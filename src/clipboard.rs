use arboard::Clipboard;
use anyhow::{Context, Result};

/// Copy the given text to the system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .context("Failed to initialize system clipboard. Ensure your display server (X11/Wayland) is running if on Linux.")?;
    clipboard.set_text(text.to_string())
        .context("Failed to write text to system clipboard.")?;
    Ok(())
}
