# Repository Guidelines

## Project Structure & Module Organization
This is a Rust project with two binaries and a shared library:

- `src/bin/server.rs`: authoritative multiplayer server (Tokio-based).
- `src/bin/client.rs`: client runtime and Bevy app entrypoint.
- `src/game/`: game domain logic (`state`, `orders`, `hex`, ECS systems, GUI rendering).
- `src/net/`: wire protocol and typed session/networking utilities.
- `src/lib.rs`: shared crate surface used by binaries.

Tests are mostly inline (`#[cfg(test)]`) inside source modules (for example `src/net/protocol.rs` and `src/game/systems/resolution.rs`).

## Build, Test, and Development Commands
- `cargo build --bin server --features resolve`: build the server with turn-resolution logic.
- `cargo build --bin client --features render`: build the graphical client path.
- `cargo run --bin server --features resolve`: start server on default port `7878`.
- `cargo run --bin client --features render`: run client with Bevy rendering.
- `cargo run --bin client -- 127.0.0.1:7878`: connect client to a specific host.
- `cargo test`: run all unit tests across library and binaries.
- `cargo fmt` and `cargo clippy --all-targets --all-features`: formatting and lint checks before PR.

## Coding Style & Naming Conventions
- Follow standard Rust style (rustfmt defaults, 4-space indentation).
- Use `snake_case` for functions/modules/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep networking protocol types explicit and strongly typed (`ClientMessage`, `ServerMessage`), and isolate transport concerns inside `src/net/`.

## Testing Guidelines
- Place unit tests close to the code they validate.
- Prefer deterministic tests for resolution and protocol encoding/decoding.
- Test names should describe behavior, e.g. `uncontested_move_succeeds`.
- For multiplayer changes, include a short manual validation note (server + 2 clients) in the PR.

## Commit & Pull Request Guidelines
- Current history uses short, imperative commit messages (for example: `refactor`, `restructure package`); keep subject lines concise and action-oriented.
- Prefer focused commits by concern (protocol, resolution, rendering).
- PRs should include:
  - what changed and why,
  - commands run (`cargo test`, manual run steps),
  - linked issue (if any),
  - screenshots or terminal logs when UI/network behavior changes.
