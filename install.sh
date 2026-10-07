#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
DEST="$HOME/.local/bin/t2d2"

cd "$REPO_ROOT"
cargo build --release

mkdir -p "$(dirname "$DEST")"
install -m 0755 "target/release/t2d2" "$DEST"

echo "Installed: $DEST"

case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "Note: $HOME/.local/bin is not on your PATH. Add it or invoke with full path." ;;
esac