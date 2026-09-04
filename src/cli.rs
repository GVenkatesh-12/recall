use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "recall",
    version,
    about = "A modern terminal command memory and clipboard utility",
    long_about = "Recall is a minimal CLI command memory tool. It stores the terminal commands you want to remember in a local SQLite database, lists them in chronological order, and allows copying, editing, running, or deleting commands by their displayed index number.",
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

    /// Save the last executed command from shell history
    #[arg(
        short,
        long,
        help = "Capture and save the last executed command from your shell history",
        group = "action"
    )]
    pub last: bool,

    /// Execute a command directly by its displayed index
    #[arg(
        short = 'x',
        long = "run",
        value_name = "INDEX",
        help = "Execute a command directly by its displayed list number",
        group = "action"
    )]
    pub run: Option<usize>,

    /// Delete a command by its displayed index
    #[arg(
        short,
        long,
        value_name = "INDEX",
        help = "Delete a command from the database by its displayed list number",
        group = "action"
    )]
    pub delete: Option<usize>,

    /// Skip confirmation prompt when deleting or running a command
    #[arg(
        short,
        long,
        help = "Skip confirmation prompt when deleting or running a command"
    )]
    pub force: bool,

    /// Skip confirmation prompt when running a command
    #[arg(short, long, help = "Skip confirmation prompt when running a command")]
    pub yes: bool,

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

    /// Copy a command by list number, run a command ('run <id>'), or 'update'
    #[arg(
        value_name = "TARGET",
        help = "Command number to copy (e.g. 1), 'run <id>' to execute, or 'update'",
        group = "action"
    )]
    pub target: Option<String>,

    /// Secondary argument (e.g. command number for 'run <id>')
    #[arg(
        value_name = "ARG",
        help = "Target command number for 'run <id>'",
        requires = "target"
    )]
    pub arg: Option<String>,
}

impl Cli {
    /// Parse arguments from the command line
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let cli = Cli::try_parse_from(["recall"]).unwrap();
        assert!(cli.save.is_none());
        assert!(!cli.last);
        assert!(cli.run.is_none());
        assert!(cli.delete.is_none());
        assert!(cli.target.is_none());
    }

    #[test]
    fn test_parse_last_flag() {
        let cli = Cli::try_parse_from(["recall", "-l"]).unwrap();
        assert!(cli.last);

        let cli2 = Cli::try_parse_from(["recall", "--last"]).unwrap();
        assert!(cli2.last);
    }

    #[test]
    fn test_parse_run_flag() {
        let cli = Cli::try_parse_from(["recall", "-x", "3"]).unwrap();
        assert_eq!(cli.run, Some(3));

        let cli2 = Cli::try_parse_from(["recall", "--run", "5", "-y"]).unwrap();
        assert_eq!(cli2.run, Some(5));
        assert!(cli2.yes);
    }

    #[test]
    fn test_parse_run_positional() {
        let cli = Cli::try_parse_from(["recall", "run", "2"]).unwrap();
        assert_eq!(cli.target.as_deref(), Some("run"));
        assert_eq!(cli.arg.as_deref(), Some("2"));

        let cli_yes = Cli::try_parse_from(["recall", "run", "2", "-y"]).unwrap();
        assert_eq!(cli_yes.target.as_deref(), Some("run"));
        assert_eq!(cli_yes.arg.as_deref(), Some("2"));
        assert!(cli_yes.yes);
    }

    #[test]
    fn test_parse_copy_positional() {
        let cli = Cli::try_parse_from(["recall", "1"]).unwrap();
        assert_eq!(cli.target.as_deref(), Some("1"));
        assert!(cli.arg.is_none());
    }

    #[test]
    fn test_parse_update_positional() {
        let cli = Cli::try_parse_from(["recall", "update"]).unwrap();
        assert_eq!(cli.target.as_deref(), Some("update"));
    }
}
