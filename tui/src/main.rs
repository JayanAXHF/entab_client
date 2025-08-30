use clap::Parser;
use cli::Cli;
use client_core::login;
use color_eyre::Result;

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
    if args.login {
        login::Login::login(args.store_credentials, args.fetch_credentials)
            .await
            .expect("Failed to login");
    }
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
