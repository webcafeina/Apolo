#!/usr/bin/env bash
# El banco de rendimiento (entrega 8b, ADR 0024): tiempo y pico de memoria de
# Apolo con fotos de 24 megapíxeles, para saber si cabe en un Mac de 8 GB.
#
#   scripts/banco.sh            → target/banco/resultado.md
#   BANCO_LOTE=40 scripts/banco.sh   (un lote más corto)
#
# Las fotos son las 24 de Kodak (pruebas/lotes/generar.sh las deja en caché)
# ampliadas a 6000 × 4000, en JPEG de calidad alta, como las de una cámara.
# Ampliadas tienen menos detalle que una foto de 24 MP de verdad: el tiempo de
# codificar puede salir algo corto, pero la memoria depende de los píxeles.
# El lote son 200 enlaces duros a esas 24, que no ocupan disco.
#
# Los lotes van con -j 8: un Mac con M1 tiene 8 núcleos y Lotes usa uno por
# núcleo (hasta ocho), aunque esta máquina tenga menos. Cada caso pasa por
# /usr/bin/time (GNU time): el tiempo y el pico de memoria residente.
# No cambia ningún byte de salida: solo mide.

set -euo pipefail

raiz="$(cd "$(dirname "$0")/.." && pwd)"
cd "$raiz"
lote="${BANCO_LOTE:-200}"
b="$raiz/target/banco"
cache="${XDG_CACHE_HOME:-$HOME/.cache}"
kodak="$cache/apolo-kodak"
fotos="$cache/apolo-banco/fotos24"

[ -x /usr/bin/time ] || { echo "falta GNU time (/usr/bin/time)" >&2; exit 1; }
if [ ! -f "$kodak/kodim24.png" ]; then
  echo "Faltan las fotos de Kodak: pruebas/lotes/generar.sh las descarga." >&2
  pruebas/lotes/generar.sh "$(mktemp -d)" >/dev/null
fi

# Las 24 fotos de 24 MP, una vez.
mkdir -p "$fotos"
for f in "$kodak"/kodim*.png; do
  n="$(basename "$f" .png)"
  [ -f "$fotos/$n.jpg" ] && continue
  # Las verticales (512 × 768), en vertical.
  if ffprobe -v error -select_streams v -show_entries stream=height -of csv=p=0 "$f" | grep -q 768; then
    escala=4000:6000
  else
    escala=6000:4000
  fi
  ffmpeg -hide_banner -loglevel error -y -i "$f" -vf "scale=$escala:flags=lanczos" -q:v 2 "$fotos/$n.jpg"
done

rm -rf "$b"
mkdir -p "$b/lote" "$b/salida"
for i in $(seq 1 "$lote"); do
  ln "$fotos/kodim$(printf %02d $(( (i - 1) % 24 + 1 ))).jpg" "$b/lote/foto-$(printf %03d "$i").jpg"
done
foto="$fotos/kodim01.jpg"
ffmpeg -hide_banner -loglevel error -y -i "$foto" "$b/foto.png"

export APOLO_CLANG="${APOLO_CLANG:-$HOME/.local/llvm18/bin}"
cargo build --release -q -p apolo-cli -p apolo-mcp
cargo build --release -q -p apolo-servicio --example banco_estudio
apolo="$raiz/target/release/apolo"
mcp="$raiz/target/release/apolo-mcp"
export APOLO_CONFIG="$b/config"

informe="$b/resultado.md"
{
  echo "# Banco de rendimiento"
  echo
  echo "$(date -I) · $(git rev-parse --short HEAD) · $(nproc) núcleos · $(free -g | awk '/Mem:/{print $2}') GB"
  echo "· foto de $(ffprobe -v error -show_entries stream=width,height -of csv=p=0:s=x "$foto") · lote de $lote"
  echo
  echo "| Caso | Segundos | Pico (MB) |"
  echo "|---|---:|---:|"
} > "$informe"

# medir <nombre> <orden…>: una fila de la tabla.
medir() {
  local nombre="$1"; shift
  rm -rf "$b/salida"; mkdir -p "$b/salida"
  local t
  if ! t=$( { /usr/bin/time -f "%e %M" "$@" >/dev/null 2>"$b/error.txt"; } 2>&1 ); then
    t=$(tail -1 "$b/error.txt")
    echo "| $nombre | falló | |" >> "$informe"
    echo "$nombre: falló ($(tail -3 "$b/error.txt"))" >&2
    return
  fi
  t=$(tail -1 "$b/error.txt")
  local s kb
  read -r s kb <<< "$t"
  printf "| %s | %s | %d |\n" "$nombre" "$s" $(( kb / 1024 )) >> "$informe"
  printf "%-45s %8ss %8d MB\n" "$nombre" "$s" $(( kb / 1024 ))
}

echo "Una foto con la CLI:"
medir "apolo webp" "$apolo" webp "$foto" -o "$b/salida/f.webp"
medir "apolo jpeg" "$apolo" jpeg -outfile "$b/salida/f.jpg" "$foto"
medir "apolo png (oxipng)" "$apolo" png "$b/foto.png" --out "$b/salida/f.png"
medir "apolo qoi" "$apolo" qoi "$foto" "$b/salida/f.qoi"
medir "apolo avif" "$apolo" avif "$foto" "$b/salida/f.avif"
medir "apolo jxl" "$apolo" jxl "$foto" "$b/salida/f.jxl"
"$apolo" webp "$foto" -o "$b/f.webp" 2>/dev/null
medir "apolo medir (SSIMULACRA 2)" "$apolo" medir "$foto" "$b/f.webp"

echo "Lotes ($lote fotos, 8 a la vez):"
medir "lote → webp" "$apolo" lote -q -j 8 "$b/lote" -s "$b/salida/l" -f webp
medir "lote → avif" "$apolo" lote -q -j 8 "$b/lote" -s "$b/salida/l" -f avif
medir "lote → webp,jpeg,avif, el más ligero" "$apolo" lote -q -j 8 "$b/lote" -s "$b/salida/l" -f webp,jpeg,avif --mas-ligero
medir "lote → webp midiendo" "$apolo" lote -q -j 8 "$b/lote" -s "$b/salida/l" -f webp --medir

echo "El Estudio (el Servicio de la ventana):"
# Su pico no lo da /usr/bin/time: el ejemplo lo pone a cero en cada paso para
# medirlos por separado, y time solo ve el del último. Lo da el propio ejemplo.
inicio=$(date +%s%N)
"$raiz/target/release/examples/banco_estudio" "$foto" > "$b/estudio.txt"
s=$(awk -v a="$inicio" -v b="$(date +%s%N)" 'BEGIN{printf "%.2f", (b-a)/1e9}')
pico=$(awk -F'\t' '/^pico de la sesión/{print $3}' "$b/estudio.txt")
printf "| Estudio: la sesión entera | %.2f | %s |\n" "$s" "$pico" >> "$informe"
printf "%-45s %8.2fs %8s MB\n" "Estudio: la sesión entera" "$s" "$pico"

echo "El MCP:"
medir "apolo-mcp: comparar_formatos" python3 -I "$raiz/scripts/banco_mcp.py" "$mcp" "$foto" "$b/salida"

{
  echo
  echo "Paso a paso del Estudio (segundos y pico en MB de cada paso):"
  echo
  echo '```'
  cat "$b/estudio.txt"
  echo '```'
} >> "$informe"

echo
echo "→ $informe"
