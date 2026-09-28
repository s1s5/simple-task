pub mod cli;
pub mod config;
pub mod error;
pub mod handoff;
pub mod mcp;
pub mod store;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "simple-task",
    version,
    about = "タスク登録 + worker ハンドオフ MCP"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Mcp,
    List,
    Show { slug: String },
    Path,
}

pub async fn dispatch(cli: Cli) -> error::Result<()> {
    let config = config::Config::from_env()?;
    match cli.command.unwrap_or(Command::Mcp) {
        Command::Mcp => mcp::run(config).await,
        Command::List => cli::list(&config),
        Command::Show { slug } => cli::show(&config, &slug),
        Command::Path => cli::path(&config),
    }
}
