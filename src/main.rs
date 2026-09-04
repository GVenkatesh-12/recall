mod cli;
mod clipboard;
mod commands;
mod db;
mod models;
mod ui;

use anyhow::Result;
use cli::Cli;

fn main() {
    if let Err(err) = run() {
        // Print the error in the specified user-friendly Red X format
        ui::print_error(&format!("{:#}", err));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse CLI arguments
    let args = Cli::parse_args();

    // Check if user requested self-update via --update or 'recall update'
    let is_update =
        args.update || matches!(args.target.as_deref(), Some("update") | Some("self-update"));

    if is_update {
        return commands::update::handle();
    }

    // Establish DB connection and run migrations
    let conn = db::init_db()?;

    // Route command execution
    if let Some(ref text) = args.save {
        commands::save::handle(&conn, text)?;
    } else if args.last {
        commands::last::handle(&conn)?;
    } else if let Some(idx) = args.run {
        commands::run::handle(&conn, idx, args.yes || args.force)?;
    } else if let Some(idx) = args.delete {
        commands::delete::handle(&conn, idx, args.force)?;
    } else if let Some(idx) = args.edit {
        commands::edit::handle(&conn, idx, None)?;
    } else if let Some(ref t) = args.target {
        if t == "run" {
            match args.arg {
                Some(ref arg) => match arg.parse::<usize>() {
                    Ok(idx) => commands::run::handle(&conn, idx, args.yes || args.force)?,
                    Err(_) => {
                        ui::print_error(&format!(
                            "Invalid command number '{}' for run. Expected a number (e.g. 'recall run 1').",
                            arg
                        ));
                    }
                },
                None => {
                    ui::print_error("Missing command number for run. Usage: recall run <id>");
                }
            }
        } else {
            match t.parse::<usize>() {
                Ok(idx) => commands::copy::handle(&conn, idx)?,
                Err(_) => {
                    ui::print_error(&format!(
                        "Invalid argument '{}'. Expected a command number (e.g. 'recall 1'), 'recall run <id>', or 'recall update'.",
                        t
                    ));
                }
            }
        }
    } else {
        commands::list::handle(&conn)?;
    }

    Ok(())
}
