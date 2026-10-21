use clap::{Parser, Subcommand};
use lira_core::ipc::{client_request, socket_path, Command};
use lira_core::settings::Settings;
use lira_store::Store;

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
    /// List known agent sessions (local store + discovery tip)
    Sessions,
}

#[tokio::main]
async fn main() {
    let locale = Settings::load().unwrap_or_default().ui.locale.resolve();
    let cli = Cli::parse();

    if matches!(cli.command, Commands::Sessions) {
        match Store::open_default() {
            Ok(store) => match store.list_sessions() {
                Ok(list) if list.is_empty() => {
                    println!("(no sessions in store yet — speak to an agent first)");
                }
                Ok(list) => {
                    for s in list {
                        println!("{}\t{}\t{}", s.id, s.title, s.adapter);
                    }
                }
                Err(err) => {
                    eprintln!("store error: {err}");
                    std::process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("store error: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    let cmd = match cli.command {
        Commands::Toggle => Command::Toggle,
        Commands::Start => Command::Start,
        Commands::Stop => Command::Stop,
        Commands::Status => Command::Status,
        Commands::Sessions => unreachable!(),
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
