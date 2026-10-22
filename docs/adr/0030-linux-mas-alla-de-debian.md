# ADR 0030 — Linux más allá de Debian: RPM, AUR y repositorios APT y RPM

**Fecha:** 2026-10-22 · **Estado:** aceptada · v0.16.0 · matiza la [0010](0010-plataformas.md) y la [0018](0018-actualizarse-sola.md)

## Contexto

Hasta la v0.15.0, Apolo en Linux era un `.deb` (Debian 12+ y Ubuntu 22.04+) y una AppImage, amd64 y
arm64. En Fedora, openSUSE o Arch solo quedaba la AppImage, que no se integra con el sistema. El
cliente pidió que se instalara en las demás distribuciones como hace Warp (deb, rpm, pacman y
repositorios propios), y que se planteara adaptar la interfaz a KDE (eso es la ADR 0031, v0.17.0).

Investigado el 2026-10-22 con dos agentes:
- Tauri 2.12 genera `deb`, `rpm` y `appimage`; Flatpak y Snap no, se hacen aparte.
- `tauri-plugin-updater` (2.13.1 en Apolo) actualiza AppImage, deb y **rpm** (desde la 2.10): lee
  el tipo de paquete grabado en el binario y busca `{os}-{arch}-{tipo}` en `latest.json`; si no
  está, cae en `{os}-{arch}` (la AppImage) y con un rpm fallaría.
- Las librerías de Apolo (libheif, aom, libjxl, libwebp, mozjpeg…) van estáticas: del sistema solo
  usa WebKitGTK 4.1, GTK3 y glibc (2.35, de Ubuntu 22.04). El mismo binario vale para Fedora y Arch.
- Flathub exige compilar desde el código sin red (con Rust, pnpm y los submódulos C con clang) y
  pasar su revisión; el Snap Store, un nombre y su confinamiento.

## Decisión

**Decidido con el cliente** (2026-10-22):

| Tema | Decisión |
|---|---|
| Formatos | **RPM** (Fedora, openSUSE), **AUR** (`apolo-bin`, para Arch y sus derivadas) y **repositorios APT y RPM propios**. Sin Flatpak ni Snap por ahora |
| Repositorios | En **apolo.webcafeina.com** (`/apt` y `/rpm`), dentro del despliegue de Pages: solo la última versión, sin guardar paquetes en git, sin DNS nuevo ni coste |
| Firma | Una clave GPG propia (RSA 4096, sin caducidad) para el rpm y los dos repositorios |
| AUR | La cuenta es del cliente; la clave SSH, de CI |

- **RPM**: `rpm` en `bundle.targets`; en `tauri.linux.conf.json`, el mismo lanzador y el mismo
  `metainfo.xml` que el deb, `release` 1 y las dependencias con sus nombres de Fedora
  (`webkit2gtk4.1`, `gtk3`). Firmado por Tauri con `TAURI_SIGNING_RPM_KEY` (el secreto
  `PAQUETES_GPG_CLAVE`). `adjuntar` exige `linux-{x86_64,aarch64}-rpm` en `latest.json` (16
  entradas): así el rpm se actualiza solo desde la banda, con `rpm -U` y `pkexec`.
- **Repositorios** (`web.yml`, que ahora se lanza también al publicar una versión): baja los `.deb`
  y `.rpm` de la última Release y monta `web/apt` (`apt-ftparchive`, `InRelease` y `Release.gpg`
  firmados, `apolo.sources` en deb822 con `Signed-By: /etc/apt/keyrings/apolo.asc`) y `web/rpm`
  (`createrepo_c`, `repomd.xml.asc`, `apolo.repo` con `gpgcheck` y `repo_gpgcheck`). La clave
  pública, en `web/apolo.asc` y `empaquetado/linux/apolo.asc`. El entorno `github-pages` admite
  ahora también las etiquetas `v*` (trampas.md).
- **AUR** (`empaquetado/aur/PKGBUILD`, `rellenar.py`, `aur.yml`): `apolo-bin` extrae el `.deb` de
  la Release (x86_64 y aarch64) y depende de `webkit2gtk-4.1` y `gtk3`. Al publicar, `aur.yml`
  lo rellena con la versión y las sumas, comprueba que se construye y lo sube con
  `KSXGitHub/github-actions-deploy-aur` (MIT, fijada por su commit) y el secreto `AUR_CLAVE`.
- **El actualizador en Arch**: el binario del `.deb` se cree deb y haría `dpkg -i`. El PKGBUILD deja
  `pacman` en `/usr/share/apolo/gestor`; `plataforma` lo devuelve (`gestor`) y la banda de versión
  nueva dice «Apolo se actualiza con tu gestor de paquetes (pacman)», sin «Descargar». Con los
  repositorios APT y RPM, el actualizador de Apolo sigue: es el mismo paquete.
- **Humo** (`humo.yml`): Fedora en amd64 y arm64 (`rpm --checksig`, `dnf install`, ninguna
  librería sin resolver) y Arch en amd64 (el PKGBUILD con el `.deb` de la ejecución, `makepkg` y
  `pacman -U`).
- **La web y el README**: las filas de Linux con `.deb · .rpm · AppImage`, y las órdenes de APT,
  dnf, zypper y AUR. En Linux, el botón principal lleva a esas órdenes: desde el navegador no se
  sabe la distribución.
- **Claves**: la privada GPG, su frase (en hexadecimal) y la SSH del AUR están en
  `~/.config/apolo/claves/` (`paquetes.*`, `gpg/`, `aur.key`) y en los secretos `PAQUETES_GPG_CLAVE`,
  `PAQUETES_GPG_FRASE` y `AUR_CLAVE`. **No se borran ni se escriben nunca en el chat.**

## Alternativas descartadas

- **Flatpak en Flathub**: la tienda universal y la única que enseña icono y editor en cualquier
  Centro de aplicaciones, pero exige compilar todo desde el código sin red y su revisión, y se
  actualiza por Flathub y no por Apolo. Queda en la deuda.
- **Snap**: confinamiento estricto y una tienda más; poco uso fuera de Ubuntu.
- **Que el paquete añada el repositorio solo** (como Chrome o Warp): cómodo, pero toca la
  configuración del sistema sin preguntar. Más adelante, si se pide.
- **Repositorios en otro sitio** (un subdominio o Cloudflare R2): DNS nuevo o coste; con solo la
  última versión, Pages basta (~100 MB).
- **Los nombres de dependencias de openSUSE** con dependencias «ricas» (`(a or b)`): no está claro
  que el rpm de Tauri las escriba bien. Se queda con los de Fedora (deuda).

## Consecuencias

- Cuatro formatos en Linux y tres maneras de actualizarse: la banda (deb, rpm, AppImage), `apt` o
  `dnf` con los repositorios, y `pacman`/`yay` con el AUR.
- Una clave GPG más que custodiar: si se pierde, quien use los repositorios tiene que importar la
  nueva.
- El rpm se compila en Ubuntu 22.04 (glibc 2.35): Fedora 36+ y openSUSE Leap 16, sí; RHEL 9, no.
- `latest.json` pasa de 14 a 16 entradas, y la revisión del borrador lo mira.

## Verificación

- `make comprobar` y e2e (la banda con `?gestor=pacman`) en verde.
- **No verificado** al escribir esto: el rpm firmado y su entrada en `latest.json`, el humo de
  Fedora y Arch, los repositorios desplegados y el AUR (todo en CI, con la compilación de prueba y
  la v0.16.0); la prueba guiada en las máquinas virtuales Fedora, Ubuntu (repositorio APT) y Arch
  Linux ARM. openSUSE, sin máquina.
