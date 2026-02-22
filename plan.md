Step 0 — Triage (do first)

  - Remove or fix resolution.rs Bevy system (fix the entity_id matching bug; it becomes the local-mode resolver)
  - Delete TurnTimer, logger.rs, config.rs, GamePlugin, turn.rs/collect_orders, setup.rs ship spawn — they are stubs that add noise without purpose. Keep only TargetPosition (needed for animation).
  - setup.rs should spawn only the camera.

  ---
  Step 1 — Tests

  The pure, Bevy-free logic needs tests now, before rendering complexity makes it harder.

  src/game/hex.rs — unit tests:
  - neighbors() returns 6 unique coords
  - Known axial neighbors are correct
  - Hex::distance(a, b) (add this function — needed for shooting ranges later)

  src/bin/server.rs → extract resolve_turn to src/game/resolution.rs (no Bevy, no feature flag):
  - Move unblocked
  - Hold order
  - Collision (two ships → same hex → both stay)
  - Partial collision (one blocked, others move)
  - Orders for non-existent ship IDs are ignored
  - Empty orders map

  src/protocol.rs — roundtrip tests:
  - ClientMessage serialize → deserialize
  - ServerMessage::StateSnapshot with embedded GameSnapshot

  GameServer state machine (in server.rs, using #[cfg(test)]):
  - Lobby full rejection
  - Game-in-progress join rejection
  - try_start only fires at min_players
  - submit_orders ignored in wrong turn/phase
  - Disconnect during planning triggers resolution if all remaining submitted
  - Disconnect during lobby does not trigger resolution

  ---
  Step 2 — Phase 4a: Hex grid + ship rendering

  - Add Hex::to_world(size) -> Vec2 conversion (flat-top axial to pixel)
  - setup_board system: spawn hex tile entities as Sprite (flat-top, tinted, grid of ~7-radius)
  - spawn_ship_visuals system: on ship entity creation, add a Sprite component (colored circle/rect by owner_id)
  - Resource HexLayout { size: f32 } for the tile size

  ---
  Step 3 — Phase 4b: Animation

  - On StateSnapshot receive, set TargetPosition on each ship (instead of directly updating Position)
  - animate_transitions system: lerp Transform from current world pos toward TargetPosition world pos over a fixed duration (e.g. 0.4s) using a AnimTimer component
  - When lerp completes: update Position to match, remove AnimTimer, transition back to Planning

  ---
  Step 4 — Phase 4c: Click input

  - pixel_to_hex(world_pos: Vec2, size: f32) -> Hex utility (inverse of to_world)
  - Two-step input: first click selects a ship (if owner_id == local_player_id), second click on a valid hex queues a Move order
  - Visual feedback: selected ship gets a highlight component/color tint
  - Replace the keyboard-only handle_input system

  ---
  Step 5 — Phase 4d: HUD

  - Turn counter text
  - State banner ("Planning — press Space to submit" / "Waiting for opponents…" / "Animating")
  - Player list with connected status (derive from snapshot owner_ids)

  ---
  Step 6 — Phase 5: Game completeness

  - Hex::distance already added in Step 1 — use it for shooting range validation
  - Order::Shoot { from_ship_id, target_hex } — add to protocol and orders
  - Server: during resolve_turn, after moves, process shoot orders (ships at distance ≤ 2 deal damage; health ≤ 0 → remove ship)
  - Server: after resolution, check win condition — last player with ships alive wins; emit ServerMessage::GameOver { winner_id }
  - Client: receive GameOver, show result screen, disable input

  ---
  Step 7 — End-to-end playtest

  - Run server -n 2, two client windows, play a full game to GameOver
  - Integration test: #[tokio::test] spins up a real server + two clients in-process, plays out 3 turns, asserts correct ship positions and turn numbers