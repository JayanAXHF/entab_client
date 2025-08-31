use clap::{CommandFactory, Parser};
use cli::Cli;
use client_core::login;
use color_eyre::Result;
use std::io;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    if let Some(command) = args.command {
        match command {
            cli::Command::Completions { shell } => {
                let mut cmd = Cli::command();
                let name = cmd.get_name().to_string();
                clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
                return Ok(());
            }
        }
    }

    if args.login {
        login::Login::login(args.store_credentials, args.fetch_credentials)
            .await
            .expect("Failed to login");
    }
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
