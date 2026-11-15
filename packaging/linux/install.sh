#!/usr/bin/env sh
set -eu

SOURCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(cd "$SOURCE_DIR/../.." && pwd)
DATA_HOME=${XDG_DATA_HOME:-"$HOME/.local/share"}
INSTALL_DIR=$DATA_HOME/mnemosyne/app
BIN_DIR=${HOME}/.local/bin
APPLICATIONS_DIR=$DATA_HOME/applications

echo "========================================="
echo "  Mnemosyne & Decod Linux Installer     "
echo "========================================="
echo ""

# 1. Detect Package Manager and install system dependencies if needed
detect_and_install_deps() {
    if [ -f "/etc/void-release" ] || command -v xbps-install >/dev/null 2>&1; then
        echo "Detected Void Linux (xbps)"
        if command -v sudo >/dev/null 2>&1; then
            echo "Installing runtime libraries via xbps-install..."
            sudo xbps-install -Sy libxkbcommon-devel fontconfig-devel wayland-devel libX11-devel libXcursor-devel libXi-devel libXrandr-devel nodejs python3 python3-pip || true
        fi
    elif [ -f "/etc/fedora-release" ] || command -v dnf >/dev/null 2>&1; then
        echo "Detected Fedora Linux (dnf)"
        if command -v sudo >/dev/null 2>&1; then
            echo "Installing runtime libraries via dnf..."
            sudo dnf install -y libxkbcommon-devel fontconfig-devel wayland-devel libX11-devel libXcursor-devel libXi-devel libXrandr-devel nodejs python3 python3-pip || true
        fi
    elif [ -f "/etc/arch-release" ] || command -v pacman >/dev/null 2>&1; then
        echo "Detected Arch Linux (pacman)"
        if command -v sudo >/dev/null 2>&1; then
            echo "Installing runtime libraries via pacman..."
            sudo pacman -S --needed --noconfirm libxkbcommon fontconfig wayland libx11 libxcursor libxi libxrandr nodejs npm python python-pip || true
        fi
    elif [ -f "/etc/debian_version" ] || command -v apt-get >/dev/null 2>&1; then
        echo "Detected Debian/Ubuntu (apt)"
        if command -v sudo >/dev/null 2>&1; then
            echo "Installing runtime libraries via apt..."
            sudo apt-get update -y || true
            sudo apt-get install -y libxkbcommon-dev libfontconfig1-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev nodejs npm python3 python3-pip || true
        fi
    fi
}

detect_and_install_deps

# 2. Locate or build mnemosyne binary
BINARY_PATH="$SOURCE_DIR/mnemosyne"
if [ ! -x "$BINARY_PATH" ]; then
    if [ -x "$REPO_ROOT/target/release/mnemosyne" ]; then
        BINARY_PATH="$REPO_ROOT/target/release/mnemosyne"
    elif command -v cargo >/dev/null 2>&1; then
        echo "Building Mnemosyne in release mode..."
        (cd "$REPO_ROOT" && cargo build --release)
        BINARY_PATH="$REPO_ROOT/target/release/mnemosyne"
    else
        echo 'mnemosyne binary is missing and cargo is not installed.' >&2
        exit 1
    fi
fi

# 3. Install binary and script
mkdir -p "$INSTALL_DIR" "$BIN_DIR" "$APPLICATIONS_DIR"
install -m 755 "$BINARY_PATH" "$INSTALL_DIR/mnemosyne"
if [ -f "$SOURCE_DIR/uninstall.sh" ]; then
    install -m 755 "$SOURCE_DIR/uninstall.sh" "$INSTALL_DIR/uninstall.sh"
    ln -sfn "$INSTALL_DIR/uninstall.sh" "$BIN_DIR/mnemosyne-uninstall"
fi
ln -sfn "$INSTALL_DIR/mnemosyne" "$BIN_DIR/mnemosyne"

# 4. Create Desktop Entry
desktop_exec="$INSTALL_DIR/mnemosyne"
cat > "$APPLICATIONS_DIR/mnemosyne.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Mnemosyne
Comment=Cross-platform AI vibe-coding & RE workspace
Exec=$desktop_exec
TryExec=$desktop_exec
Icon=applications-development
Terminal=false
Categories=Development;IDE;
EOF
chmod 644 "$APPLICATIONS_DIR/mnemosyne.desktop"

echo "✓ Mnemosyne installed to $INSTALL_DIR"
echo "✓ Symlink created at $BIN_DIR/mnemosyne"
echo "✓ Desktop launcher created at $APPLICATIONS_DIR/mnemosyne.desktop"

# 5. Check and install Claude Code CLI
echo ""
echo "Checking Claude Code CLI..."
if command -v claude >/dev/null 2>&1 || [ -x "$HOME/.local/bin/claude" ]; then
    echo "✓ Claude Code CLI is already installed"
else
    echo "Installing Claude Code CLI..."
    if command -v npm >/dev/null 2>&1; then
        npm install -g @anthropic-ai/claude-code || true
    elif command -v curl >/dev/null 2>&1; then
        curl -fsSL https://claude.ai/install.sh | bash || true
    else
        echo "⚠ Neither npm nor curl found. Please install Claude Code CLI manually."
    fi
fi

# 6. Check Decod
DECOD_DIR="$REPO_ROOT/../Decod"
if [ ! -d "$DECOD_DIR" ]; then
    DECOD_DIR="$HOME/Decod"
fi
if [ -f "$DECOD_DIR/setup_decod.py" ]; then
    echo ""
    echo "Setting up Decod dependencies..."
    python3 "$DECOD_DIR/setup_decod.py" --install-all || true
fi

echo ""
echo "========================================="
echo "  Installation Complete!                 "
echo "========================================="
echo "Launch Mnemosyne from your app menu or run: mnemosyne"
