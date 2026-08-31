use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "recall",
    version,
    about = "A lightweight CLI terminal command memory and clipboard utility",
    long_about = "Recall is a minimal CLI command memory tool. It stores the terminal commands you want to remember in a local SQLite database, lists them ordered by recency, and allows copying or deleting commands by their displayed index number.",
    group = clap::ArgGroup::new("action").multiple(false)
)]
pub struct Cli {
    /// Save a new command
    #[arg(
        short,
        long,
        value_name = "TEXT",
        help = "Save a new command to the database",
        group = "action"
    )]
    pub save: Option<String>,

    /// Delete a command by its displayed index
    #[arg(
        short,
        long,
        value_name = "INDEX",
        help = "Delete a command from the database by its displayed list number",
        group = "action"
    )]
    pub delete: Option<usize>,

    /// Skip confirmation prompt when deleting a command (use with --delete)
    #[arg(
        short,
        long,
        help = "Skip confirmation prompt when deleting a command",
        requires = "delete"
    )]
    pub force: bool,

    /// Copy a command to clipboard by its displayed index (1-based)
    #[arg(
        value_name = "INDEX",
        help = "Copy the contents of a command to the clipboard by its displayed list number",
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
