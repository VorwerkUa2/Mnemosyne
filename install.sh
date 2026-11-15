#!/usr/bin/env sh
set -eu

# If running inside a git checkout with packaging/linux/install.sh present:
SCRIPT_DIR=""
if [ -n "${BASH_SOURCE[0]:-}" ]; then
    SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)
fi

if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/packaging/linux/install.sh" ]; then
    exec "$SCRIPT_DIR/packaging/linux/install.sh" "$@"
fi

# Otherwise, running via: curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
echo "========================================="
echo "  Mnemosyne & Decod One-Line Installer   "
echo "========================================="
echo ""

TEMP_DIR=$(mktemp -d /tmp/mnemosyne-setup-XXXXXX)
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT INT TERM

echo "Downloading latest Mnemosyne package..."
if command -v git >/dev/null 2>&1; then
    git clone --depth 1 https://github.com/VorwerkUa2/Mnemosyne.git "$TEMP_DIR/mnemosyne"
elif command -v curl >/dev/null 2>&1; then
    curl -fsSL https://github.com/VorwerkUa2/Mnemosyne/archive/refs/heads/main.tar.gz | tar -xz -C "$TEMP_DIR"
    mv "$TEMP_DIR"/Mnemosyne-main "$TEMP_DIR/mnemosyne"
elif command -v wget >/dev/null 2>&1; then
    wget -qO- https://github.com/VorwerkUa2/Mnemosyne/archive/refs/heads/main.tar.gz | tar -xz -C "$TEMP_DIR"
    mv "$TEMP_DIR"/Mnemosyne-main "$TEMP_DIR/mnemosyne"
else
    echo "Error: git, curl, or wget is required to install Mnemosyne." >&2
    exit 1
fi

chmod +x "$TEMP_DIR/mnemosyne/packaging/linux/install.sh"
exec "$TEMP_DIR/mnemosyne/packaging/linux/install.sh" "$@"
