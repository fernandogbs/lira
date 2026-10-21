use clap::{Parser, Subcommand};
use lira_core::ipc::{client_request, socket_path, Command};
use lira_core::settings::Settings;

#[derive(Parser)]
#[command(name = "lira", about = "Lira voice assistant control")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Toggle,
    Start,
    Stop,
    Status,
}

#[tokio::main]
async fn main() {
    let locale = Settings::load().unwrap_or_default().ui.locale.resolve();
    let cli = Cli::parse();
    let cmd = match cli.command {
        Commands::Toggle => Command::Toggle,
        Commands::Start => Command::Start,
        Commands::Stop => Command::Stop,
        Commands::Status => Command::Status,
    };

    let path = socket_path();
    if !path.exists() {
        eprintln!("{}", locale.not_running());
        std::process::exit(1);
    }

    match client_request(&path, cmd).await {
        Ok(resp) => {
            println!("{}", locale.status_line(resp.state));
            if let Some(msg) = resp.message {
                eprintln!("{}", msg);
            }
            if !resp.ok {
                std::process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("{}", locale.not_running());
            std::process::exit(1);
        }
    }
}
