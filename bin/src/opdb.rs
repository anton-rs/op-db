use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser, Subcommand};
use tracing::Level;

/// Command arguments for `opdb`
#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count, default_value = "2")]
    v: u8,

    /// `migrate subcommand`
    #[command(subcommand)]
    commands: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    Migrate { datadir: PathBuf, output: PathBuf },
}

fn main() -> Result<()> {
    // Parse the command arguments
    let Args { v, commands } = Args::parse();

    // Initialize the tracing subscriber
    init_tracing_subscriber(v)?;

    match commands {
        SubCommands::Migrate { datadir, output } => {
            tracing::info!(
                target: "opdb-cli",
                "Starting database migration from {} to {}...",
                datadir.display(),
                output.display()
            );
            // TODO(clabby): Hook up to migrator
        }
    }

    Ok(())
}

/// Initializes the tracing subscriber
///
/// # Arguments
/// * `verbosity_level` - The verbosity level (0-4)
///
/// # Returns
/// * `Result<()>` - Ok if successful, Err otherwise.
fn init_tracing_subscriber(verbosity_level: u8) -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match verbosity_level {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow!(e))
}
