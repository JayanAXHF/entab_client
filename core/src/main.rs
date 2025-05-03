mod login;
use clap::Parser;
use client_core::AssignmentType;
use crossterm::cursor::SavePosition;
use crossterm::ExecutableCommand;
use std::io::stdout;
use std::process::exit;
use tracing_subscriber::field::MakeExt;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    login: bool,

    #[arg(short, long, default_value_t = AssignmentType::Homework)]
    type_: AssignmentType,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _cleanup = client_core::Cleanup;
    let args = Args::parse();
    if args.login {
        login::Login::login().await?;
        exit(0);
    }

    stdout().execute(SavePosition)?;
    tracing_subscriber::fmt()
        .map_fmt_fields(|f| f.debug_alt())
        .init();
    let mut app = client_core::App::new(args.type_).await;
    app.run().await?;
    Ok(())
}
