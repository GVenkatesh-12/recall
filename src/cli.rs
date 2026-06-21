use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "recall",
    version,
    about = "A lightweight CLI note capture and clipboard utility",
    long_about = "Recall is a minimal CLI notes manager. It stores notes in a local SQLite database, lists them ordered by recency, and allows copying or deleting notes by their displayed index number.",
    group = clap::ArgGroup::new("action").multiple(false)
)]
pub struct Cli {
    /// Save a new note
    #[arg(
        short,
        long,
        value_name = "TEXT",
        help = "Save a new note to the database",
        group = "action"
    )]
    pub save: Option<String>,

    /// Delete a note by its displayed index
    #[arg(
        short,
        long,
        value_name = "INDEX",
        help = "Delete a note from the database by its displayed list number",
        group = "action"
    )]
    pub delete: Option<usize>,

    /// Skip confirmation prompt when deleting a note (use with --delete)
    #[arg(
        short,
        long,
        help = "Skip confirmation prompt when deleting a note",
        requires = "delete"
    )]
    pub force: bool,

    /// Copy a note to clipboard by its displayed index (1-based)
    #[arg(
        value_name = "INDEX",
        help = "Copy the contents of a note to the clipboard by its displayed list number",
        group = "action"
    )]
    pub copy: Option<usize>,
}

impl Cli {
    /// Parse arguments from the command line
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
