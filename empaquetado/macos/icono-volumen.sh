#!/bin/bash
# Pone el icono del volumen (el disco con el sol, disco.svg) en un .dmg ya hecho.
#
# Tauri monta el .dmg con el icono de la aplicación como icono del volumen y no
# deja elegir otro, así que se cambia después: se pasa la imagen a lectura y
# escritura, se monta, se sustituye .VolumeIcon.icns, y se vuelve a comprimir.
# Solo corre en macOS (iconutil, hdiutil, SetFile). Lo llama publicar.yml.
#
# Uso: icono-volumen.sh Apolo_X.Y.Z_universal.dmg
set -euo pipefail

dmg="$1"
aqui=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
trap 'hdiutil detach "$tmp/vol" -quiet 2>/dev/null || true; rm -rf "$tmp"' EXIT

iconutil -c icns "$aqui/disco.iconset" -o "$tmp/disco.icns"

hdiutil convert "$dmg" -format UDRW -o "$tmp/rw.dmg" -quiet
# Holgura para el icono nuevo: la imagen de Tauri va justa.
hdiutil resize -size "$(( $(stat -f%z "$tmp/rw.dmg") / 1048576 + 20 ))m" "$tmp/rw.dmg"
mkdir "$tmp/vol"
hdiutil attach "$tmp/rw.dmg" -mountpoint "$tmp/vol" -nobrowse -noautoopen -quiet
cp "$tmp/disco.icns" "$tmp/vol/.VolumeIcon.icns"
SetFile -c icnC "$tmp/vol/.VolumeIcon.icns"
SetFile -a C "$tmp/vol"
hdiutil detach "$tmp/vol" -quiet

rm -f "$dmg"
hdiutil convert "$tmp/rw.dmg" -format UDZO -imagekey zlib-level=9 -o "$dmg" -quiet
echo "Icono del volumen puesto en $dmg"
