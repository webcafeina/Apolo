#!/usr/bin/env bash
# La copia semanal de Apolo a R2 (docs/restaurar.md):
#   - repos/AAAA-MM-DD/: el repositorio entero y el tap de Homebrew, como git bundle (todas las
#     ramas y etiquetas). Se guardan las 4 últimas semanas.
#   - motores/: el código de cada submódulo de terceros (libheif, aom, libjxl…) en el commit que
#     usa Apolo, como tar.gz. Solo se sube si ese commit no está ya: no cambia de una semana a otra.
#   - versiones/vX.Y.Z/: los ficheros de las 3 últimas versiones publicadas. Las más viejas se
#     borran de R2 (siguen en GitHub mientras exista).
# Avisa a healthchecks.io al empezar, al acabar bien y si falla; si un domingo no llega nada, avisa
# él solo.
#
# Cron, los domingos a las 04:00 (la copia de webcafeina-local va a las 03:00):
#   0 4 * * 0 /home/webcafeina/proyectos/apolo/scripts/copias/semanal.sh

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/comun.sh"
exec >> "$REGISTRO" 2>&1

REPO_LOCAL="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/../.." && pwd)"
SEMANAS=4
VERSIONES=3
TEMPORAL="$(mktemp -d)"

aviso() { [[ -n "${HEALTHCHECK_URL:-}" ]] && "$CURL" -fsS -m 10 --retry 3 "$HEALTHCHECK_URL$1" -o /dev/null || true; }
fin() {
  local codigo=$?
  rm -rf "$TEMPORAL"
  if [[ $codigo -ne 0 ]]; then
    registro "FALLO (código $codigo)."
    aviso /fail
  fi
  exit $codigo
}
trap fin EXIT

HOY="$(date +%F)"
registro "=== Copia semanal ($HOY) ==="
aviso /start

# 1. Los repositorios, enteros, desde GitHub (no desde el clon local, que puede tener cosas a medias).
mkdir -p "$TEMPORAL/repos"
for repo in Apolo homebrew-apolo; do
  "$GIT" clone -q --mirror "https://github.com/webcafeina/$repo.git" "$TEMPORAL/espejo-$repo"
  "$GIT" -C "$TEMPORAL/espejo-$repo" bundle create -q "$TEMPORAL/repos/$repo.bundle" --all
  "$GIT" bundle verify -q "$TEMPORAL/repos/$repo.bundle"
  rm -rf "$TEMPORAL/espejo-$repo"
done
rc "$TEMPORAL/repos" copy /data "r2:$R2_BUCKET/repos/$HOY"
registro "Repositorios: $(du -sh "$TEMPORAL/repos" | cut -f1) en repos/$HOY."
# Fuera las semanas de más (las carpetas van por fecha: el orden alfabético es el de la fecha).
rc "$TEMPORAL" lsf --dirs-only "r2:$R2_BUCKET/repos" | sort | head -n -"$SEMANAS" | while read -r vieja; do
  rc "$TEMPORAL" purge "r2:$R2_BUCKET/repos/$vieja"
  registro "Borrada la semana repos/$vieja."
done
rm -rf "$TEMPORAL/repos"

# 2. Los motores de terceros, en el commit que tiene cada submódulo en este clon (el de main).
mkdir -p "$TEMPORAL/motores"
YA="$(rc "$TEMPORAL" lsf "r2:$R2_BUCKET/motores" 2>/dev/null || true)"
"$GIT" -C "$REPO_LOCAL" submodule status | sed 's/^[ +-]//' | awk '{print $1, $2}' | while read -r commit ruta; do
  nombre="$(basename "$ruta")-$commit.tar.gz"
  grep -qxF "$nombre" <<< "$YA" && continue
  "$GIT" -C "$REPO_LOCAL/$ruta" archive --format=tar.gz -o "$TEMPORAL/motores/$nombre" "$commit"
  registro "Motor nuevo: $nombre."
done
if [[ -n "$(ls -A "$TEMPORAL/motores")" ]]; then
  rc "$TEMPORAL/motores" copy /data "r2:$R2_BUCKET/motores"
fi
rm -rf "$TEMPORAL/motores"

# 3. Las últimas versiones publicadas, de la API pública de GitHub (sin sesión: cron no la tiene).
ETIQUETAS="$("$CURL" -fsS "https://api.github.com/repos/webcafeina/Apolo/releases?per_page=$VERSIONES" |
  python3 -c 'import json, sys; [print(r["tag_name"]) for r in json.load(sys.stdin) if not r["draft"]]')"
EN_R2="$(rc "$TEMPORAL" lsf --dirs-only "r2:$R2_BUCKET/versiones" 2>/dev/null | tr -d / || true)"
for etiqueta in $ETIQUETAS; do
  grep -qxF "$etiqueta" <<< "$EN_R2" && continue
  mkdir -p "$TEMPORAL/$etiqueta"
  "$CURL" -fsS "https://api.github.com/repos/webcafeina/Apolo/releases/tags/$etiqueta" |
    python3 -c 'import json, sys; [print(a["browser_download_url"]) for a in json.load(sys.stdin)["assets"]]' |
    while read -r url; do "$CURL" -fsSL --retry 3 -o "$TEMPORAL/$etiqueta/$(basename "$url")" "$url"; done
  # Que lo bajado cuadre con su SHA256SUMS.txt antes de guardarlo.
  (cd "$TEMPORAL/$etiqueta" && grep -v ' SHA256SUMS.txt$' SHA256SUMS.txt | sha256sum -c --quiet -)
  rc "$TEMPORAL/$etiqueta" copy /data "r2:$R2_BUCKET/versiones/$etiqueta"
  registro "Versión $etiqueta: $(du -sh "$TEMPORAL/$etiqueta" | cut -f1), sumas comprobadas."
  rm -rf "${TEMPORAL:?}/$etiqueta"
done
for etiqueta in $EN_R2; do
  grep -qxF "$etiqueta" <<< "$ETIQUETAS" && continue
  rc "$TEMPORAL" purge "r2:$R2_BUCKET/versiones/$etiqueta"
  registro "Borrada la versión $etiqueta (ya no es de las $VERSIONES últimas)."
done

registro "Bien. En R2: $(rc "$TEMPORAL" size "r2:$R2_BUCKET" 2>/dev/null | tr '\n' ' ')"
aviso ""
