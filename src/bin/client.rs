use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use bevy::prelude::*;
use clap::Parser;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use naval_game::game::components::{Health, Position, Ship, ShipId};
use naval_game::game::hex::Hex;
use naval_game::game::resources::{CurrentTurn, OrderQueue};
use naval_game::game::state::GameState;
use naval_game::game::systems::input;
use naval_game::protocol::{ClientMessage, GameSnapshot, ServerMessage};
use naval_game::session::Session;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "client")]
struct Args {
    /// Server address
    #[arg(default_value = "127.0.0.1:7878")]
    server: String,

    /// Player name
    #[arg(short, long, default_value = "Player")]
    name: String,
}

// ── Networking resources ──────────────────────────────────────────────────────

/// Inbound channel: server → Bevy. Wrapped in Mutex to satisfy Sync.
#[derive(Resource)]
struct NetRx(Mutex<mpsc::UnboundedReceiver<ServerMessage>>);

/// Outbound channel: Bevy → server.
#[derive(Resource)]
struct NetTx(mpsc::UnboundedSender<ClientMessage>);

#[derive(Resource, Default)]
struct LocalPlayerId(Option<u32>);

/// Maps server ship ID → Bevy entity so snapshots update entities in place.
#[derive(Resource, Default)]
struct ShipEntities(HashMap<u32, Entity>);

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    let (server_tx, bevy_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let (bevy_tx, client_rx) = mpsc::unbounded_channel::<ClientMessage>();

    // Networking runs in its own OS thread with its own tokio runtime so that
    // Bevy can own the main thread unmodified.
    let addr = args.server.clone();
    let name = args.name.clone();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(net_task(addr, name, server_tx, client_rx));
    });

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(OrderQueue::default())
        .insert_resource(CurrentTurn::default())
        .insert_resource(LocalPlayerId::default())
        .insert_resource(ShipEntities::default())
        .insert_resource(NetRx(Mutex::new(bevy_rx)))
        .insert_resource(NetTx(bevy_tx))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, poll_network)
        .add_systems(
            Update,
            (input::handle_input, submit_orders).run_if(in_state(GameState::Planning)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Async networking task (separate thread) ───────────────────────────────────

async fn net_task(
    addr: String,
    player_name: String,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
    mut client_rx: mpsc::UnboundedReceiver<ClientMessage>,
) {
    let stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[net] Failed to connect to {addr}: {e}");
            return;
        }
    };
    eprintln!("[net] Connected to {addr}");

    let mut session = Session::<ServerMessage, ClientMessage>::new(stream);

    if let Err(e) = session.send(&ClientMessage::Join { player_name }).await {
        eprintln!("[net] Failed to send Join: {e}");
        return;
    }

    loop {
        tokio::select! {
            result = session.recv() => match result {
                Ok(Some(msg)) => {
                    if server_tx.send(msg).is_err() {
                        break; // Bevy shut down
                    }
                }
                Ok(None) => { eprintln!("[net] Server closed connection"); break; }
                Err(e)   => { eprintln!("[net] Recv error: {e}"); break; }
            },
            msg = client_rx.recv() => match msg {
                Some(m) => {
                    if let Err(e) = session.send(&m).await {
                        eprintln!("[net] Send error: {e}"); break;
                    }
                }
                None => break, // Bevy shut down
            },
        }
    }

    eprintln!("[net] Disconnected");
}

// ── Bevy systems ──────────────────────────────────────────────────────────────

/// Drain the inbound channel every frame. Handles all server messages:
/// updates local player state, reconciles ship entities, drives GameState.
fn poll_network(
    mut commands: Commands,
    net_rx: Res<NetRx>,
    mut local_id: ResMut<LocalPlayerId>,
    mut ship_entities: ResMut<ShipEntities>,
    mut positions: Query<&mut Position>,
    mut healths: Query<&mut Health>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_turn: ResMut<CurrentTurn>,
) {
    let mut rx = net_rx.0.lock().unwrap();
    while let Ok(msg) = rx.try_recv() {
        match msg {
            ServerMessage::Welcome { player_id } => {
                info!("Joined as player {player_id}");
                local_id.0 = Some(player_id);
            }
            ServerMessage::TurnStarted { turn } => {
                info!("Turn {turn} started — Planning");
                current_turn.0 = turn;
                next_state.set(GameState::Planning);
            }
            ServerMessage::TurnResolved { turn } => {
                info!("Turn {turn} resolved — Animating");
                next_state.set(GameState::Animating);
            }
            ServerMessage::StateSnapshot(bytes) => {
                match bincode::deserialize::<GameSnapshot>(&bytes) {
                    Ok(snap) => reconcile_ships(
                        &mut commands,
                        &mut ship_entities,
                        &mut positions,
                        &mut healths,
                        snap,
                    ),
                    Err(e) => warn!("Bad snapshot: {e}"),
                }
            }
            ServerMessage::Error(e) => warn!("Server error: {e}"),
            ServerMessage::Pong => {}
        }
    }
}

/// Update existing ship entities from a snapshot, spawn new ones, despawn removed.
/// Keeps the same Entity alive across turns — Phase 4 animation will lerp between states.
fn reconcile_ships(
    commands: &mut Commands,
    ship_entities: &mut ShipEntities,
    positions: &mut Query<&mut Position>,
    healths: &mut Query<&mut Health>,
    snap: GameSnapshot,
) {
    let snap_ids: HashSet<u32> = snap.ships.iter().map(|s| s.id).collect();

    // Despawn ships no longer in the snapshot
    ship_entities.0.retain(|id, entity| {
        if !snap_ids.contains(id) {
            commands.entity(*entity).despawn();
            false
        } else {
            true
        }
    });

    // Update existing ships or spawn new ones
    for ship in &snap.ships {
        if let Some(&entity) = ship_entities.0.get(&ship.id) {
            if let Ok(mut pos) = positions.get_mut(entity) {
                pos.hex = Hex::new(ship.q, ship.r);
            }
            if let Ok(mut hp) = healths.get_mut(entity) {
                hp.0 = ship.health;
            }
        } else {
            let entity = commands
                .spawn((
                    ShipId(ship.id),
                    Ship { owner_id: ship.owner_id },
                    Position { hex: Hex::new(ship.q, ship.r) },
                    Health(ship.health),
                ))
                .id();
            ship_entities.0.insert(ship.id, entity);
        }
    }
}

/// Send the local order queue to the server when the player presses Enter or Space.
fn submit_orders(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut orders: ResMut<OrderQueue>,
    net_tx: Res<NetTx>,
    current_turn: Res<CurrentTurn>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        let queued = std::mem::take(&mut orders.orders);
        info!(
            "Submitting {} order(s) for turn {}",
            queued.len(),
            current_turn.0
        );
        net_tx
            .0
            .send(ClientMessage::SubmitOrders {
                turn: current_turn.0,
                orders: queued,
            })
            .ok();
    }
}
