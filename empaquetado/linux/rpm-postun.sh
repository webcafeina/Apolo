#!/bin/sh
# Al desinstalar el .rpm (no al actualizarlo: entonces $1 es 1): quita el repositorio que dio de alta
# rpm-post.sh, solo si lleva su marca.
if [ "$1" = 0 ]; then
  for f in /etc/yum.repos.d/apolo.repo /etc/zypp/repos.d/apolo.repo; do
    if grep -qs "Puesto por el paquete de Apolo" "$f"; then rm -f "$f"; fi
  done
fi
exit 0
