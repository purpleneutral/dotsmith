use anyhow::Result;
use clap::Parser;

mod cli;
mod core;
mod util;

use cli::{Commands, DotsmithCli};

fn main() -> Result<()> {
    let cli = DotsmithCli::parse();

    let result = match cli.command {
        Commands::Init => cli::init::run(cli.verbose),
        Commands::Add { ref tool } => cli::add::run(cli.verbose, tool),
        Commands::Remove { ref tool } => cli::remove::run(cli.verbose, tool),
        Commands::List => cli::list::run(cli.verbose),
        Commands::Status => cli::status::run(cli.verbose),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", colored::Colorize::red("error"), e);
        std::process::exit(1);
    }

    Ok(())
}
