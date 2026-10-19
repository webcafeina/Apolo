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
# Las fotos son reales: el Kodak Lossless True Color Image Suite, 24 fotos que
# Kodak publicó para uso sin restricciones y que se usan como conjunto estándar
# para probar compresión (https://r0k.us/graphics/kodak/). Se descargan con su
# sha256 fijo, así que la carpeta sale siempre igual. Miden 768 × 512 (o 512 ×
# 768); las de «para-cancelar» se amplían a 12 megapíxeles. Las capturas, los
# logos, el pixel art, el GIF de patrón y la imagen rota son sintéticos: no son
# fotos, y cada uno prueba un caso. No van a git. Hasta el 2026-10-19 las fotos
# eran fractales de ffmpeg; el cliente pidió fotos de verdad.

set -euo pipefail

destino="${1:?uso: generar.sh <carpeta de destino>}"
raiz="$(cd "$(dirname "$0")/../.." && pwd)"
d="$destino/apolo-pruebas-lotes"
rm -rf "$d"
mkdir -p "$d"/vacaciones/{playa,"montaña",capturas,logos,iphone,varios} "$d/para-cancelar"

ff() { ffmpeg -hide_banner -loglevel error -y "$@"; }

# Las fotos de Kodak, en una caché, comprobadas con su sha256.
cache="${XDG_CACHE_HOME:-$HOME/.cache}/apolo-kodak"
mkdir -p "$cache"
while read -r suma n; do
  f="$cache/kodim$n.png"
  [ -f "$f" ] || curl -sSfL -o "$f" "https://r0k.us/graphics/kodak/kodak/kodim$n.png"
  echo "$suma  $f" | sha256sum -c --quiet || { echo "kodim$n.png no es la esperada" >&2; exit 1; }
done <<'SUMAS'
a56e27cbf5f843c048b6af1d6e090760e9c92fadba88b7dee0205918a37523bd 01
4f4b74a79237e311d72cad958237b5f7088d8bce1c82305ebefe1a70e3022dfd 02
e25ca1ff2f0c0cb5fdfd5f9b0a0bb21ac4c3de3c84a67f35b09a85d3306249db 03
e3b946107c5d3441c022f678d0c3caf1e224d81b1604ba840a4f88e562de61aa 04
10349e963c5c813d327852f82c1795fa4148d69fedffc4c589bee458e3ac3d53 05
363510303b715d4cbc384e1ce227e466b613a09e1b71ae985882bf8e7fbd9b18 06
b77d3f006f42414bb242222e0482e750c0fb9e5ee8d4bed2f6f11c5605fe54a4 07
ba23983c76b4832ee0e8af0592664756841a16779acd69f792e268fb6d13d6e7 08
6a4361c2fc194feb4edaa9f9a4a0620fb9943e460ac7fdf037fb0f6dd6607a7d 09
9dfb70f5867c29ff9ed6313683f19b3d867849e40fbc0c4c54a4a89df341cf23 10
7936814b58b5387fce2e4e2488b4ec830dadd95fa9520f358ddb30990b50f2b6 11
d78c37c2f04f23761ed2367dd77e2db584ddd4c3950833fecf89f199a8126980 12
bc34a3ce58dea09dce1704c997171602de90cb34d0c8503a988b77f473d39b08 13
55a94550ff18f3246c4074fd32b77b0c74447c26b6ad274d564d999c0450ba6e 14
7538cbb80cb9103606c48b806eae57d56c885c7f90b9b3be70a41160f9cbb683 15
a89c7268ccd4718ba424a99fc4643c572cf692ca6eae887185ceb4e9f11d2e54 16
37afcc89fbdcb76d9518e04b2fc011027e2f4cd14b3b2f83cefd721641a47c5b 17
1a9258c365988961d87a0598725b609139c303ad48a5aad6c503c3b1a87849aa 18
b7450b264b1b0a411390d8931b112c27905a992520fc90569dc4b920aa32bbdc 19
3b46c71e3b92a563820ba32936be8330c586c41f938efd94be938386aae4328a 20
ac958597c82073f6bb65129c68f72b651db5b9efd82e11547d07350214bc268b 21
1cee58eb1f2d9c7ebb254d208a03c783ce6cf2c4d8c2cf45e235dd23b4ce1b29 22
e3111a2fd4da24af15d6459ef9eacfe54106b38e27b4a21821b75c3f5d2d5baf 23
1071c68372cc5a01435c2c225a5cf7d4bb803846ec08bb6b3d6721b156d7cb96 24
SUMAS

# Una foto de Kodak como JPG, a su tamaño o a otro (escala: «4000:-2»).
foto() { # salida número [escala]
  ff -i "$cache/kodim$2.png" ${3:+-vf "scale=$3:flags=lanczos"} -q:v 3 "$1"
}

echo "Fotos…"
# Mar, lago y río para la playa; las verticales para la montaña; los loros.
i=1; for n in 12 21 16 06 11 14; do foto "$d/vacaciones/playa/playa-$i.jpg" "$n"; i=$((i + 1)); done
i=1; for n in 09 10 17 18 19; do foto "$d/vacaciones/montaña/cumbre-$i.jpg" "$n"; i=$((i + 1)); done
foto "$d/vacaciones/foto con espacios.jpg" 23

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
ff -i "$cache/kodim01.png" -compression_algo deflate "$d/vacaciones/varios/escaneo.tiff"
ff -i "$cache/kodim24.png" "$d/vacaciones/varios/antigua.bmp"
ff -f lavfi -i "cellauto=s=400x300:rule=110:seed=7,scale=800x600:flags=neighbor" -frames:v 1 "$d/vacaciones/varios/patron.gif"
ff -i "$cache/kodim22.png" -c:v libwebp -quality 90 "$d/vacaciones/varios/ya-es-webp.webp"
# Diminuta: 8 × 8 píxeles de un solo color.
ff -f lavfi -i "color=c=0xffc83d:s=8x8" -frames:v 1 "$d/vacaciones/varios/punto.png"

echo "Lo que debe fallar o no recogerse…"
head -c 4096 /dev/urandom > "$d/vacaciones/varios/rota.jpg"
cp "$d/vacaciones/playa/playa-1.jpg" "$d/vacaciones/playa/._playa-1.jpg"
printf 'Bud1' > "$d/vacaciones/.DS_Store"
echo "Notas del viaje: esto no es una imagen y Lotes no debe contarlo." > "$d/vacaciones/notas.txt"

echo "Para cancelar (48 fotos ampliadas a unos 12 megapíxeles)…"
for i in $(seq 1 48); do
  n=$(printf '%02d' $(( (i - 1) % 24 + 1 )))
  foto "$d/para-cancelar/$(printf 'IMG_%04d' "$i").jpg" "$n" "'if(gt(iw,ih),4240,-2)':'if(gt(iw,ih),-2,4240)'"
done

cat > "$d/LEEME.txt" <<'EOF'
Carpeta para probar Apolo. Las fotos son reales: el Kodak Lossless True Color
Image Suite (https://r0k.us/graphics/kodak/), que Kodak publicó para uso sin
restricciones. Las capturas, los logos, el pixel art y el GIF son sintéticos.

vacaciones/        25 imágenes que Apolo sabe leer, en subcarpetas:
  playa/           6 fotos JPG de mar y costa (más «._playa-1.jpg», que es de macOS y se salta)
  montaña/         5 fotos JPG en vertical (la tilde del nombre es lo que se prueba)
  capturas/        3 PNG: dos capturas de pantalla y un pixel art
  logos/           3 PNG con transparencia
  iphone/          1 HEIC (la foto de ejemplo de libheif)
  foto con espacios.jpg   los loros, con espacios en el nombre
  varios/          TIFF, BMP, GIF, un WebP, un PNG diminuto y una imagen rota
  notas.txt y .DS_Store, que no son imágenes y no deben contar

Lo que debería pasar al convertir «vacaciones» con el preset por defecto:
  - Dice «25 imágenes»: 24 se convierten y 1 falla (varios/rota.jpg, que no
    es una imagen).
  - La salida propuesta es «vacaciones-webp», al lado de esta carpeta.
  - Dentro se repiten playa/, montaña/, capturas/, logos/, iphone/ y varios/.
  - Dos pesan más en WebP que su original y salen señaladas con «+»:
    capturas/pixel-art.png y varios/patron.gif, las dos unas cuatro veces
    más. Es lo normal en pixel art comprimido con pérdida.
  - En total, unos 4,1 MB → 1,3 MB (−68 %).
  - Repetir el lote a la misma salida crea playa-1-2.webp, etc.: no pisa nada.

para-cancelar/     48 fotos ampliadas a 12 megapíxeles: tardan lo bastante para
                   pulsar «Cancelar» a medias.
EOF

du -sh "$d"
find "$d" -type f | wc -l
