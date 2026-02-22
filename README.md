
## Doc

| File              | Responsibility |
| ----------------- | -------------- |
| src/game.rs       | Standalone file for running, interacting and rendering the game    |
| src/session.rs    | Send-and-receive API for 1 TCP session                            |
| src/logger.rs     | Logger struct for stdout, stderr                                  |
| src/bin/server.rs | Entry point — host the game                                       |
| src/bin/client.rs | Entry point — connect to a game server                            |
  
### Run commands:
```sh
  cargo run --bin server                    # Terminal 1 — start the server

  cargo run --bin client                    # Terminal 2 — Player 1 connects

  cargo run --bin client                    # Terminal 3 — Player 2 connects

  cargo run --bin client 192.168.x.x:7878   # To connect from another machine:
```
### Build commands:
```sh 
  cargo build --bin server          # server only (no Bevy needed)

  cargo build --features game       # full crate including Bevy ECS module

  ./target/debug/server -vvv        # run with full trace logging
```



# 🛠 Project Summary — Multiplayer Naval Strategy Game (Rust + Bevy)

## 🎯 Goal

A **robust hobby-grade multiplayer naval strategy game** written in Rust using:

* **Bevy** for client-side rendering and ECS game logic
* **Tokio** for networking
* **Authoritative server model**
* Support for **~5 concurrent players**
* 1 server instance = 1 game session

The system should be:

* Deterministic
* Fault tolerant
* Cleanly architected
* Scalable in design (even if small in scope)

---

# 🧠 Game Design

### Genre

Turn-based naval strategy game on a **hexagonal grid**.

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
* Turn resolution
* Game state snapshots

---

# 🏗 Architecture Overview

## Folder Structure

```
src/
├── bin/
│   ├── client.rs
│   └── server.rs
├── game/
│   ├── components.rs
│   ├── config.rs
│   ├── hex.rs
│   ├── orders.rs
│   ├── resources.rs
│   ├── state.rs
│   └── systems/
│       ├── animation.rs
│       ├── input.rs
│       ├── resolution.rs
│       ├── setup.rs
│       └── turn.rs
├── game.rs
├── lib.rs
├── logger.rs
├── main.rs
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

```
Planning → Resolving → Animating → Planning
```

* Planning: collect player orders
* Resolving: authoritative simultaneous resolution
* Animating: smooth interpolation on clients

---

# 🔒 Robustness Requirements

* Handles up to ~5 players reliably
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

4. Minimal but production-grade design

5. Designed for scalability (even if hobby scope)

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

**Context:**
I am building a hobby-grade but robust multiplayer naval strategy game in Rust using Bevy (client) and Tokio (server). It supports ~5 players per match. The server is authoritative and handles deterministic simultaneous turn resolution on a hex grid. Networking uses TCP with length-prefixed framing and serde+bincode. The architecture cleanly separates game logic, networking (session layer), and rendering. Clients animate transitions between discrete turn states.
