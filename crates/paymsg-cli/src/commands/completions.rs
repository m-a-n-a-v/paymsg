//! Completions command implementation - generate shell completions.

use anyhow::Result;
use clap::{Command, Parser};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Parser)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: Shell,
}

/// Execute the completions command
pub fn execute(args: CompletionsArgs, cmd: &mut Command) -> Result<()> {
    generate(args.shell, cmd, "paymsg", &mut io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_variants() {
        // Just ensure the Shell enum works
        let shells = vec![Shell::Bash, Shell::Zsh, Shell::Fish];
        assert_eq!(shells.len(), 3);
    }
}
