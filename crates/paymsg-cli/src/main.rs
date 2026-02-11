//! CLI binary for paymsg.

use clap::{CommandFactory, Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "paymsg")]
#[command(about = "Parse, validate, translate, and generate ISO 20022 and SWIFT MT messages", long_about = None)]
#[command(version)]
#[command(after_help = r#"EXAMPLES:
  # Parse an MT103 message from file
  paymsg parse mt103.txt --pretty

  # Validate a pacs.008 message
  paymsg validate pacs008.xml --severity error

  # Translate MT103 to pacs.008
  paymsg translate mt103.txt --to pacs008

  # Get message info without full parsing
  paymsg info mt103.txt

  # Generate shell completions
  paymsg completions bash > paymsg-completion.bash

For more information, visit: https://github.com/your-org/paymsg
"#)]
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
    #[command(after_help = "EXAMPLES:\n  paymsg parse mt103.txt --pretty\n  cat message.xml | paymsg parse --format mx")]
    Parse(commands::parse::ParseArgs),

    /// Validate a message against schema and business rules
    #[command(after_help = "EXAMPLES:\n  paymsg validate pacs008.xml\n  paymsg validate mt103.txt --severity error --rules amount,date")]
    Validate(commands::validate::ValidateArgs),

    /// Translate a message between MT and MX formats
    #[command(after_help = "EXAMPLES:\n  paymsg translate mt103.txt --to pacs008\n  paymsg translate pacs008.xml --to mt103 --validate")]
    Translate(commands::translate::TranslateArgs),

    /// Show message information without full parsing
    #[command(after_help = "EXAMPLES:\n  paymsg info mt103.txt\n  cat message.xml | paymsg info")]
    Info(commands::info::InfoArgs),

    /// Generate shell completions
    #[command(after_help = "EXAMPLES:\n  paymsg completions bash > paymsg-completion.bash\n  paymsg completions zsh > _paymsg")]
    Completions(commands::completions::CompletionsArgs),
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
        Commands::Info(args) => commands::info::execute(args),
        Commands::Completions(args) => {
            let mut cmd = Cli::command();
            commands::completions::execute(args, &mut cmd)
        }
    }
}
