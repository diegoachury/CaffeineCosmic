#!/usr/bin/env bash
# Instalación local desde el código fuente (para usuarios que no usan Flatpak).
# Reutiliza el Makefile con PREFIX=$HOME/.local y añade el autoarranque de COSMIC.
set -euo pipefail

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="$HOME/.local"
APPID="io.github.diegoachury.CaffeineCosmic"
AUTOSTART_DIR="$HOME/.config/autostart"

echo "==> Compilando e instalando (PREFIX=$PREFIX)…"
make -C "$SRC_DIR" build
make -C "$SRC_DIR" install PREFIX="$PREFIX"

echo "==> Configurando autoarranque…"
mkdir -p "$AUTOSTART_DIR"
# El autostart de COSMIC no garantiza PATH, así que usamos ruta absoluta en Exec.
sed "s|^Exec=.*|Exec=$PREFIX/bin/cosmic-caffeine|" \
    "$SRC_DIR/data/$APPID.desktop" > "$AUTOSTART_DIR/$APPID.desktop"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "==> Listo."
echo "    Inicia ahora con:  cosmic-caffeine &"
echo "    (Asegúrate de que $PREFIX/bin esté en tu PATH.)"
