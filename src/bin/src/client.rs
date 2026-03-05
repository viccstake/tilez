use clap::Parser;
use net::{ClientMessage, ServerMessage, Session};
use tokio::net::TcpStream;

mod error;

use error::{Error, Result};

#[derive(Debug, Parser)]
struct Args {
    #[arg(default_value = "127.0.0.1:7878")]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let stream = TcpStream::connect(&args.server)
        .await
        .map_err(|source| Error::Connect {
            address: args.server.clone(),
            source,
        })?;
    let mut session: Session<ServerMessage, ClientMessage> = Session::new(stream);

    println!("connected to {}", args.server);
    while let Some(message) = session.recv().await? {
        match message {
            ServerMessage::TurnAdvanced { turn } => {
                println!("turn advanced to {}", turn);
            }
            ServerMessage::Pong => {
                println!("received pong");
            }
        }
    }

    println!("server closed connection");
    Ok(())
}
