use clap::{Parser, Subcommand};

use crate::keybind::key::Key;

#[derive(Subcommand, Debug, PartialEq)]
pub enum SubCommand {
    /// Show which-key pannel
    Show {
        /// Show the children of this first-level group key
        #[arg(value_name = "KEY", value_parser = parse_key)]
        key: Option<String>,
    },
    /// Quit which-key-wayland
    Quit,
    /// Force reload configuration file
    Reload,
}

#[derive(Parser, Debug)]
#[command(name = "which-key-wayland", version)]
/// A key-hint panel for Wayland, inspired by the Neovim plugin which-key.nvim and the Helix editor style.
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

fn parse_key(value: &str) -> Result<String, String> {
    value
        .parse::<Key>()
        .map(|_| value.to_string())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_no_subcommand() {
        let cli = Cli::try_parse_from(["which-key-wayland"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn parse_show_subcommand() {
        let cli = Cli::try_parse_from(["which-key-wayland", "show"]).unwrap();
        assert_eq!(cli.command, Some(SubCommand::Show { key: None }));
    }

    #[test]
    fn parse_show_with_key() {
        let cli = Cli::try_parse_from(["which-key-wayland", "show", "Ctrl+a"]).unwrap();
        assert_eq!(
            cli.command,
            Some(SubCommand::Show {
                key: Some("Ctrl+a".to_string())
            })
        );
    }

    #[test]
    fn parse_show_with_invalid_key() {
        let result = Cli::try_parse_from(["which-key-wayland", "show", "Ctrl++a"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_show_with_too_many_keys() {
        let result = Cli::try_parse_from(["which-key-wayland", "show", "a", "b"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_quit_subcommand() {
        let cli = Cli::try_parse_from(["which-key-wayland", "quit"]).unwrap();
        assert_eq!(cli.command, Some(SubCommand::Quit));
    }

    #[test]
    fn parse_reload_subcommand() {
        let cli = Cli::try_parse_from(["which-key-wayland", "reload"]).unwrap();
        assert_eq!(cli.command, Some(SubCommand::Reload));
    }

    #[test]
    fn parse_invalid_subcommand() {
        let result = Cli::try_parse_from(["which-key-wayland", "invalid"]);
        assert!(result.is_err());
    }
}
