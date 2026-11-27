#!/bin/sh
# Tras instalar el .rpm: da de alta el repositorio RPM de Apolo, con el mismo fichero que la web, para
# que quien instaló el .rpm descargado también se ponga al día con dnf upgrade (o zypper), no solo
# desde la ventana. Si ya hay uno que apunta al repositorio, no se toca nada. Lo que se pone lleva una
# marca para quitarlo al desinstalar (rpm-postun.sh). Si algo falla, la instalación sigue.
for d in /etc/yum.repos.d /etc/zypp/repos.d; do
  if [ -d "$d" ] && ! grep -rqs "apolo.webcafeina.com/rpm" "$d"; then
    cat > "$d/apolo.repo" 2>/dev/null <<'REPO' || echo "Apolo: no se ha podido dar de alta su repositorio en $d." >&2
# Puesto por el paquete de Apolo: se quita al desinstalarlo.
[apolo]
name=Apolo
baseurl=https://apolo.webcafeina.com/rpm
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=https://apolo.webcafeina.com/apolo.asc
REPO
  fi
done
exit 0
