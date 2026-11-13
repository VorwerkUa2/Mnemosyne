#!/usr/bin/env sh
set -eu

DATA_HOME=${XDG_DATA_HOME:-"$HOME/.local/share"}
INSTALL_DIR=$DATA_HOME/mnemosyne/app
BIN_LINK=${HOME}/.local/bin/mnemosyne
UNINSTALL_LINK=${HOME}/.local/bin/mnemosyne-uninstall
DESKTOP_FILE=$DATA_HOME/applications/mnemosyne.desktop

if [ -L "$BIN_LINK" ] && [ "$(readlink -- "$BIN_LINK")" = "$INSTALL_DIR/mnemosyne" ]; then
    rm -f -- "$BIN_LINK"
fi
if [ -L "$UNINSTALL_LINK" ] && [ "$(readlink -- "$UNINSTALL_LINK")" = "$INSTALL_DIR/uninstall.sh" ]; then
    rm -f -- "$UNINSTALL_LINK"
fi
rm -f -- "$DESKTOP_FILE"
rm -rf -- "$DATA_HOME/mnemosyne"
printf 'Mnemosyne was uninstalled. Your projects and chat history were left untouched.\n'
