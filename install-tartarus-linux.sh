#!/usr/bin/env bash
# One-shot installer for tartarus-linux (Razer Tartarus Pro analog driver).
# Usage:  bash install-tartarus-linux.sh
# Or:    curl -fsSL <URL> | bash
set -euo pipefail

REPO="https://github.com/ZidoDasZibro/tartarus-linux.git"
DIR="${HOME}/tartarus-linux"

echo "==> tartarus-linux one-shot install"
echo

# --- deps ---
need=()
command -v git   >/dev/null 2>&1 || need+=(git)
command -v cargo >/dev/null 2>&1 || need+=(rust cargo)
command -v pkg-config >/dev/null 2>&1 || need+=(pkgconf)
# udev headers (Fedora name)
if ! pkg-config --exists libudev 2>/dev/null && ! [ -f /usr/include/libudev.h ]; then
  need+=(systemd-devel)
fi

if ((${#need[@]})); then
  echo "Missing packages: ${need[*]}"
  if command -v dnf >/dev/null 2>&1; then
    echo "Installing with dnf (sudo)…"
    sudo dnf install -y git rust cargo pkgconf systemd-devel
  elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S --needed --noconfirm git rust pkgconf systemd-libs
  elif command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y git cargo pkg-config libudev-dev
  else
    echo "Install these manually, then re-run: ${need[*]}"
    exit 1
  fi
fi

# --- fetch ---
if [[ -d "$DIR/.git" ]]; then
  echo "==> updating $DIR"
  git -C "$DIR" pull --ff-only || true
else
  echo "==> cloning into $DIR"
  git clone --depth 1 "$REPO" "$DIR"
fi

# --- install ---
cd "$DIR"
bash INSTALL.sh

echo
echo "All done."
echo "  Driver:   tartarus-linux-start   (menu: Tartarus Linux Driver)"
echo "  Web UI:   tartarus-linux-ui      (menu: Tartarus Linux Config)"
echo "            http://127.0.0.1:8787/"
echo "  Stop:     tartarus-linux-stop"
echo
echo "If you were just added to group 'input', log out and back in once."
