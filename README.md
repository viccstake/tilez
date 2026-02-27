
## Doc

### Run commands:
```sh
  cargo run --bin server --features resolve   # Terminal 1 — start the server

  cargo run --bin client --features render    # Terminal 2 — Player 1 connects

  cargo run --bin client                      # Terminal 3 — Player 2 connects
  ...

  cargo run --bin client 192.168.x.x:7878     # To connect from another machine:
```

### Build commands:
```sh 
  cargo build --bin server --features resolve 

  cargo build --bin client --features render
  
  # run with full trace logging
  ./target/debug/server -vvv
```

# 🛠 Project Summary — Multiplayer Naval Strategy Game (Rust + Bevy + Tokio)

## 🎯 Goal

A **robust multiplayer naval strategy game** written in Rust using:

* **Bevy** for client-side rendering and ECS game logic
* **Tokio** for networking
* **Authoritative server model**
* Support for **32 concurrent players**
* 1 server instance = 1 game session

The system should be:

* Deterministic - procedural seed generation
* Fault tolerant - by leveraging funtional programming and idiomatic Rust
  - First priority is on implementing external traits to handle as much application fault tolerance as possible (e.g. impl Drop)
* Cleanly architected
  - Two binaries: Client and Server
  - Use Rust's package management with path flattening to stucture the code in different files by
    1. Communication (`net`)
    2. State / API (`game`)
    3. Render / UI (`gui`)
  - Use internal traits to support dynamic dispatch for central code
  - Leverage determenistic Bevy to handle all game logic and resolution of actions
* Scalable in design
  - Adding MORE of something (e.g. new ships) should be trivial
  - Logic inside the game grouped (using traits) in such a way that providing a new feature to some existing set of features is easy
  - Cargo features dictate how the binary is built
    - Resolve: Binary produces its own next state (i.e. `resolve_turn` is included and used)
    - Render: The bevy GUI is rendered (appearance depends on binary implementation). If this feature is not set, I/O is handled through the network interface.

---

# 🧠 Game Design

### Genre

Turn-based naval strategy game on a **hexagonal grid**. 

Each player locks in their moves and all moves are executed "simaltenously" (peaudo-description = iterate orders left: order.do(1 unit of work))

### Core Mechanics

* Each player controls ships.
* Game runs in discrete phases:

  1. **Planning** – Players submit orders.
  2. **Resolving** – Server resolves all orders simultaneously.
  3. **Animating** – Clients animate the resolved state.
* Simultaneous resolution may cause:

  * Ship collisions
  * Conflicting moves
  * Combat interactions

Server is authoritative for:

* Order validation 
* Turn resolution (producing the next state)
* Game state snapshots (all clients get this snapshot)

Client runs the same game but:

* The game is just reflecting the state from the server, and provides an interactive GUI (and Terminal) API for the user.
* The GUI should provide a HUD, Sidebars, Menus, Camera View, Animated Map.
* The player should be prompted with valid orders in the GUI inferred from the state for each owned ship.
* After an order is placed, it gets forwarded to the server, possibly overwriting an old order. The server responds if the move has been registered.
* The server sends back a state snapshot after each turn, the animation is inferred from the transition from old to new state.

### Root Structs (inexchangeable in the codebase)
```rust
struct Player {
  identifier: PlayerId,
  level: impl ProgressionTracking,
  game: GameState
}
```
```rust
struct GameState {
  board: HexGrid<u64>
}
```
---

# 🏗 Architecture Overview

## Folder Structure

```
.
└── src
    ├── lib.rs
    ├── main.rs
    ├── bin
    │   ├── client.rs
    │   └── server.rs
    ├── game
    │   ├── mod.rs
    │   ├── components.rs
    │   ├── hex.rs
    │   ├── orders.rs
    │   ├── resources.rs
    │   ├── state.rs
    │   ├── gui
    │   │   ├── hud.rs
    │   │   ├── animation.rs
    │   │   └── rendering.rs
    │   └── systems
    │       ├── mod.rs
    │       ├── input.rs
    │       ├── resolution.rs
    │       └── setup.rs
    └── net
        ├── mod.rs
        ├── protocol.rs
        └── session.rs
```

---

# 🌐 Networking Layer

## Transport

* TCP via Tokio
* Length-prefixed framing using `LengthDelimitedCodec`
* Serialization via `serde + bincode`
* Fully typed message protocol

## Session Layer

Generic typed session:

```rust
Session<IncomingMessage, OutgoingMessage>
```

Responsibilities:

* Framing
* Serialization/deserialization
* Connection lifecycle
* No game logic inside

---

## Network Protocol

### Client → Server

```rust
enum ClientMessage {
    Join { player_name: String },
    SubmitOrders { turn: u32, orders: Vec<Order> },
    Ping,
}
```

### Server → Client

```rust
enum ServerMessage {
    Welcome { player_id: u32 },
    TurnStarted { turn: u32 },
    TurnResolved { turn: u32 },
    StateSnapshot(Vec<u8>),
    Error(String),
    Pong,
}
```

---

# 🖥 Server Architecture

* Tokio async runtime
* Accepts TCP connections
* Each connection handled via `Session`
* Stores authoritative game state
* Collects orders for current turn
* Resolves turn simultaneously
* Broadcasts:

  * Turn resolution
  * Game state snapshot

Key properties:

* Server validates all input
* Clients are not trusted
* Handles disconnect gracefully
* Can convert disconnected players to AI (planned)

---

# 🎮 Client Architecture

* Bevy runs rendering + ECS
* Networking runs in async background task
* Communication between networking and Bevy via channels
* Client:

  * Sends orders
  * Receives state snapshots
  * Animates transitions

Client is not authoritative.

---

# 🔁 Game State Flow

```mermaid
flowchart LR
  P[Planning] --> R[Resolving] --> A[Animating] --> P
```

* Planning: collect player orders
* Resolving: authoritative simultaneous resolution
* Animating: smooth interpolation on clients

---

# 🔒 Robustness Requirements

* Handles up to ~X players reliably
* Graceful disconnect handling
* Heartbeat (Ping/Pong) system
* Deterministic turn resolution
* No TCP message corruption (length-prefixed framing)

---

# 🧱 Core Principles

1. Strict separation:

   * Networking
   * Game logic
   * Rendering

2. Deterministic server logic

3. Typed message protocol

4. Scalable production-grade design

---

# 🚀 Long-Term Extension Possibilities

* AI players
* Replays
* Fog of war
* Ship classes
* Combat resolution system
* Fleet grouping
* Save/load
* Lockstep prediction

---
