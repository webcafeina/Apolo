#!/usr/bin/env bash
# La copia cifrada de las claves de firma de Apolo (docs/restaurar.md): la del actualizador
# (minisign), la GPG de los paquetes y la SSH del AUR, todo ~/.config/apolo/claves/, en un tar
# cifrado con GPG (AES-256) con la frase de ~/.config/apolo/claves-copia.frase, que el cliente
# guarda también en Coda. Va a R2 (claves/) y se queda una copia en ~/backups/apolo/ para bajarla
# a otro ordenador. Antes de darla por buena, se baja de R2, se descifra y se compara con el
# original.
#
# Se lanza a mano: una vez, y otra cada vez que cambie una clave.
#   scripts/copias/claves.sh

source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/comun.sh"
exec > >(tee -a "$REGISTRO") 2>&1

FRASE="$CONFIG/claves-copia.frase"
[[ -s "$FRASE" ]] || { registro "ERROR: falta $FRASE."; exit 1; }

TEMPORAL="$(mktemp -d)"
trap 'rm -rf "$TEMPORAL"' EXIT
NOMBRE="apolo-claves-$(date +%F).tar.gpg"

registro "=== Copia de las claves ($NOMBRE) ==="
tar -C "$CONFIG" -cf "$TEMPORAL/claves.tar" claves
"$GPG" --batch --yes --quiet --pinentry-mode loopback --passphrase-file "$FRASE" \
  --symmetric --cipher-algo AES256 -o "$TEMPORAL/$NOMBRE" "$TEMPORAL/claves.tar"

rc "$TEMPORAL" copyto "/data/$NOMBRE" "r2:$R2_BUCKET/claves/$NOMBRE"
mkdir -p "$TEMPORAL/vuelta"
rc "$TEMPORAL/vuelta" copyto "r2:$R2_BUCKET/claves/$NOMBRE" "/data/$NOMBRE"
"$GPG" --batch --quiet --pinentry-mode loopback --passphrase-file "$FRASE" \
  -d -o "$TEMPORAL/vuelta/claves.tar" "$TEMPORAL/vuelta/$NOMBRE"
if ! cmp -s "$TEMPORAL/claves.tar" "$TEMPORAL/vuelta/claves.tar"; then
  registro "ERROR: lo que hay en R2 no se descifra igual que el original."
  exit 1
fi
cp "$TEMPORAL/$NOMBRE" "$REGISTRO_DIR/$NOMBRE"
registro "Bien: r2:$R2_BUCKET/claves/$NOMBRE, descifrado y comparado; copia local en $REGISTRO_DIR/$NOMBRE."
