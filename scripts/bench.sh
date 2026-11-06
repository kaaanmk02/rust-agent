#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release 2>&1 | tail -n 5
BIN=./target/release/rust-agent
CONF=/tmp/rust-agent-bench.json
cat > "$CONF" <<'JSON'
{"listen":"127.0.0.1:19999","auth_token":"bench","telegram":{"enabled":false},"discord":{"enabled":false},"slack":{"enabled":false},"whatsapp":{"enabled":false}}
JSON
"$BIN" serve --listen 127.0.0.1:19999 --config "$CONF" >/tmp/bench-server.log 2>&1 &
PID=$!
trap 'kill "$PID" >/dev/null 2>&1 || true' EXIT
sleep 2
BASE=http://127.0.0.1:19999
TOTAL=200
CONC=20
echo "running $TOTAL requests with concurrency $CONC ..."
ab -n $TOTAL -c $CONC -k -p /dev/null -T application/json "$BASE/api/chat" 2>&1 | tail -n 20 || true
echo "--- /proc/$PID smaps rss (KB) ---"
awk '/^Rss:/{s+=$2} END{print s}' /proc/$PID/smaps || true
echo "--- /proc/$PID status VmRSS (kB) ---"
grep VmRSS /proc/$PID/status || true
wait "$PID" || true
