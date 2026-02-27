#!/usr/bin/env bash
set -euo pipefail

PORT="${PORT:-7878}"
HOST="${HOST:-127.0.0.1}"
MAX_TURNS="${MAX_TURNS:-2}"
P1_NAME="${P1_NAME:-P1}"
P2_NAME="${P2_NAME:-P2}"
LOG_DIR="${LOG_DIR:-/tmp/seb-mul-game-smoke-$(date +%s)}"

mkdir -p "$LOG_DIR"
SERVER_LOG="$LOG_DIR/server.log"
P1_LOG="$LOG_DIR/client1.log"
P2_LOG="$LOG_DIR/client2.log"

SERVER_PID=""
P1_PID=""
P2_PID=""

cleanup() {
  if [[ -n "$P1_PID" ]]; then kill "$P1_PID" >/dev/null 2>&1 || true; fi
  if [[ -n "$P2_PID" ]]; then kill "$P2_PID" >/dev/null 2>&1 || true; fi
  if [[ -n "$SERVER_PID" ]]; then kill "$SERVER_PID" >/dev/null 2>&1 || true; fi
}
trap cleanup EXIT

echo "[smoke] Logs: $LOG_DIR"
echo "[smoke] Starting server on ${HOST}:${PORT}"
cargo run --bin server --features resolve -- --port "$PORT" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# Give the server a short time to compile/start listening.
sleep 3

echo "[smoke] Starting headless clients (${P1_NAME}, ${P2_NAME}) for ${MAX_TURNS} resolved turns"
cargo run --bin client -- --headless --name "$P1_NAME" --max-turns "$MAX_TURNS" "${HOST}:${PORT}" >"$P1_LOG" 2>&1 &
P1_PID=$!
cargo run --bin client -- --headless --name "$P2_NAME" --max-turns "$MAX_TURNS" "${HOST}:${PORT}" >"$P2_LOG" 2>&1 &
P2_PID=$!

wait "$P1_PID"
P1_CODE=$?
wait "$P2_PID"
P2_CODE=$?

echo "[smoke] client exit codes: P1=$P1_CODE P2=$P2_CODE"
echo "[smoke] --- server log tail ---"
tail -n 40 "$SERVER_LOG" || true
echo "[smoke] --- client1 log ---"
tail -n 40 "$P1_LOG" || true
echo "[smoke] --- client2 log ---"
tail -n 40 "$P2_LOG" || true

if [[ "$P1_CODE" -ne 0 || "$P2_CODE" -ne 0 ]]; then
  echo "[smoke] FAILED"
  exit 1
fi

echo "[smoke] OK"
