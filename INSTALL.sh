#!/bin/bash
# Install tartarus-linux for the current user + udev rules (sudo once).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
CFG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/tartarus-linux"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

echo "==> tartarus-linux install"
echo "    prefix: $PREFIX"

# deps hint
if ! command -v cargo >/dev/null 2>&1; then
  echo "Need rust/cargo. On Fedora/Nobara:  sudo dnf install rust cargo systemd-devel"
  exit 1
fi

echo "==> build release"
cd "$ROOT"
cargo build --release

echo "==> install binary + helpers"
mkdir -p "$BIN_DIR" "$APP_DIR" "$CFG_DIR" "$UNIT_DIR"
install -m 755 "$ROOT/target/release/tartarus-linux" "$BIN_DIR/tartarus-linux"
install -m 755 "$ROOT/scripts/tartarus-linux-ui" "$BIN_DIR/tartarus-linux-ui"
install -m 755 "$ROOT/scripts/tartarus-linux-start" "$BIN_DIR/tartarus-linux-start"
install -m 755 "$ROOT/scripts/tartarus-linux-stop" "$BIN_DIR/tartarus-linux-stop"

echo "==> desktop menu entries"
install -m 644 "$ROOT/packaging/tartarus-linux.desktop" "$APP_DIR/"
install -m 644 "$ROOT/packaging/tartarus-linux-config.desktop" "$APP_DIR/"
# ensure Exec is on PATH
sed -i "s|^Exec=tartarus-linux-start|Exec=$BIN_DIR/tartarus-linux-start|" "$APP_DIR/tartarus-linux.desktop"
sed -i "s|^Exec=tartarus-linux-ui|Exec=$BIN_DIR/tartarus-linux-ui|" "$APP_DIR/tartarus-linux-config.desktop"
update-desktop-database "$APP_DIR" 2>/dev/null || true

echo "==> default config (if missing)"
if [[ ! -f "$CFG_DIR/config.toml" ]]; then
  cp "$ROOT/config.example.toml" "$CFG_DIR/config.toml"
  echo "    wrote $CFG_DIR/config.toml"
fi

echo "==> systemd --user unit"
install -m 644 "$ROOT/tartarus-linux.service" "$UNIT_DIR/tartarus-linux.service"
systemctl --user daemon-reload
systemctl --user enable tartarus-linux.service
systemctl --user start tartarus-linux.service || true

echo "==> udev rules (needs sudo for /dev/uinput access)"
if [[ -f "$ROOT/99-tartarus-linux.rules" ]]; then
  sudo cp "$ROOT/99-tartarus-linux.rules" /etc/udev/rules.d/
  sudo udevadm control --reload
  sudo udevadm trigger || true
fi
# input group
if ! id -nG "$USER" | grep -qw input; then
  echo "    adding $USER to group 'input' (re-login required)"
  sudo usermod -aG input "$USER" || true
fi

echo ""
echo "Done."
echo "  Start:     tartarus-linux-start   (or menu: Tartarus Linux Driver)"
echo "  Config UI: tartarus-linux-ui     (or menu: Tartarus Linux Config)"
echo "  Stop:      tartarus-linux-stop"
echo "  URL:       http://127.0.0.1:8787/"
echo "  Autostart: systemctl --user enable --now tartarus-linux.service"
echo ""
echo "If group 'input' was just added, log out and back in once."
