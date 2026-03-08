use std::time::Duration;

use clap::Parser;
use game::state::MatchState;
use net::{ClientMessage, ServerMessage, Session};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::time;

mod error;

use error::{Error, Result};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:7878")]
    bind: String,
    #[arg(long, default_value_t = 1000)]
    tick_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(&args.bind)
        .await
        .map_err(|source| Error::Bind {
            address: args.bind.clone(),
            source,
        })?;

    let (turn_tx, turn_rx) = watch::channel(0_u64);
    tokio::spawn(run_turn_loop(
        Duration::from_millis(args.tick_ms),
        turn_tx.clone(),
    ));

    println!("server listening on {}", args.bind);
    accept_loop(listener, turn_rx).await?;
    Ok(())
}

async fn run_turn_loop(tick_interval: Duration, turn_tx: watch::Sender<u64>) {
    let mut ticker = time::interval(tick_interval);
    let mut state = MatchState::new();

    loop {
        ticker.tick().await;

        let turn = match state.advance_turn() {
            Ok(turn) => turn,
            Err(error) => {
                eprintln!("turn loop stopped: {}", error);
                break;
            }
        };

        if turn_tx.send(turn).is_err() {
            break;
        }
    }
}

async fn accept_loop(listener: TcpListener, turn_rx: watch::Receiver<u64>) -> Result<()> {
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let client_turn_rx = turn_rx.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, client_turn_rx).await {
                eprintln!("client {} disconnected: {}", peer_addr, error);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, mut turn_rx: watch::Receiver<u64>) -> Result<()> {
    let mut session: Session<ClientMessage, ServerMessage> = Session::new(stream);

    let initial_turn = *turn_rx.borrow_and_update();
    send_turn_update(&mut session, initial_turn).await?;

    loop {
        tokio::select! {
            change_result = turn_rx.changed() => {
                if change_result.is_err() {
                    return Ok(());
                }
                let current_turn = *turn_rx.borrow_and_update();
                send_turn_update(&mut session, current_turn).await?;
            }
            message_result = session.recv() => {
                match message_result? {
                    Some(ClientMessage::Ping) => {
                        session.send(&ServerMessage::Pong).await?;
                    }
                    None => return Ok(()),
                }
            }
        }
    }
}

async fn send_turn_update(
    session: &mut Session<ClientMessage, ServerMessage>,
    turn: u64,
) -> Result<()> {
    session
        .send(&ServerMessage::TurnAdvanced { turn })
        .await
        .map_err(Into::into)
}
