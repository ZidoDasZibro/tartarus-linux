#!/bin/bash
# Run from inside the extracted update next to your clone:
#   bash APPLY_UPDATE.sh /home/zibro/tartarus-linux
set -e
DEST="${1:-$HOME/tartarus-linux}"
SRC="$(cd "$(dirname "$0")" && pwd)"
echo "Updating $DEST from $SRC"
mkdir -p "$DEST/src"
cp -v "$SRC"/src/*.rs "$DEST/src/"
cp -v "$SRC/Cargo.toml" "$DEST/Cargo.toml"
cp -v "$SRC/config.example.toml" "$DEST/config.example.toml"
cd "$DEST"
cargo build --release
echo ""
echo "OK. Kill old process and run:"
echo "  pkill -f tartarus-linux || true"
echo "  $DEST/target/release/tartarus-linux"
echo "Expect: tartarus-linux v0.2.0 — 1 layer, key|axis binds"
