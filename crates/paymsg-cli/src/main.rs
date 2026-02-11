//! CLI binary for paymsg.

use clap::Parser;

#[derive(Parser)]
#[command(name = "paymsg")]
#[command(about = "Parse, validate, translate, and generate ISO 20022 and SWIFT MT messages")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    println!("paymsg CLI - Coming soon!");

    Ok(())
}
