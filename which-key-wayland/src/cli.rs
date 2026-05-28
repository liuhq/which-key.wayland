use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug, PartialEq)]
pub enum SubCommand {
    Show,
    Quit,
    /// Force reload configuration file
    Reload,
}

#[derive(Parser, Debug)]
#[command(name = "which-key-wayland")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
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
        assert_eq!(cli.command, Some(SubCommand::Show));
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
