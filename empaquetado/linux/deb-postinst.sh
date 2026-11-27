#!/bin/sh
# Tras instalar el .deb: da de alta el repositorio APT de Apolo, con los mismos ficheros que la web
# (apolo.webcafeina.com, «Linux»), para que quien instaló el .deb descargado también se ponga al día
# con apt upgrade, no solo desde la ventana. Si ya hay una fuente que apunta al repositorio (puesta a
# mano), no se toca nada. Lo que se pone lleva una marca para quitarlo al purgar (deb-postrm.sh). Si
# algo falla, la instalación sigue: es una comodidad, no parte de Apolo.
if [ "$1" = configure ] && ! grep -rqs "apolo.webcafeina.com/apt" /etc/apt/sources.list /etc/apt/sources.list.d/; then
  {
    install -d -m 0755 /etc/apt/keyrings &&
      install -m 0644 /usr/share/apolo/apolo.asc /etc/apt/keyrings/apolo.asc &&
      cat > /etc/apt/sources.list.d/apolo.sources <<'FUENTE'
# Puesto por el paquete de Apolo: se quita al purgarlo (apt purge apolo).
Types: deb
URIs: https://apolo.webcafeina.com/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/apolo.asc
FUENTE
  } || echo "Apolo: no se ha podido dar de alta su repositorio APT; se actualiza igual desde la ventana." >&2
fi
exit 0
