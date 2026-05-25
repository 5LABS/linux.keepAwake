#!/usr/bin/env bash
set -euo pipefail

# Build and install keep-awake into the user's local prefix, register an app
# launcher and (by default) an autostart entry so it starts on login.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
AUTOSTART_DIR="$HOME/.config/autostart"
BIN_PATH="$BIN_DIR/keep-awake"

echo ">> Building release binary..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

echo ">> Installing binary to $BIN_PATH"
mkdir -p "$BIN_DIR"
install -m 0755 "$SCRIPT_DIR/target/release/keep-awake" "$BIN_PATH"

desktop_entry() {
  cat <<EOF
[Desktop Entry]
Type=Application
Name=Keep Awake
Comment=Keep the system and screen awake
Exec=$BIN_PATH
Icon=keep-awake
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
EOF
}

echo ">> Installing app launcher to $APP_DIR/keep-awake.desktop"
mkdir -p "$APP_DIR"
desktop_entry > "$APP_DIR/keep-awake.desktop"

# Autostart on login (can later be toggled off from the tray menu).
echo ">> Enabling autostart at $AUTOSTART_DIR/keep-awake.desktop"
mkdir -p "$AUTOSTART_DIR"
desktop_entry > "$AUTOSTART_DIR/keep-awake.desktop"

echo
echo "Done. Start now with:  $BIN_PATH &"
echo "It will also start automatically on your next login."
