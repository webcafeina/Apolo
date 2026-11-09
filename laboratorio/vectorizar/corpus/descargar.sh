#!/usr/bin/env bash
# Vuelve a bajar las fuentes del corpus (las de LICENCIAS.md). Los SVG ya están en git; las fotos no,
# por su peso: este guion las baja de Wikimedia Commons y copia las dos que ya tiene el repositorio.
set -euo pipefail
cd "$(dirname "$0")/fuentes"
raiz=../../../..
mkdir -p fotos
ua="ApoloBanco/0.1 (https://apolo.webcafeina.com)"
baja() { [ -s "$1" ] || curl -sfL --max-time 60 -A "$ua" -o "$1" "$2"; }
w=https://upload.wikimedia.org/wikipedia/commons/thumb
baja fotos/arandanos.jpg "$w/0/0b/Blueberries-In-Pack.jpg/1280px-Blueberries-In-Pack.jpg"
baja fotos/pera.png "$w/8/82/Golden_Asian_Pear_2.png/1280px-Golden_Asian_Pear_2.png"
baja fotos/retrato.jpg "$w/6/65/Portrait_of_a_Woman_MET_DP162139.jpg/1280px-Portrait_of_a_Woman_MET_DP162139.jpg"
[ -e fotos/mascara.webp ] || cp "$raiz/web/public/comparador/mascara/original.webp" fotos/mascara.webp
[ -e fotos/libwebp.webp ] || cp "$raiz/pruebas/corpus/foto.webp" fotos/libwebp.webp
echo "fotos listas"
