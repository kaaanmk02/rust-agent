#!/bin/bash
set -euo pipefail
cd /root/rust-agent-clean
cargo run -- serve --listen 127.0.0.1:19999
