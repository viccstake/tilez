use clap::{ArgAction, Parser};
use seb_mul_game::logger::Logger;
use std::fmt;
use std::io::{self, Write as _};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name    = "client",
    version,
    about   = "Seb n Vic Multiplayer Game — terminal client",
    long_about = "Connects to a running game server and plays interactively.\n\
                  Commands (type when it is your turn):\n  \
                    place <x> <y> <radius>\n  \
                    shoot <piece#> <dx> <dy> <force>"
)]
struct Args {
    /// Server address to connect to
    #[arg(default_value = "127.0.0.1:7878")]
    addr: String,

    /// Increase output verbosity (-v verbose, -vv debug, -vvv trace)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

// ── CLIENT EVENTS (operational logging to stderr) ─────────────────────────────

enum ClientEvent<'a> {
    Connecting { addr: &'a str },
    Connected  { addr: &'a str },
    Sending    { cmd: &'a str },
    Received   { raw: &'a str },
    Disconnected,
}

impl fmt::Display for ClientEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientEvent::Connecting { addr }  => write!(f, "Connecting to {addr}…"),
            ClientEvent::Connected  { addr }  => write!(f, "Connected to {addr}"),
            ClientEvent::Sending    { cmd }   => write!(f, "→ {cmd}"),
            ClientEvent::Received   { raw }   => write!(f, "← {raw}"),
            ClientEvent::Disconnected         => write!(f, "Connection closed by server"),
        }
    }
}

// ── BOARD STATE ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Piece {
    index:  usize,
    owner:  u8,
    x:      f32,
    y:      f32,
    radius: f32,
}

struct BoardState {
    pieces: Vec<Piece>,
}

impl BoardState {
    /// Parse the payload after `STATE <n> `.
    fn parse(line: &str) -> Option<Self> {
        let mut t = line.split_whitespace();
        let n: usize = t.next()?.parse().ok()?;
        let mut pieces = Vec::with_capacity(n);
        for index in 0..n {
            pieces.push(Piece {
                index,
                owner:  t.next()?.parse().ok()?,
                x:      t.next()?.parse().ok()?,
                y:      t.next()?.parse().ok()?,
                radius: t.next()?.parse().ok()?,
            });
        }
        Some(Self { pieces })
    }
}

/// Piece renders as a compact single-line summary.
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  #{:<2}  P{}  pos=({:>8.2}, {:>8.2})  radius={:.2}",
            self.index, self.owner, self.x, self.y, self.radius
        )
    }
}

/// Board renders as a labelled list of all pieces.
impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pieces.is_empty() {
            return write!(f, "  (board is empty)");
        }
        for piece in &self.pieces {
            writeln!(f, "{piece}")?;
        }
        Ok(())
    }
}

// ── SERVER MESSAGES ───────────────────────────────────────────────────────────

enum ServerMsg {
    Waiting,
    Ready      { player_id: u8 },
    YourTurn,
    OpponentTurn,
    Ok,
    Error      (String),
    State      (BoardState),
    Disconnected,
    Unknown    (String),
}

impl ServerMsg {
    fn parse(line: &str) -> Self {
        if line == "WAITING"        { return Self::Waiting; }
        if line == "YOUR_TURN"      { return Self::YourTurn; }
        if line == "OPPONENT_TURN"  { return Self::OpponentTurn; }
        if line == "OK"             { return Self::Ok; }
        if line == "DISCONNECTED"   { return Self::Disconnected; }

        if let Some(rest) = line.strip_prefix("READY ") {
            if let Ok(id) = rest.trim().parse::<u8>() {
                return Self::Ready { player_id: id };
            }
        }
        if let Some(rest) = line.strip_prefix("ERROR ") {
            return Self::Error(rest.trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("STATE ") {
            if let Some(board) = BoardState::parse(rest) {
                return Self::State(board);
            }
        }
        Self::Unknown(line.to_string())
    }
}

/// Each server message knows how to display itself to the player.
impl fmt::Display for ServerMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerMsg::Waiting =>
                write!(f, "Waiting for a second player to connect…"),
            ServerMsg::Ready { player_id } =>
                write!(f, "Game on!  You are Player {player_id}."),
            ServerMsg::YourTurn =>
                write!(f, ""),          // prompt is printed separately
            ServerMsg::OpponentTurn =>
                write!(f, "Opponent's turn — waiting…"),
            ServerMsg::Ok =>
                write!(f, "Move accepted."),
            ServerMsg::Error(reason) =>
                write!(f, "Rejected: {reason}"),
            ServerMsg::State(board) =>
                write!(f, "Board:\n{board}"),
            ServerMsg::Disconnected =>
                write!(f, "Opponent disconnected.  Game over."),
            ServerMsg::Unknown(raw) =>
                write!(f, "(unknown message: {raw:?})"),
        }
    }
}

// ── USER INPUT ────────────────────────────────────────────────────────────────

/// A validated command ready to be sent over the wire.
enum Cmd {
    Place { x: f32, y: f32, radius: f32 },
    Shoot { index: usize, dx: f32, dy: f32, force: f32 },
}

impl Cmd {
    /// Parse a line typed by the player (case-insensitive keyword).
    fn parse(raw: &str) -> Result<Self, String> {
        let mut t = raw.split_whitespace();
        match t.next().unwrap_or("").to_ascii_uppercase().as_str() {
            "PLACE" => {
                let x      = parse_f32(&mut t, "x")?;
                let y      = parse_f32(&mut t, "y")?;
                let radius = parse_f32(&mut t, "radius")?;
                if radius <= 0.0 {
                    return Err("radius must be > 0".into());
                }
                Ok(Self::Place { x, y, radius })
            }
            "SHOOT" => {
                let index = t.next()
                    .ok_or("missing piece index")?
                    .parse::<usize>()
                    .map_err(|_| "piece index must be a whole number".to_string())?;
                let dx    = parse_f32(&mut t, "dx")?;
                let dy    = parse_f32(&mut t, "dy")?;
                let force = parse_f32(&mut t, "force")?;
                if force <= 0.0 {
                    return Err("force must be > 0".into());
                }
                Ok(Self::Shoot { index, dx, dy, force })
            }
            "" => Err("empty input".into()),
            kw => Err(format!("unknown command '{kw}'")),
        }
    }

    /// Serialise to the wire format expected by the server.
    fn to_wire(&self) -> String {
        match self {
            Self::Place { x, y, radius } =>
                format!("PLACE {x} {y} {radius}\n"),
            Self::Shoot { index, dx, dy, force } =>
                format!("SHOOT {index} {dx} {dy} {force}\n"),
        }
    }
}

fn parse_f32<'a>(
    t: &mut impl Iterator<Item = &'a str>,
    name: &str,
) -> Result<f32, String> {
    t.next()
        .ok_or_else(|| format!("missing {name}"))?
        .parse::<f32>()
        .map_err(|_| format!("{name} must be a number"))
}

// ── PROMPT ────────────────────────────────────────────────────────────────────

fn print_prompt(player_id: u8) {
    print!("\nP{player_id}> ");
    io::stdout().flush().ok();
}

fn print_help() {
    println!("  Commands:");
    println!("    place <x> <y> <radius>          — place a new piece");
    println!("    shoot <piece#> <dx> <dy> <force> — shoot an existing piece");
}

// ── MAIN ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let log  = Logger::new(args.verbose);

    log.info(ClientEvent::Connecting { addr: &args.addr });

    let stream = match TcpStream::connect(&args.addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to {}: {e}", args.addr);
            std::process::exit(1);
        }
    };

    log.info(ClientEvent::Connected { addr: &args.addr });

    let (reader, mut writer) = tokio::io::split(stream);
    let mut server_lines = BufReader::new(reader).lines();
    let mut stdin_lines  = BufReader::new(tokio::io::stdin()).lines();

    // Game state tracked client-side.
    let mut player_id: u8 = 0;
    let mut my_turn       = false;

    loop {
        tokio::select! {
            // ── Server → Client ───────────────────────────────────────────────
            result = server_lines.next_line() => {
                let raw = match result {
                    Ok(Some(l)) => l,
                    _ => {
                        log.info(ClientEvent::Disconnected);
                        println!("\nDisconnected from server.");
                        break;
                    }
                };

                log.trace(ClientEvent::Received { raw: &raw });

                let msg = ServerMsg::parse(raw.trim());

                match &msg {
                    ServerMsg::Ready { player_id: id } => {
                        player_id = *id;
                        println!("\n{msg}");
                        print_help();
                    }
                    ServerMsg::YourTurn => {
                        my_turn = true;
                        print_prompt(player_id);
                    }
                    ServerMsg::Error(_) => {
                        println!("\n{msg}");
                        // Turn stays with us; re-prompt.
                        if my_turn {
                            print_prompt(player_id);
                        }
                    }
                    ServerMsg::Disconnected => {
                        println!("\n{msg}");
                        break;
                    }
                    ServerMsg::OpponentTurn => {
                        my_turn = false;
                        println!("\n{msg}");
                    }
                    ServerMsg::Ok => {
                        // Followed immediately by STATE; don't print yet.
                        log.verbose(format!("server acknowledged move"));
                    }
                    ServerMsg::State(_) | ServerMsg::Waiting | ServerMsg::Unknown(_) => {
                        println!("\n{msg}");
                    }
                }
            }

            // ── Stdin → Server (only when it is our turn) ─────────────────────
            result = stdin_lines.next_line(), if my_turn => {
                let raw = match result {
                    Ok(Some(l)) => l,
                    _ => {
                        println!("\nInput closed.");
                        break;
                    }
                };

                let trimmed = raw.trim();

                if trimmed.is_empty() {
                    print_prompt(player_id);
                    continue;
                }

                if matches!(trimmed.to_ascii_uppercase().as_str(), "HELP" | "?") {
                    print_help();
                    print_prompt(player_id);
                    continue;
                }

                match Cmd::parse(trimmed) {
                    Ok(cmd) => {
                        let wire = cmd.to_wire();
                        log.verbose(ClientEvent::Sending { cmd: wire.trim_end() });
                        if writer.write_all(wire.as_bytes()).await.is_err() {
                            eprintln!("Failed to send command.");
                            break;
                        }
                        // Disable stdin until the server responds (OK or ERROR).
                        my_turn = false;
                    }
                    Err(reason) => {
                        println!("  ? {reason}");
                        print_help();
                        print_prompt(player_id);
                    }
                }
            }
        }
    }
}
