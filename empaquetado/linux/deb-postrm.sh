#!/bin/sh
# Al purgar el .deb: quita el repositorio que dio de alta deb-postinst.sh, solo si lleva su marca (uno
# puesto a mano se queda).
if [ "$1" = purge ] && grep -qs "Puesto por el paquete de Apolo" /etc/apt/sources.list.d/apolo.sources; then
  rm -f /etc/apt/sources.list.d/apolo.sources /etc/apt/keyrings/apolo.asc
fi
exit 0
