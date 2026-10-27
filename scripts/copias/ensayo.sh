#!/usr/bin/env bash
# El ensayo mensual de restauración (docs/restaurar.md), desde la copia de R2, que es la que hay que
# demostrar: baja el repositorio de la última semana y lo clona, baja la última copia de las claves,
# la descifra con su frase y la compara con las claves de verdad, y comprueba las sumas de la última
# versión guardada. Si algo no cuadra, avisa a healthchecks.io (/fail).
#
# Cron, el día 1 de cada mes a las 05:00:
#   0 5 1 * * /home/webcafeina/proyectos/apolo/scripts/copias/ensayo.sh

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/comun.sh"
exec >> "$REGISTRO" 2>&1

TEMPORAL="$(mktemp -d)"
fin() {
  local codigo=$?
  rm -rf "$TEMPORAL"
  if [[ $codigo -ne 0 ]]; then
    registro "ENSAYO FALLIDO (código $codigo)."
    [[ -n "${HEALTHCHECK_URL:-}" ]] && "$CURL" -fsS -m 10 --retry 3 "$HEALTHCHECK_URL/fail" -o /dev/null || true
  fi
  exit $codigo
}
trap fin EXIT
registro "=== Ensayo de restauración ==="

# El repositorio de la última semana: se clona y tiene main y las etiquetas de versión.
semana="$(rc "$TEMPORAL" lsf --dirs-only "r2:$R2_BUCKET/repos" | sort | tail -1 | tr -d /)"
mkdir -p "$TEMPORAL/repos"
rc "$TEMPORAL/repos" copy "r2:$R2_BUCKET/repos/$semana" /data
"$GIT" clone -q "$TEMPORAL/repos/Apolo.bundle" "$TEMPORAL/Apolo"
"$GIT" -C "$TEMPORAL/Apolo" rev-parse -q --verify origin/main > /dev/null
[[ -n "$("$GIT" -C "$TEMPORAL/Apolo" tag -l 'v*')" ]]
registro "Repositorio de $semana: se clona, con main y $("$GIT" -C "$TEMPORAL/Apolo" tag -l 'v*' | wc -l) etiquetas."

# Las claves: la última copia se descifra y es idéntica a lo que hay en el VPS.
copia="$(rc "$TEMPORAL" lsf "r2:$R2_BUCKET/claves" | sort | tail -1)"
mkdir -p "$TEMPORAL/claves"
rc "$TEMPORAL/claves" copyto "r2:$R2_BUCKET/claves/$copia" "/data/$copia"
"$GPG" --batch --quiet --pinentry-mode loopback --passphrase-file "$CONFIG/claves-copia.frase" \
  -d "$TEMPORAL/claves/$copia" | tar -C "$TEMPORAL/claves" -xf -
diff -rq "$TEMPORAL/claves/claves" "$CONFIG/claves" > /dev/null || {
  registro "Las claves de $copia no son las del VPS: ¿cambió una clave sin hacer otra copia (claves.sh)?"
  exit 1
}
registro "Claves ($copia): se descifran y son las del VPS."

# La última versión guardada: sus ficheros cuadran con su SHA256SUMS.txt.
version="$(rc "$TEMPORAL" lsf --dirs-only "r2:$R2_BUCKET/versiones" | sort -V | tail -1 | tr -d /)"
mkdir -p "$TEMPORAL/version"
rc "$TEMPORAL/version" copy "r2:$R2_BUCKET/versiones/$version" /data
(cd "$TEMPORAL/version" && grep -v ' SHA256SUMS.txt$' SHA256SUMS.txt | sha256sum -c --quiet -)
registro "Versión $version: las sumas cuadran."
registro "Ensayo bien."
