set -e
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/cargo bulunamadı: https://rustup.rs"
  exit 1
fi
cargo build --release
echo "binary: target/release/rust-agent"
echo "run:   ./target/release/rust-agent serve --listen 0.0.0.0:9090"
