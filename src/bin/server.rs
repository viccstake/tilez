use clap::{ArgAction, Parser};
use seb_mul_game::logger::Logger;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name    = "server",
    version,
    about   = "Seb n Vic Multiplayer Game — dedicated server",
    long_about = "Accepts pairs of TCP clients and runs authoritative game sessions.\n\
                  Protocol is line-delimited UTF-8; see src/bin/server.rs for the full spec."
)]
struct Args {
    /// Address to listen on
    #[arg(short, long, default_value = "0.0.0.0:7878")]
    bind: String,

    /// Increase output verbosity (-v verbose, -vv debug, -vvv trace)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Maximum number of games that can run concurrently
    #[arg(short = 'g', long, default_value_t = 16)]
    max_games: u32,
}

// ── DISPLAY EVENTS ────────────────────────────────────────────────────────────
//
// Every loggable occurrence is an `Event` variant.  Implementing `Display`
// here means the logger receives a rich, human-readable message while still
// using Rust's zero-cost formatting machinery (no allocation until a variant
// is actually emitted at the current verbosity level).

enum Event {
    Listening      { addr: String },
    WaitingForPair { game_id: u32 },
    PlayerConnected { n: u8, game_id: u32, addr: SocketAddr },
    GameStarted    { game_id: u32 },
    GameEnded      { game_id: u32 },
    PlayerMsg      { game_id: u32, player: u8, msg: String },
    PlayerDisconnected { game_id: u32, player: u8 },
    InvalidCmd     { game_id: u32, player: u8, raw: String },
    AcceptError    { reason: String },
    SlotsFull,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Listening { addr } =>
                write!(f, "Server listening on {addr}"),
            Event::WaitingForPair { game_id } =>
                write!(f, "[game {game_id}] Waiting for two players to connect"),
            Event::PlayerConnected { n, game_id, addr } =>
                write!(f, "[game {game_id}] Player {n} connected from {addr}"),
            Event::GameStarted { game_id } =>
                write!(f, "[game {game_id}] Game started"),
            Event::GameEnded { game_id } =>
                write!(f, "[game {game_id}] Game ended"),
            Event::PlayerMsg { game_id, player, msg } =>
                write!(f, "[game {game_id}] P{player} → {msg}"),
            Event::PlayerDisconnected { game_id, player } =>
                write!(f, "[game {game_id}] Player {player} disconnected"),
            Event::InvalidCmd { game_id, player, raw } =>
                write!(f, "[game {game_id}] P{player} sent unrecognised command: {raw:?}"),
            Event::AcceptError { reason } =>
                write!(f, "Accept error: {reason}"),
            Event::SlotsFull =>
                write!(f, "Max concurrent games reached — new connections will queue"),
        }
    }
}

// ── PROTOCOL SPEC ─────────────────────────────────────────────────────────────
//
// Client → Server (one line per message):
//   PLACE <x> <y> <radius>
//   SHOOT <piece_index> <dx> <dy> <force>
//
// Server → Client (one line per message):
//   WAITING                — holding for second player
//   READY <player_id>      — game begins; your id is 0 or 1
//   YOUR_TURN
//   OPPONENT_TURN
//   OK                     — move accepted
//   ERROR <reason>         — move rejected; try again
//   STATE <n> [<owner> <x> <y> <r>]×n
//   DISCONNECTED           — opponent left; game over

// ── CLIENT COMMANDS ───────────────────────────────────────────────────────────

#[derive(Debug)]
enum ClientCmd {
    Place { x: f32, y: f32, radius: f32 },
    Shoot { index: usize, dx: f32, dy: f32, force: f32 },
}

impl ClientCmd {
    fn parse(line: &str) -> Option<Self> {
        let mut t = line.split_whitespace();
        match t.next()? {
            "PLACE" => Some(Self::Place {
                x:      t.next()?.parse().ok()?,
                y:      t.next()?.parse().ok()?,
                radius: t.next()?.parse().ok()?,
            }),
            "SHOOT" => Some(Self::Shoot {
                index: t.next()?.parse().ok()?,
                dx:    t.next()?.parse().ok()?,
                dy:    t.next()?.parse().ok()?,
                force: t.next()?.parse().ok()?,
            }),
            _ => None,
        }
    }
}

// ── AUTHORITATIVE GAME STATE ──────────────────────────────────────────────────

#[derive(Clone)]
struct Piece {
    owner:  u8,
    x:      f32,
    y:      f32,
    radius: f32,
}

/// Piece serialises as `<owner> <x> <y> <radius>` — embedded directly into
/// the `STATE` line that is broadcast to both players after every move.
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:.3} {:.3} {:.3}", self.owner, self.x, self.y, self.radius)
    }
}

struct GameState {
    pieces: Vec<Piece>,
    turn:   u8,     // 0 or 1
}

impl GameState {
    fn new() -> Self {
        Self { pieces: Vec::new(), turn: 0 }
    }

    /// Full board serialised as a server message ready to write to a socket.
    fn state_line(&self) -> String {
        let body: Vec<String> = self.pieces.iter().map(|p| p.to_string()).collect();
        format!("STATE {} {}\n", self.pieces.len(), body.join(" "))
    }

    fn place(&mut self, owner: u8, x: f32, y: f32, radius: f32) -> Result<(), &'static str> {
        if owner != self.turn {
            return Err("not your turn");
        }
        if radius <= 0.0 {
            return Err("radius must be positive");
        }
        for p in &self.pieces {
            let dist = ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt();
            if dist < p.radius + radius {
                return Err("overlaps an existing piece");
            }
        }
        self.pieces.push(Piece { owner, x, y, radius });
        self.turn = 1 - self.turn;
        Ok(())
    }

    fn shoot(
        &mut self,
        owner: u8,
        index: usize,
        dx: f32,
        dy: f32,
        force: f32,
    ) -> Result<(), &'static str> {
        if owner != self.turn {
            return Err("not your turn");
        }
        let len = (dx * dx + dy * dy).sqrt();
        if len < f32::EPSILON {
            return Err("direction vector must be non-zero");
        }
        let piece = self.pieces.get(index).ok_or("piece index out of range")?;
        if piece.owner != owner {
            return Err("that piece does not belong to you");
        }
        let p = &mut self.pieces[index];
        p.x += (dx / len) * force;
        p.y += (dy / len) * force;
        self.turn = 1 - self.turn;
        Ok(())
    }
}

// ── PER-GAME SESSION ──────────────────────────────────────────────────────────

async fn run_game(
    s1: TcpStream,
    a1: SocketAddr,
    s2: TcpStream,
    a2: SocketAddr,
    game_id: u32,
    log: Arc<Logger>,
) {
    log.info(Event::PlayerConnected { n: 1, game_id, addr: a1 });
    log.info(Event::PlayerConnected { n: 2, game_id, addr: a2 });
    log.info(Event::GameStarted { game_id });

    let (r1, mut w1) = tokio::io::split(s1);
    let (r2, mut w2) = tokio::io::split(s2);
    let mut lines1 = BufReader::new(r1).lines();
    let mut lines2 = BufReader::new(r2).lines();

    // Announce game start and initial turn order.
    let _ = w1.write_all(b"READY 0\nYOUR_TURN\n").await;
    let _ = w2.write_all(b"READY 1\nOPPONENT_TURN\n").await;

    let mut state = GameState::new();

    loop {
        // Poll both streams; whichever produces a line first wins this tick.
        // tokio::select! is cancellation-safe here: BufReader preserves any
        // partially buffered data if a branch is dropped.
        let (line, player) = tokio::select! {
            res = lines1.next_line() => match res {
                Ok(Some(l)) => (l, 0u8),
                _ => {
                    log.info(Event::PlayerDisconnected { game_id, player: 0 });
                    let _ = w2.write_all(b"DISCONNECTED\n").await;
                    break;
                }
            },
            res = lines2.next_line() => match res {
                Ok(Some(l)) => (l, 1u8),
                _ => {
                    log.info(Event::PlayerDisconnected { game_id, player: 1 });
                    let _ = w1.write_all(b"DISCONNECTED\n").await;
                    break;
                }
            },
        };

        let trimmed = line.trim().to_string();
        log.verbose(Event::PlayerMsg { game_id, player, msg: trimmed.clone() });

        // Reject out-of-turn messages without advancing state.
        if player != state.turn {
            let reply = format!("ERROR not your turn\n");
            let w = if player == 0 { &mut w1 } else { &mut w2 };
            let _ = w.write_all(reply.as_bytes()).await;
            continue;
        }

        let result = match ClientCmd::parse(&trimmed) {
            Some(ClientCmd::Place { x, y, radius }) => {
                log.debug(format!("[game {game_id}] P{player} PLACE x={x:.3} y={y:.3} r={radius:.3}"));
                state.place(player, x, y, radius)
            }
            Some(ClientCmd::Shoot { index, dx, dy, force }) => {
                log.debug(format!("[game {game_id}] P{player} SHOOT #{index} dir=({dx:.3},{dy:.3}) force={force:.3}"));
                state.shoot(player, index, dx, dy, force)
            }
            None => {
                log.warn(Event::InvalidCmd { game_id, player, raw: trimmed.clone() });
                Err("unrecognised command")
            }
        };

        match result {
            Ok(()) => {
                let state_msg = state.state_line();
                log.trace(format!("[game {game_id}] {state_msg}"));
                let _ = w1.write_all(b"OK\n").await;
                let _ = w2.write_all(b"OK\n").await;
                let _ = w1.write_all(state_msg.as_bytes()).await;
                let _ = w2.write_all(state_msg.as_bytes()).await;
                // Signal the new active player.
                if state.turn == 0 {
                    let _ = w1.write_all(b"YOUR_TURN\n").await;
                    let _ = w2.write_all(b"OPPONENT_TURN\n").await;
                } else {
                    let _ = w1.write_all(b"OPPONENT_TURN\n").await;
                    let _ = w2.write_all(b"YOUR_TURN\n").await;
                }
            }
            Err(reason) => {
                let err = format!("ERROR {reason}\n");
                let w = if player == 0 { &mut w1 } else { &mut w2 };
                let _ = w.write_all(err.as_bytes()).await;
            }
        }
    }

    log.info(Event::GameEnded { game_id });
}

// ── ENTRY POINT ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let log  = Arc::new(Logger::new(args.verbose));

    let max_games = args.max_games.max(1) as usize;
    let slots = Arc::new(Semaphore::new(max_games));

    let listener = TcpListener::bind(&args.bind).await.unwrap_or_else(|e| {
        eprintln!("Failed to bind to {}: {e}", args.bind);
        std::process::exit(1);
    });

    log.info(Event::Listening { addr: args.bind.clone() });
    log.verbose(format!("Max concurrent games: {max_games}"));

    let game_counter = Arc::new(AtomicU32::new(0));

    loop {
        // Acquire a game slot before accepting connections.
        // When every slot is occupied the loop pauses here,
        // naturally back-pressuring new TCP connections.
        let permit = match Arc::clone(&slots).acquire_owned().await {
            Ok(p)  => p,
            Err(_) => break,
        };

        let game_id = game_counter.fetch_add(1, Ordering::Relaxed);
        log.verbose(Event::WaitingForPair { game_id });

        // Accept first player and tell them to hold.
        let (mut s1, a1) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e)   => {
                log.warn(Event::AcceptError { reason: e.to_string() });
                drop(permit);
                continue;
            }
        };
        let _ = s1.write_all(b"WAITING\n").await;

        if slots.available_permits() == 0 {
            log.verbose(Event::SlotsFull);
        }

        // Accept second player.
        let (s2, a2) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e)   => {
                log.warn(Event::AcceptError { reason: e.to_string() });
                drop(permit);
                continue;
            }
        };

        let log_task = Arc::clone(&log);
        tokio::spawn(async move {
            // Permit is held for the lifetime of the game task.
            let _permit = permit;
            run_game(s1, a1, s2, a2, game_id, log_task).await;
        });
    }
}
