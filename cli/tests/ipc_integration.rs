use std::sync::{Arc, Mutex};

use lira_core::ipc::{client_request, Command, CommandHandler, IpcServer, Response};
use lira_core::AppState;
use tempfile::tempdir;

#[tokio::test]
async fn cli_talks_to_temp_socket() {
    let dir = tempdir().unwrap();
    let sock = dir.path().join("lira.sock");
    let state = Arc::new(Mutex::new(AppState::Idle));
    let handler: CommandHandler = Arc::new({
        let state = Arc::clone(&state);
        move |cmd| {
            let mut s = state.lock().unwrap();
            match cmd {
                Command::Toggle => *s = s.toggle(),
                Command::Start => *s = AppState::Listening,
                Command::Stop => *s = AppState::Idle,
                Command::Status => {}
            }
            Response::ok(*s)
        }
    });

    let server = IpcServer::bind(&sock).await.unwrap();
    let h = Arc::clone(&handler);
    tokio::spawn(async move {
        for _ in 0..2 {
            let _ = server.accept_one(&h).await;
        }
    });

    let resp = client_request(&sock, Command::Toggle).await.unwrap();
    assert_eq!(resp.state, AppState::Listening);

    let resp = client_request(&sock, Command::Status).await.unwrap();
    assert_eq!(resp.state, AppState::Listening);
}
