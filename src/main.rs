mod cli;
mod db;
mod models;
mod clipboard;
mod ui;
mod commands;

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
    
    // Establish DB connection and run migrations
    let conn = db::init_db()?;
    
    // Route command execution
    if let Some(ref text) = args.save {
        commands::save::handle(&conn, text)?;
    } else if let Some(idx) = args.delete {
        commands::delete::handle(&conn, idx, args.force)?;
    } else if let Some(idx) = args.copy {
        commands::copy::handle(&conn, idx)?;
    } else {
        commands::list::handle(&conn)?;
    }
    
    Ok(())
}
