use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use naval_game::game::orders::Order;
use naval_game::game::systems::resolution::{resolve_turn, ShipState};
use naval_game::net::protocol::{ClientMessage, GameSnapshot, ServerMessage, ShipSnapshot};
use naval_game::net::Session;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "server")]
struct Args {
    #[arg(short, long, default_value_t = 7878)]
    port: u16,

    /// Number of players required to start a game
    #[arg(short = 'n', long, default_value_t = 2)]
    min_players: usize,
}

// ── server state ──────────────────────────────────────────────────────────────

struct PlayerSlot {
    id: u32,
    name: String,
    tx: mpsc::UnboundedSender<ServerMessage>,
    connected: bool,
    has_submitted: bool,
}

#[derive(Debug, PartialEq)]
enum Phase {
    Lobby,
    Planning,
}

struct GameServer {
    min_players: usize,
    max_players: usize,
    next_player_id: u32,
    next_ship_id: u32,
    players: Vec<PlayerSlot>,
    phase: Phase,
    turn: u32,
    pending_orders: HashMap<u32, Vec<Order>>,
    ships: Vec<ShipState>,
}

impl GameServer {
    fn new(min_players: usize, max_players: usize) -> Self {
        Self {
            min_players,
            max_players,
            next_player_id: 0,
            next_ship_id: 0,
            players: Vec::new(),
            phase: Phase::Lobby,
            turn: 0,
            pending_orders: HashMap::new(),
            ships: Vec::new(),
        }
    }

    fn try_join(
        &mut self,
        name: String,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<u32, String> {
        if self.phase != Phase::Lobby {
            return Err("Game already in progress".into());
        }
        if self.players.iter().filter(|p| p.connected).count() >= self.max_players {
            return Err("Server is full".into());
        }
        let id = self.next_player_id;
        self.next_player_id += 1;
        self.players.push(PlayerSlot {
            id,
            name,
            tx,
            connected: true,
            has_submitted: false,
        });
        Ok(id)
    }

    /// Called after every Join. Starts the game once enough players are connected.
    fn try_start(&mut self) {
        if self.phase != Phase::Lobby {
            return;
        }
        if self.players.iter().filter(|p| p.connected).count() < self.min_players {
            return;
        }
        self.initialize_ships();
        self.phase = Phase::Planning;
        self.turn = 1;
        let snap_bytes = self.snapshot_bytes();
        self.broadcast(&ServerMessage::TurnStarted { turn: self.turn });
        self.broadcast(&ServerMessage::StateSnapshot(snap_bytes));
        let roster: Vec<&str> = self.players.iter()
            .filter(|p| p.connected)
            .map(|p| p.name.as_str())
            .collect();
        info!("Game started — turn {} — players: {}", self.turn, roster.join(", "));
    }

    fn submit_orders(&mut self, player_id: u32, turn: u32, orders: Vec<Order>) {
        if self.phase != Phase::Planning || turn != self.turn {
            return;
        }
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.has_submitted = true;
        }
        self.pending_orders.insert(player_id, orders);

        let all_in = self
            .players
            .iter()
            .filter(|p| p.connected)
            .all(|p| p.has_submitted);

        if all_in {
            self.resolve_and_broadcast();
        }
    }

    fn disconnect(&mut self, player_id: u32) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.connected = false;
        }
        // If Planning and the disconnected player was the last one pending, resolve now.
        if self.phase == Phase::Planning {
            let connected: Vec<&PlayerSlot> =
                self.players.iter().filter(|p| p.connected).collect();
            if connected.is_empty() {
                return;
            }
            if connected.iter().all(|p| p.has_submitted) {
                self.resolve_and_broadcast();
            }
        }
    }

    fn resolve_and_broadcast(&mut self) {
        resolve_turn(&mut self.ships, &self.pending_orders);
        self.pending_orders.clear();
        for p in &mut self.players {
            p.has_submitted = false;
        }
        let resolved = self.turn;
        self.turn += 1;

        let snap_bytes = self.snapshot_bytes();
        self.broadcast(&ServerMessage::TurnResolved { turn: resolved });
        self.broadcast(&ServerMessage::StateSnapshot(snap_bytes));
        self.broadcast(&ServerMessage::TurnStarted { turn: self.turn });
        info!("Turn {resolved} resolved — starting turn {}", self.turn);
    }

    fn broadcast(&self, msg: &ServerMessage) {
        for p in self.players.iter().filter(|p| p.connected) {
            let _ = p.tx.send(msg.clone());
        }
    }

    fn snapshot_bytes(&self) -> Vec<u8> {
        let snap = GameSnapshot {
            turn: self.turn,
            ships: self
                .ships
                .iter()
                .map(|s| ShipSnapshot {
                    id: s.id,
                    owner_id: s.owner_id,
                    q: s.q,
                    r: s.r,
                    health: s.health,
                })
                .collect(),
        };
        bincode::serialize(&snap).expect("snapshot serialization failed")
    }

    fn initialize_ships(&mut self) {
        const STARTS: [(i32, i32); 5] = [(0, -4), (0, 4), (-4, 0), (4, 0), (-4, 4)];
        self.ships.clear();
        self.next_ship_id = 0;
        // Collect ids first to avoid borrow conflict
        let ids: Vec<u32> = self
            .players
            .iter()
            .filter(|p| p.connected)
            .map(|p| p.id)
            .collect();
        for (i, owner_id) in ids.into_iter().enumerate() {
            let (q, r) = STARTS.get(i).copied().unwrap_or((i as i32 * 2, 0));
            self.ships.push(ShipState {
                id: self.next_ship_id,
                owner_id,
                q,
                r,
                health: 10,
            });
            self.next_ship_id += 1;
        }
    }
}

// ── connection handling ───────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(GameServer::new(args.min_players, 5)));

    let listener = TcpListener::bind(("0.0.0.0", args.port)).await?;
    info!(
        "Server listening on 0.0.0.0:{} — waiting for {} player(s)",
        args.port, args.min_players
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("New connection from {addr}");
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            handle_connection(stream, state).await;
        });
    }
}

async fn handle_connection(stream: TcpStream, state: Arc<Mutex<GameServer>>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    let mut session = Session::<ClientMessage, ServerMessage>::new(stream);

    // Wait for the first Join (respond to pings in the meantime)
    let player_name = loop {
        match session.recv().await {
            Ok(Some(ClientMessage::Join { player_name })) => break player_name,
            Ok(Some(ClientMessage::Ping)) => {
                session.send(&ServerMessage::Pong).await.ok();
            }
            Ok(None) | Err(_) => return,
            Ok(Some(_)) => {}
        }
    };

    // Try to register — guard must be dropped before any .await
    let join_result = {
        let mut gs = state.lock().unwrap();
        gs.try_join(player_name.clone(), tx)
    };
    let player_id = match join_result {
        Ok(id) => id,
        Err(e) => {
            warn!("Rejected '{player_name}': {e}");
            session.send(&ServerMessage::Error(e)).await.ok();
            return;
        }
    };
    info!("Player {player_id} '{player_name}' joined");

    if session
        .send(&ServerMessage::Welcome { player_id })
        .await
        .is_err()
    {
        state.lock().unwrap().disconnect(player_id);
        return;
    }

    // Start the game if we now have enough players
    state.lock().unwrap().try_start();

    // Main loop: incoming messages from the client OR outgoing messages from the server
    loop {
        tokio::select! {
            result = session.recv() => match result {
                Ok(Some(msg)) => on_message(msg, player_id, &state, &mut session).await,
                Ok(None) => break,
                Err(e) => { error!("recv error player {player_id}: {e}"); break; }
            },
            msg = rx.recv() => match msg {
                Some(m) => { if session.send(&m).await.is_err() { break; } }
                None => break,
            },
        }
    }

    state.lock().unwrap().disconnect(player_id);
    info!("Player {player_id} '{player_name}' disconnected");
}

async fn on_message(
    msg: ClientMessage,
    player_id: u32,
    state: &Arc<Mutex<GameServer>>,
    session: &mut Session<ClientMessage, ServerMessage>,
) {
    match msg {
        ClientMessage::Ping => {
            session.send(&ServerMessage::Pong).await.ok();
        }
        ClientMessage::Join { .. } => {} // already registered, ignore
        ClientMessage::SubmitOrders { turn, orders } => {
            info!(
                "Player {player_id} submitted {} order(s) for turn {turn}",
                orders.len()
            );
            state.lock().unwrap().submit_orders(player_id, turn, orders);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(min: usize, max: usize) -> GameServer {
        GameServer::new(min, max)
    }

    fn join(gs: &mut GameServer, name: &str) -> (u32, mpsc::UnboundedReceiver<ServerMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let id = gs.try_join(name.into(), tx).expect("join failed");
        (id, rx)
    }

    // ── joining ───────────────────────────────────────────────────────────────

    #[test]
    fn join_assigns_sequential_ids() {
        let mut gs = make(2, 5);
        let (id0, _) = join(&mut gs, "P0");
        let (id1, _) = join(&mut gs, "P1");
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn lobby_full_rejects_join() {
        let mut gs = make(1, 2);
        join(&mut gs, "P1");
        join(&mut gs, "P2");
        let (tx, _) = mpsc::unbounded_channel();
        assert!(gs.try_join("P3".into(), tx).is_err());
    }

    #[test]
    fn game_in_progress_rejects_join() {
        let mut gs = make(2, 5);
        join(&mut gs, "P1");
        join(&mut gs, "P2");
        gs.try_start();
        assert_eq!(gs.phase, Phase::Planning);
        let (tx, _) = mpsc::unbounded_channel();
        assert!(gs.try_join("P3".into(), tx).is_err());
    }

    // ── starting ──────────────────────────────────────────────────────────────

    #[test]
    fn try_start_below_min_stays_in_lobby() {
        let mut gs = make(2, 5);
        join(&mut gs, "P1");
        gs.try_start();
        assert_eq!(gs.phase, Phase::Lobby);
    }

    #[test]
    fn try_start_at_min_starts_game() {
        let mut gs = make(2, 5);
        join(&mut gs, "P1");
        join(&mut gs, "P2");
        gs.try_start();
        assert_eq!(gs.phase, Phase::Planning);
        assert_eq!(gs.turn, 1);
    }

    #[test]
    fn try_start_spawns_one_ship_per_player() {
        let mut gs = make(3, 5);
        join(&mut gs, "P1");
        join(&mut gs, "P2");
        join(&mut gs, "P3");
        gs.try_start();
        assert_eq!(gs.ships.len(), 3);
    }

    #[test]
    fn try_start_is_idempotent() {
        let mut gs = make(2, 5);
        join(&mut gs, "P1");
        join(&mut gs, "P2");
        gs.try_start();
        gs.try_start(); // second call must not restart
        assert_eq!(gs.turn, 1);
        assert_eq!(gs.ships.len(), 2);
    }

    // ── turn progression ──────────────────────────────────────────────────────

    #[test]
    fn all_orders_submitted_advances_turn() {
        let mut gs = make(2, 5);
        let (id1, _) = join(&mut gs, "P1");
        let (id2, _) = join(&mut gs, "P2");
        gs.try_start();

        gs.submit_orders(id1, 1, vec![]);
        assert_eq!(gs.turn, 1); // P2 hasn't submitted yet

        gs.submit_orders(id2, 1, vec![]);
        assert_eq!(gs.turn, 2); // both in — resolved
    }

    #[test]
    fn wrong_turn_number_is_ignored() {
        let mut gs = make(2, 5);
        let (id1, _) = join(&mut gs, "P1");
        let (id2, _) = join(&mut gs, "P2");
        gs.try_start();

        gs.submit_orders(id1, 99, vec![]); // wrong turn
        gs.submit_orders(id2, 1, vec![]);
        assert_eq!(gs.turn, 1); // id1's submission was ignored, not resolved yet
    }

    #[test]
    fn orders_ignored_when_not_in_planning() {
        let mut gs = make(2, 5);
        let (id1, _) = join(&mut gs, "P1");
        gs.submit_orders(id1, 0, vec![]); // still in Lobby
        assert_eq!(gs.phase, Phase::Lobby);
    }

    // ── disconnect handling ───────────────────────────────────────────────────

    #[test]
    fn disconnect_after_submit_resolves_remaining() {
        let mut gs = make(2, 5);
        let (id1, _) = join(&mut gs, "P1");
        let (id2, _) = join(&mut gs, "P2");
        gs.try_start();

        gs.submit_orders(id1, 1, vec![]); // P1 done
        gs.disconnect(id2); // P2 leaves — P1 already submitted, so resolve
        assert_eq!(gs.turn, 2);
    }

    #[test]
    fn disconnect_before_submit_waits_for_remaining() {
        let mut gs = make(3, 5);
        let (id1, _) = join(&mut gs, "P1");
        let (id2, _) = join(&mut gs, "P2");
        let (id3, _) = join(&mut gs, "P3");
        gs.try_start();

        gs.disconnect(id3); // P3 leaves, P1 and P2 still pending
        assert_eq!(gs.turn, 1);

        gs.submit_orders(id1, 1, vec![]);
        assert_eq!(gs.turn, 1); // P2 still outstanding

        gs.submit_orders(id2, 1, vec![]);
        assert_eq!(gs.turn, 2); // all remaining submitted
    }

    #[test]
    fn all_disconnect_does_not_panic_or_resolve() {
        let mut gs = make(2, 5);
        let (id1, _) = join(&mut gs, "P1");
        let (id2, _) = join(&mut gs, "P2");
        gs.try_start();

        gs.disconnect(id1);
        gs.disconnect(id2);
        assert_eq!(gs.turn, 1); // no resolution — no connected players
    }

    // ── broadcast ─────────────────────────────────────────────────────────────

    #[test]
    fn broadcast_reaches_all_connected_players() {
        let mut gs = make(2, 5);
        let (_, mut rx1) = join(&mut gs, "P1");
        let (_, mut rx2) = join(&mut gs, "P2");

        gs.broadcast(&ServerMessage::Pong);

        assert!(matches!(rx1.try_recv(), Ok(ServerMessage::Pong)));
        assert!(matches!(rx2.try_recv(), Ok(ServerMessage::Pong)));
    }

    #[test]
    fn broadcast_skips_disconnected_players() {
        let mut gs = make(2, 5);
        let (id1, mut rx1) = join(&mut gs, "P1");
        let (_, mut rx2) = join(&mut gs, "P2");

        gs.disconnect(id1);
        gs.broadcast(&ServerMessage::Pong);

        assert!(rx1.try_recv().is_err()); // disconnected — no message
        assert!(matches!(rx2.try_recv(), Ok(ServerMessage::Pong)));
    }
}
