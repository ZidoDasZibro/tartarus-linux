#!/bin/bash
set -euo pipefail
PREFIX="${PREFIX:-$HOME/.local}"
systemctl --user disable --now tartarus-linux.service 2>/dev/null || true
pkill -x tartarus-linux 2>/dev/null || true
rm -f "$PREFIX/bin/tartarus-linux" \
      "$PREFIX/bin/tartarus-linux-ui" \
      "$PREFIX/bin/tartarus-linux-start" \
      "$PREFIX/bin/tartarus-linux-stop" \
      "${XDG_DATA_HOME:-$HOME/.local/share}/applications/tartarus-linux.desktop" \
      "${XDG_DATA_HOME:-$HOME/.local/share}/applications/tartarus-linux-config.desktop" \
      "${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/tartarus-linux.service"
systemctl --user daemon-reload 2>/dev/null || true
echo "Removed user install. Config left in ~/.config/tartarus-linux/"
echo "Optional: sudo rm /etc/udev/rules.d/99-tartarus-linux.rules"
