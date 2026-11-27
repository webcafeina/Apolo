# ADR 0040 — El .deb y el .rpm descargados dan de alta el repositorio

**Fecha:** 2026-11-27 · **Estado:** hecha · publicada en la 1.1.3 y comprobada por el cliente

## Contexto

El cliente preguntó si Apolo se actualiza «de forma cruzada», como Esfinge: instalado por terminal o con el
instalador descargado, que se pueda poner al día desde la ventana y desde la terminal. Repasado sistema por
sistema, casi todo ya lo cumplía (macOS con Homebrew, Windows con winget, que reconoce también una instalación
hecha a mano, y los repositorios APT y RPM). Quedaban dos huecos:

- **El `.deb` y el `.rpm` descargados de la web** se actualizaban desde la ventana (pide la contraseña con
  `pkexec`), pero `apt upgrade` y `dnf upgrade` no los veían: nadie había dado de alta el repositorio. Es como
  instala casi todo el mundo en Linux.
- El `.dmg` en macOS, que Homebrew no conoce. El cliente decidió no hacer nada con este: `brew install --cask
  --adopt apolo` es demasiado concreto para documentarlo.

Chocolatey y Arch siguen sin actualizarse desde la ventana a propósito (su gestor se quedaría con una versión
vieja; ADR 0030 y el marcador `gestor`).

## Decisión

Al instalarse, el `.deb` y el `.rpm` dan de alta el repositorio de Apolo, como hacen Chrome o VS Code:

- **`.deb`** (`empaquetado/linux/deb-postinst.sh`): en `configure`, si ninguna fuente de `/etc/apt` apunta ya a
  `apolo.webcafeina.com/apt`, copia la clave del paquete (`/usr/share/apolo/apolo.asc`, la misma que la web) a
  `/etc/apt/keyrings/apolo.asc` y escribe `/etc/apt/sources.list.d/apolo.sources`, **línea por línea igual** que
  el que publica `web.yml`. `deb-postrm.sh` los quita **solo al purgar** y solo si llevan la marca «Puesto por el
  paquete de Apolo».
- **`.rpm`** (`rpm-post.sh`): en `/etc/yum.repos.d` (Fedora) y `/etc/zypp/repos.d` (openSUSE), si no hay ya un
  repositorio que apunte a `apolo.webcafeina.com/rpm`, escribe `apolo.repo`, igual que el de la web.
  `rpm-postun.sh` lo quita al desinstalar (`$1 = 0`), no al actualizar, y solo si lleva la marca.
- **Lo puesto a mano no se toca** ni se duplica: se busca la URL, no el nombre del fichero.
- **Si algo falla, la instalación sigue**: es una comodidad, no parte de Apolo. Los cuatro scripts salen con 0.

## Alternativas descartadas

- **Explicarlo solo en la web** («para recibir actualizaciones con apt, añade el repositorio»): ya lo explica, y
  quien baja el `.deb` no lo lee.
- **Un paquete aparte `apolo-repositorio`**, como hacen algunos proyectos: una descarga más que nadie hace.
- **Quitar el repositorio también con `apt remove`**: lo normal en Debian es dejar la configuración hasta el
  `purge`, y así lo hace Chrome.
- **Que el `.deb` lleve el `.sources` como fichero propio (conffile)**: chocaría con el que ya tenga puesto a
  mano quien siguió la web (`dpkg` se negaría a sobrescribir un fichero que no es suyo).

## Consecuencias

- Quien instala el `.deb` o el `.rpm` descargado queda al día con el resto del sistema, sin hacer nada más, y
  sigue pudiendo actualizar desde la ventana.
- El paquete toca `/etc` fuera de sus ficheros. Por eso la marca y lo de no pisar nada ajeno.
- El PKGBUILD de Arch reempaqueta el `.deb` sin ejecutar sus scripts: Arch no cambia.
- La web y las notas de la 1.1.3 tienen que contarlo (`docs/siguiente.md`). No se cambia la web antes, porque
  hasta la 1.1.3 los paquetes publicados no lo hacen.

## Verificación

- **Comprobado el 2026-11-27 en contenedores** con los scripts sueltos: en Debian 12, alta, repetición (no
  duplica), purga (lo quita) y fuente puesta a mano (no la toca ni la quita), y `apt-get update` lee el
  repositorio con la firma y ve la 1.1.2 como candidata. En Fedora 43, alta, actualización (`post 2` y
  `postun 1`: se queda), desinstalación (lo quita) y repositorio a mano (no lo toca), y `dnf info apolo` ve la
  1.1.2 con la clave de la web.
- **Con los paquetes compilados** (Publicar 45984107578, en verde con sus seis pruebas de humo): el `.deb`
  instalado con `apt` en Ubuntu 24.04 escribe la fuente y la clave, `apt update` lee el repositorio y
  `apt purge` lo deja todo como estaba; el `.rpm` con `dnf` en Fedora 43, igual, y `dnf remove` lo quita.
- **Actualizando desde la 1.1.2**, que es el caso de quien ya tiene Apolo: la 1.1.2 oficial de la Release (sin
  repositorio) actualizada con `apt-get install` a ese mismo `.deb` renumerado como 1.1.3 —lo que hace la
  ventana al actualizarse— deja la fuente puesta y `apt update` lee el repositorio. En el `.rpm`, el script
  con `$1 = 2` (actualización) se probó suelto, no con un paquete renumerado.
- **Lo vigila la CI** (`humo.yml`): tras instalar el `.deb` en Ubuntu, el `.sources` existe y `apt-get update` lee
  el repositorio; tras instalar el `.rpm` en Fedora, existe `apolo.repo`.
- **Comprobado por el cliente el 2026-11-27 con la 1.1.3 publicada**: en su Ubuntu y en su Fedora, actualizando
  desde la ventana, aparece la fuente (`apolo.sources`, `apolo.repo`) y `apt update` y `dnf check-update` leen el
  repositorio. Mac y Windows se actualizaron solos a la 1.1.3.
- **No comprobado**: openSUSE (`zypper`) más allá de la ruta, y que `apt upgrade` ofrezca de verdad una versión siguiente (se verá con la 1.1.4).
