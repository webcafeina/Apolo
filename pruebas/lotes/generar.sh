#!/usr/bin/env bash
# Genera una carpeta para probar Lotes a mano (ADR 0019), con ffmpeg:
#
#   pruebas/lotes/generar.sh <destino>      → <destino>/apolo-pruebas-lotes
#
# Lleva un poco de todo lo que Lotes tiene que saber tratar:
#   - fotos en subcarpetas, una con tilde en el nombre y otra con espacios;
#   - capturas de pantalla y logos con transparencia (PNG);
#   - TIFF, BMP, GIF, WebP y HEIC (la foto de ejemplo de libheif);
#   - pixel art y un GIF de patrón, que pesan más en WebP con pérdida;
#   - una imagen rota, que debe fallar sola;
#   - lo que no hay que recoger: «._foto.jpg» de macOS, .DS_Store, un .txt;
#   - «para-cancelar»: fotos de 12 megapíxeles, para que dé tiempo a cancelar.
#
# Las imágenes son sintéticas (fractales, patrones, degradados y grano de foto):
# nada de nadie. No van a git; el resultado pesa unos 100 MB.

set -euo pipefail

destino="${1:?uso: generar.sh <carpeta de destino>}"
raiz="$(cd "$(dirname "$0")/../.." && pwd)"
d="$destino/apolo-pruebas-lotes"
rm -rf "$d"
mkdir -p "$d"/vacaciones/{playa,"montaña",capturas,logos,iphone,varios} "$d/para-cancelar"

ff() { ffmpeg -hide_banner -loglevel error -y "$@"; }

# Una «foto»: un trozo del conjunto de Mandelbrot, con su color, grano y viñeta.
foto() { # salida ancho alto semilla
  local x y s h
  x=$(awk -v i="$4" 'BEGIN{printf "%.6f", -0.75 + 0.35*sin(i*1.7)}')
  y=$(awk -v i="$4" 'BEGIN{printf "%.6f", 0.1 + 0.3*cos(i*2.3)}')
  s=$(awk -v i="$4" 'BEGIN{printf "%.4f", 0.4 + (i%7)*0.25}')
  h=$(( ($4 * 47) % 360 ))
  ff -f lavfi -i "mandelbrot=s=${2}x${3}:start_x=$x:start_y=$y:start_scale=$s:maxiter=256:outer=normalized_iteration_count" \
    -vf "hue=h=$h:s=1.2,eq=contrast=1.1:brightness=0.02,noise=alls=9:allf=t+u,vignette=PI/5" \
    -frames:v 1 -q:v 3 "$1"
}

echo "Fotos…"
for i in 1 2 3 4 5 6; do foto "$d/vacaciones/playa/playa-$i.jpg" 2400 1600 "$i"; done
for i in 1 2 3 4 5; do foto "$d/vacaciones/montaña/cumbre-$i.jpg" 1600 2400 "$((i + 10))"; done
foto "$d/vacaciones/foto con espacios.jpg" 3000 2000 21

echo "Capturas y logos…"
ff -f lavfi -i "testsrc2=s=1920x1080" -frames:v 1 "$d/vacaciones/capturas/captura-1.png"
ff -f lavfi -i "smptehdbars=s=1920x1080" -frames:v 1 "$d/vacaciones/capturas/captura-2.png"
ff -f lavfi -i "life=s=640x400:mold=10:r=1:ratio=0.1:seed=7:death_color=#2b2b31:life_color=#ffc83d,scale=1280x800:flags=neighbor" \
  -frames:v 1 "$d/vacaciones/capturas/pixel-art.png"
for i in 1 2 3; do
  ff -f lavfi -i "gradients=s=512x512:c0=0xffc83d:c1=0xf08c1e:c2=0x2b2b31:n=3:seed=$i,format=rgba" \
    -vf "geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(lt(hypot(X-256,Y-256),200+20*sin(atan2(Y-256,X-256)*$((i + 4)))),255,0)'" \
    -frames:v 1 "$d/vacaciones/logos/logo-$i.png"
done

echo "Otros formatos…"
cp "$raiz/crates/heic/vendor/libheif/examples/example.heic" "$d/vacaciones/iphone/IMG_0001.heic"
ff -f lavfi -i "mandelbrot=s=1200x800:maxiter=200" -frames:v 1 -compression_algo deflate "$d/vacaciones/varios/escaneo.tiff"
ff -f lavfi -i "testsrc2=s=800x600" -frames:v 1 "$d/vacaciones/varios/antigua.bmp"
ff -f lavfi -i "cellauto=s=400x300:rule=110:seed=7,scale=800x600:flags=neighbor" -frames:v 1 "$d/vacaciones/varios/patron.gif"
ff -f lavfi -i "mandelbrot=s=1000x700:start_x=-0.5:maxiter=200" -frames:v 1 -c:v libwebp -quality 90 "$d/vacaciones/varios/ya-es-webp.webp"
# Diminuta: 8 × 8 píxeles de un solo color.
ff -f lavfi -i "color=c=0xffc83d:s=8x8" -frames:v 1 "$d/vacaciones/varios/punto.png"

echo "Lo que debe fallar o no recogerse…"
head -c 4096 /dev/urandom > "$d/vacaciones/varios/rota.jpg"
cp "$d/vacaciones/playa/playa-1.jpg" "$d/vacaciones/playa/._playa-1.jpg"
printf 'Bud1' > "$d/vacaciones/.DS_Store"
echo "Notas del viaje: esto no es una imagen y Lotes no debe contarlo." > "$d/vacaciones/notas.txt"

echo "Para cancelar (48 fotos de 12 megapíxeles)…"
for i in $(seq 1 48); do foto "$d/para-cancelar/$(printf 'IMG_%04d' "$i").jpg" 4000 3000 "$((i + 100))"; done

cat > "$d/LEEME.txt" <<'EOF'
Carpeta para probar Lotes en Apolo (v0.4.0). Las imágenes son sintéticas.

vacaciones/        25 imágenes que Apolo sabe leer, en subcarpetas:
  playa/           6 fotos JPG (más «._playa-1.jpg», que es de macOS y se salta)
  montaña/         5 fotos JPG en vertical (con tilde en el nombre de la carpeta)
  capturas/        3 PNG: dos capturas de pantalla y un pixel art
  logos/           3 PNG con transparencia
  iphone/          1 HEIC (la foto de ejemplo de libheif)
  varios/          TIFF, BMP, GIF, un WebP, un PNG diminuto y una imagen rota
  foto con espacios.jpg
  notas.txt y .DS_Store, que no son imágenes y no deben contar

Lo que debería pasar al convertir «vacaciones» con el preset por defecto:
  - Dice «25 imágenes»: 24 se convierten y 1 falla (varios/rota.jpg, que no
    es una imagen).
  - La salida propuesta es «vacaciones-webp», al lado de esta carpeta.
  - Dentro se repiten playa/, montaña/, capturas/, logos/, iphone/ y varios/.
  - Dos pesan más en WebP que su original y salen señaladas con «+»:
    capturas/pixel-art.png y varios/patron.gif, las dos unas cuatro veces
    más. Es lo normal en pixel art comprimido con pérdida.
  - En total, unos 12,9 MB → 3,1 MB (−76 %).
  - Repetir el lote a la misma salida crea playa-1-2.webp, etc.: no pisa nada.

para-cancelar/     48 fotos de 12 megapíxeles: tardan lo bastante para pulsar
                   «Cancelar» a medias.
EOF

du -sh "$d"
find "$d" -type f | wc -l
