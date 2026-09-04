use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "recall",
    version,
    about = "A modern terminal command memory and clipboard utility",
    long_about = "Recall is a minimal CLI command memory tool. It stores the terminal commands you want to remember in a local SQLite database, lists them ordered by recency, and allows copying, editing, or deleting commands by their displayed index number.",
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

    /// Edit a command by its displayed index
    #[arg(
        short,
        long,
        value_name = "INDEX",
        help = "Edit an existing command by its displayed list number",
        group = "action"
    )]
    pub edit: Option<usize>,

    /// Update recall to the latest version from GitHub
    #[arg(
        short,
        long,
        help = "Update recall to the latest version from GitHub",
        group = "action"
    )]
    pub update: bool,

    /// Copy a command to clipboard by its displayed index (1-based), or 'update'
    #[arg(
        value_name = "INDEX",
        help = "Copy command to clipboard by list number (e.g. 1), or 'update'",
        group = "action"
    )]
    pub target: Option<String>,
}

impl Cli {
    /// Parse arguments from the command line
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
