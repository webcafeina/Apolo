# Lo común de las copias de seguridad de Apolo (docs/restaurar.md): las credenciales, rclone y el
# registro. Se carga con `source` desde claves.sh y semanal.sh.
#
# Las credenciales viven en ~/.config/apolo/copias.env (600), que rellena el cliente y que nunca se
# copia a ningún sitio. rclone va en un contenedor efímero (no se instala nada) y se configura por
# variables de entorno pasadas con --env-file, no con -e, para que los secretos no salgan en
# `docker inspect` ni en la lista de procesos. Es lo mismo que la copia de webcafeina-local.

set -euo pipefail
umask 077

# Cron no hereda el PATH: todo con rutas absolutas.
DOCKER=/usr/bin/docker
GIT=/usr/bin/git
GPG=/usr/bin/gpg
CURL=/usr/bin/curl
RCLONE_IMAGEN=rclone/rclone:1.71.2

CONFIG="$HOME/.config/apolo"
ENV_FILE="$CONFIG/copias.env"
# Persistente a propósito: docs/restaurar.md lo usa para bajar la copia el día que haga falta.
RCLONE_ENV="$CONFIG/copias.rclone.env"
REGISTRO_DIR="$HOME/backups/apolo"
REGISTRO="$REGISTRO_DIR/copias.log"
mkdir -p "$REGISTRO_DIR"

registro() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

if [[ ! -f "$ENV_FILE" ]]; then
  registro "ERROR: falta $ENV_FILE."
  exit 1
fi
set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a
for v in R2_ACCOUNT_ID R2_ACCESS_KEY_ID R2_SECRET_ACCESS_KEY R2_BUCKET; do
  if [[ -z "${!v:-}" || "${!v}" == CAMBIAME* ]]; then
    registro "ERROR: $v sin rellenar en $ENV_FILE."
    exit 1
  fi
done

cat > "$RCLONE_ENV" <<FIN
RCLONE_CONFIG_R2_TYPE=s3
RCLONE_CONFIG_R2_PROVIDER=Cloudflare
RCLONE_CONFIG_R2_ACCESS_KEY_ID=$R2_ACCESS_KEY_ID
RCLONE_CONFIG_R2_SECRET_ACCESS_KEY=$R2_SECRET_ACCESS_KEY
RCLONE_CONFIG_R2_ENDPOINT=https://$R2_ACCOUNT_ID.r2.cloudflarestorage.com
RCLONE_CONFIG_R2_ACL=private
FIN

# rclone sobre una carpeta local, montada en /data. `--s3-no-check-bucket` es imprescindible: sin
# él, una errata en R2_BUCKET crearía otro bucket y la copia saldría «bien» en el sitio equivocado.
# (Con el token de este bucket ni siquiera podría: da 403, que es lo que debe pasar.)
rc() {
  local carpeta="$1"
  shift
  "$DOCKER" run --rm --env-file "$RCLONE_ENV" -v "$carpeta:/data" "$RCLONE_IMAGEN" \
    --config /dev/null --s3-no-check-bucket "$@"
}
