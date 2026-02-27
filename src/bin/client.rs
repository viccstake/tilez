use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use anyhow::bail;
use bevy::prelude::*;
use clap::Parser;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use naval_game::game::components::{Health, Position, Ship, ShipId, TargetPosition};
use naval_game::game::gui::animation::animate_transitions;
use naval_game::game::gui::hud::{
    setup_hud, update_roster_text, update_state_text, update_turn_text,
};
use naval_game::game::gui::rendering::spawn_ship_visuals;
use naval_game::game::hex::Hex;
use naval_game::game::resources::{CurrentTurn, HexLayout, LocalPlayerId, OrderQueue, Selection};
use naval_game::game::state::GameState;
use naval_game::game::systems::input;
use naval_game::game::systems::input::clear_selection_on_animate;
use naval_game::game::systems::setup::setup_board;
use naval_game::net::protocol::{ClientMessage, GameSnapshot, ServerMessage};
use naval_game::net::Session;

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

    /// Run a network-only client (no Bevy window), auto-submitting empty orders.
    #[arg(long, default_value_t = false)]
    headless: bool,

    /// Number of resolved turns before exiting in headless mode.
    #[arg(long, default_value_t = 2)]
    max_turns: u32,
}

// ── Networking resources ──────────────────────────────────────────────────────

/// Inbound channel: server → Bevy. Wrapped in Mutex to satisfy Sync.
#[derive(Resource)]
struct NetRx(Mutex<mpsc::UnboundedReceiver<ServerMessage>>);

/// Outbound channel: Bevy → server.
#[derive(Resource)]
struct NetTx(mpsc::UnboundedSender<ClientMessage>);

/// Maps server ship ID → Bevy entity so snapshots update entities in place.
#[derive(Resource, Default)]
struct ShipEntities(HashMap<u32, Entity>);

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();
    if args.headless {
        if let Err(e) = run_headless(args) {
            eprintln!("[headless] Error: {e}");
            std::process::exit(1);
        }
        return;
    }
    run_gui(args);
}

fn run_gui(args: Args) {

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
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "sctk_adwaita=off".into(),
                    ..default()
                })
                .disable::<bevy::audio::AudioPlugin>()
        )
        .init_state::<GameState>()
        .insert_resource(OrderQueue::default())
        .insert_resource(CurrentTurn::default())
        .insert_resource(LocalPlayerId::default())
        .insert_resource(Selection::default())
        .insert_resource(ShipEntities::default())
        .insert_resource(HexLayout::default())
        .insert_resource(NetRx(Mutex::new(bevy_rx)))
        .insert_resource(NetTx(bevy_tx))
        .add_systems(Startup, (setup_camera, setup_board, setup_hud))
        .add_systems(Update, poll_network)
        .add_systems(Update, spawn_ship_visuals)
        .add_systems(Update, (update_turn_text, update_state_text, update_roster_text))
        .add_systems(OnEnter(GameState::Animating), clear_selection_on_animate)
        .add_systems(
            Update,
            animate_transitions.run_if(in_state(GameState::Animating)),
        )
        .add_systems(
            Update,
            (input::handle_input, submit_orders).run_if(in_state(GameState::Planning)),
        )
        .run();
}

fn run_headless(args: Args) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(headless_task(args.server, args.name, args.max_turns))
}

async fn headless_task(addr: String, player_name: String, max_turns: u32) -> anyhow::Result<()> {
    let stream = TcpStream::connect(&addr).await?;
    let mut session = Session::<ServerMessage, ClientMessage>::new(stream);
    session.send(&ClientMessage::Join { player_name: player_name.clone() }).await?;
    eprintln!("[headless:{player_name}] Connected to {addr}");

    let mut resolved_turns = 0_u32;
    while let Some(msg) = session.recv().await? {
        match msg {
            ServerMessage::Welcome { player_id } => {
                eprintln!("[headless:{player_name}] Welcome as player {player_id}");
            }
            ServerMessage::TurnStarted { turn } => {
                eprintln!("[headless:{player_name}] Turn {turn} started, submitting 0 orders");
                session
                    .send(&ClientMessage::SubmitOrders {
                        turn,
                        orders: vec![],
                    })
                    .await?;
            }
            ServerMessage::TurnResolved { turn } => {
                resolved_turns += 1;
                eprintln!("[headless:{player_name}] Turn {turn} resolved");
                if resolved_turns >= max_turns {
                    eprintln!(
                        "[headless:{player_name}] Reached max_turns={max_turns}, disconnecting"
                    );
                    return Ok(());
                }
            }
            ServerMessage::StateSnapshot(_) => {}
            ServerMessage::Error(e) => bail!("server error: {e}"),
            ServerMessage::Pong => {}
        }
    }

    Ok(())
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
                info!("Turn {turn} started");
                current_turn.0 = turn;
                // Do NOT set state here — animate_transitions drives the
                // Planning transition once all ship lerps complete.
                // Setting Planning here would cancel the Animating transition
                // set by TurnResolved (all three messages arrive in one frame).
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
/// For existing ships, inserts `TargetPosition` so `animate_transitions` can lerp
/// the visual toward the new hex.  New ships appear at their position immediately.
fn reconcile_ships(
    commands: &mut Commands,
    ship_entities: &mut ShipEntities,
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
            // Set the animation target; animate_transitions will lerp and then
            // update Position once complete.
            commands.entity(entity).insert(TargetPosition { hex: Hex::new(ship.q, ship.r) });
            if let Ok(mut hp) = healths.get_mut(entity) {
                hp.0 = ship.health;
            }
        } else {
            // Brand-new ship: appears directly at its position, no lerp.
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
