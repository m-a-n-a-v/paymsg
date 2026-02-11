//! CLI binary for paymsg.

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "paymsg")]
#[command(about = "Parse, validate, translate, and generate ISO 20022 and SWIFT MT messages")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a message and output structured JSON
    Parse(commands::parse::ParseArgs),
    /// Validate a message against schema and business rules
    Validate(commands::validate::ValidateArgs),
    /// Translate a message between MT and MX formats
    Translate(commands::translate::TranslateArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Execute command
    match cli.command {
        Commands::Parse(args) => commands::parse::execute(args),
        Commands::Validate(args) => commands::validate::execute(args),
        Commands::Translate(args) => commands::translate::execute(args),
    }
}
