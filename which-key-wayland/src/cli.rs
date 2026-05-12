use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum SubCommand {
    Show,
    Quit,
}

#[derive(Parser)]
#[command(name = "which-key-wayland")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}
