#!/usr/bin/env sh
set -eu

SOURCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DATA_HOME=${XDG_DATA_HOME:-"$HOME/.local/share"}
INSTALL_DIR=$DATA_HOME/mnemosyne/app
BIN_DIR=${HOME}/.local/bin
APPLICATIONS_DIR=$DATA_HOME/applications

if [ ! -x "$SOURCE_DIR/mnemosyne" ]; then
    echo 'The mnemosyne binary is missing or not executable. Extract the complete package first.' >&2
    exit 1
fi

mkdir -p "$INSTALL_DIR" "$BIN_DIR" "$APPLICATIONS_DIR"
if [ -L "$BIN_DIR/mnemosyne" ]; then
    if [ "$(readlink "$BIN_DIR/mnemosyne")" != "$INSTALL_DIR/mnemosyne" ]; then
        echo "$BIN_DIR/mnemosyne already points to another target; refusing to replace it." >&2
        exit 1
    fi
elif [ -e "$BIN_DIR/mnemosyne" ]; then
    echo "$BIN_DIR/mnemosyne already exists and is not a symlink; refusing to replace it." >&2
    exit 1
fi
if [ -L "$BIN_DIR/mnemosyne-uninstall" ]; then
    if [ "$(readlink "$BIN_DIR/mnemosyne-uninstall")" != "$INSTALL_DIR/uninstall.sh" ]; then
        echo "$BIN_DIR/mnemosyne-uninstall already points to another target; refusing to replace it." >&2
        exit 1
    fi
elif [ -e "$BIN_DIR/mnemosyne-uninstall" ]; then
    echo "$BIN_DIR/mnemosyne-uninstall already exists and is not a symlink; refusing to replace it." >&2
    exit 1
fi

install -m 755 "$SOURCE_DIR/mnemosyne" "$INSTALL_DIR/mnemosyne"
install -m 755 "$SOURCE_DIR/uninstall.sh" "$INSTALL_DIR/uninstall.sh"
ln -sfn "$INSTALL_DIR/mnemosyne" "$BIN_DIR/mnemosyne"
ln -sfn "$INSTALL_DIR/uninstall.sh" "$BIN_DIR/mnemosyne-uninstall"
desktop_exec=$(printf '%s' "$INSTALL_DIR/mnemosyne" | sed 's/\\/\\\\/g; s/"/\\"/g; s/`/\\`/g; s/\$/\\$/g; s/%/%%/g')

cat > "$APPLICATIONS_DIR/mnemosyne.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Mnemosyne
Comment=Cross-platform coding-agent workspace
Exec="$desktop_exec"
TryExec="$desktop_exec"
Icon=applications-development
Terminal=false
Categories=Development;IDE;
EOF

chmod 644 "$APPLICATIONS_DIR/mnemosyne.desktop"
printf 'Mnemosyne installed for the current user. Launch it from your applications menu or run:\n%s\n' "$BIN_DIR/mnemosyne"
printf 'To uninstall, run: mnemosyne-uninstall\n'
