# El lanzamiento de la 1.0.0: directorios y anuncios

Lo que se envía y se publica fuera cuando sale la 1.0.0 con la web nueva (ADR 0034, S5). **Cada
envío, con el sí del cliente**: los que van con su cuenta los hace él, y en los demás se le pide
permiso antes de abrir el PR o enviar el formulario. Los textos van en inglés donde el público es
internacional.

## Solo, al publicar la versión

| Dónde | Cómo |
|---|---|
| Registro oficial de MCP | `.github/workflows/mcp.yml`, al publicar la Release: `io.github.webcafeina/apolo` con los cinco `.mcpb` (`empaquetado/mcp/servidor.py`). Validado con `mcp-publisher validate` el 2026-10-27 con los datos de la 0.18.3 |
| Directorios de MCP | **Corregido el 2026-10-27** (`docs/distribucion.md`): no todos recogen solos. PulseMCP y mcp.directory, del registro oficial (PulseMCP, cuando reabra los envíos); Glama hay que reclamarlo y darle un Dockerfile (lo pide también awesome-mcp-servers); Smithery, con `smithery mcp publish`; mcp.so y mcpservers.org, con su formulario; el GitHub MCP Registry, por correo |
| Bing (y ChatGPT) | IndexNow en `web.yml` tras desplegar |
| Search Console y Bing Webmaster Tools | El cliente envía `https://apolo.webcafeina.com/sitemap.xml` en los dos. Bing ya está dado de alta (importado desde Search Console el 2026-10-27; tarda unas 48 horas en tener datos) |
| Homebrew, APT, RPM, PKGBUILD | Como en cada versión |

## Con un PR desde la cuenta del proyecto (con su sí)

**Estado el 2026-10-27**: enviados los dos, con el sí del cliente, desde forks en la cuenta
webcafeina: **awesome-mcp-servers#17812** (marcado con 🤖🤖🤖, como pide la lista a los agentes) y
**microsoft/winget-pkgs#463314** (`Webcafeina.Apolo` 1.0.0, x64 y ARM64, sumas comprobadas con
`SHA256SUMS.txt`). Pendientes de que los revisen.

### Uptodown (2026-10-30)

Ficha de **Windows** enviada a revisión por el cliente en https://www.uptodown.dev (cuenta
info@webcafeina.com, organización Webcafeína), en español: descripción corta «Optimiza imágenes a
WebP, AVIF y JPEG XL sin perder calidad visible» (máximo 70 caracteres), la larga de la web, icono
de 256 px, imagen destacada de 1024×500 (la social en español recortada), capturas en español,
licencia con la URL de `LICENSE`, código fuente, sin IAP ni anuncios. Fichero: el
`Apolo_1.0.0_x64-setup.exe` (SHA-256 comprobada contra `SHA256SUMS.txt`), versión mínima Windows 10.
Los dos interruptores apagados: **los editores de Uptodown pueden subir las versiones nuevas** (no
hay forma automática de subirlas nosotros) y se pueden bajar las anteriores.

Ficha de **Mac** enviada también el mismo día: los mismos textos, el `Apolo_1.0.0_universal.dmg`
(SHA-256 `75c9490b…`) y la versión mínima macOS 11. **Pendiente de mirar**: al editar la de Windows
en revisión, el cliente vio los campos vacíos. Puede ser el borrador en revisión o que algo no se
guardara (cada pestaña tiene su botón de guardar); se decidió esperar a la revisión antes de tocar
nada.

### Directorios de software (2026-10-30)

- **OpenAlternative**: rechaza el envío con menos de **50 estrellas** («Please come back once the
  project has gained more traction»). Se vuelve cuando las tenga, con awesome-rust y Scoop Extras.
- **SaaSHub**: enviado por el cliente el 2026-10-30 (cuenta info@webcafeina.com): la descripción larga
  en inglés, 15 pares de «Feature name / Value» (formatos de entrada y salida sacados del núcleo,
  motores, comparación, nota, lotes, CLI, MCP, privacidad, plataformas…), las preguntas de «qué la
  hace única», público, historia y tecnologías. Sin página de estado. «Biggest customers» se dejó
  vacío a propósito: no hay datos de quién lo usa y no se inventan.
- **Softpedia**: enviado por el cliente el 2026-10-30 desde softpedia.com/user/submit.shtml (bloquea
  las consultas automáticas, hay que abrirla en un navegador). Solo deja elegir Windows o Unix (Linux
  va en «Unix»; Mac no está). Pide enlaces **directos** (los de la Release), tamaño, icono de 32 px
  (el `favicon.ico` de la web lleva 16 y 32), una captura por URL, requisitos (WebView2 en Windows,
  WebKitGTK 4.1 en Linux), limitaciones y cambios.
- **MajorGeeks** (solo Windows): no tiene formulario; los envíos van por correo a
  mgnews@majorgeeks.com (su página «About / Contact»). Enviado por el cliente el 2026-10-30. Prueban
  cada programa en una máquina virtual y avisan de que la mayoría no pasa.
- **MacUpdate**: enviado por el cliente el 2026-10-30, con su cuenta (justo después de registrarse,
  su CloudFront dio un 403 pasajero). El `.dmg` universal, soporte en los issues de GitHub, el aviso
  de «Abrir igualmente» en los requisitos, el icono de 512 px y cinco capturas de macOS en inglés
  (Estudio claro y oscuro de `.github/readme/`, y Diferencias, Lotes y Agentes de la web a 2560 px,
  con la transparencia de la barra lateral rellena de `#eae7ee`).
- **LibHunt**: no tiene formulario de envío; recoge solo los repositorios de las listas awesome y de
  GitHub.

### Chocolatey (2026-10-30)

Paquete `apolo` en `empaquetado/chocolatey/`: `apolo.nuspec` y `tools/chocolatey{install,uninstall}.ps1`
como plantillas, que rellena `rellenar.py <versión> <SHA256SUMS.txt> <salida>`. Instala en silencio
(`/S`) el NSIS x64 o el ARM64 según `Win32_Processor.Architecture` (12 es ARM64; Chocolatey no tiene
parámetro para ARM y va como `url64bit`), con su SHA-256; desinstala con el desinstalador de NSIS
que encuentra `Get-UninstallRegistryKey`. Se empaqueta y se sube **sin Windows**, con la imagen
oficial `chocolatey/choco:latest-linux` (Apache-2.0):

```sh
python3 -I empaquetado/chocolatey/rellenar.py 1.0.0 SHA256SUMS.txt paquete
docker run --rm -v "$PWD/paquete:/paquete" -w /paquete chocolatey/choco:latest-linux choco pack apolo.nuspec --out /paquete
docker run --rm -v "$PWD/paquete:/paquete:ro" -v "$HOME/.config/apolo/chocolatey.key:/clave:ro" -w /paquete chocolatey/choco:latest-linux sh -c 'choco push apolo.1.0.0.nupkg --source https://push.chocolatey.org/ --api-key "$(cat /clave)"'
```

La cuenta es `webcafeina` (info@webcafeina.com) y la clave de API está en
`~/.config/apolo/chocolatey.key` (600; la guardó el cliente por SSH, **no se escribe en ningún
sitio**). La 1.0.0 se subió el 2026-10-30: https://community.chocolatey.org/packages/apolo/1.0.0,
en moderación. Revisan cada versión a mano (días o semanas). Sin probar en un Windows antes de
subirla: lo prueba su verificador en x64; la rama de ARM64, sin ver.

Lo que dice su correo de envío: **ningún moderador lo ve hasta que pasan las comprobaciones
automáticas** (validación, verificación, análisis) o el mantenedor responde en la caja **«Add to
Review Comments»** de la página del paquete (los comentarios de Disqus no llegan). Si algo falla,
**se vuelve a subir la misma versión** con el arreglo, mientras no esté aprobada. Sin noticias en una
o dos semanas, preguntar en esa caja. Dudas: chocolatey-moderators@googlegroups.com.

**La 1.0.1 (2026-10-30) no entra mientras la 1.0.0 espere**: `choco push` respondió `403
(Forbidden)` con la misma clave que subió la 1.0.0. Decidido con el cliente: esperar a que aprueben
la 1.0.0 y subir entonces la 1.0.1 (se rellena y empaqueta igual, con las sumas de su
`SHA256SUMS.txt`). La otra vía era retirar la 1.0.0 desde la web («self-reject») y volver al final
de la cola.

### Prensa (2026-10-30)

Nota de prensa en español enviada por el cliente a **contacto@genbeta.com** y
**contacto@xataka.com** (los buzones generales de sus páginas «Quiénes somos»), un correo a cada
medio. Contenido: qué es, qué lo hace distinto (la pérdida a la vista, los motores oficiales, las
cifras de Kodak, lotes, MCP), disponibilidad y enlaces, con la imagen social en español como
material de prensa.

### Directorios de MCP (2026-10-30)

- **mcp.so**: su formulario solo publica pagando (39 USD); la vía gratuita es un issue en
  `chatmcp/mcpso` titulado «Submit MCP Server: …». Abierto **chatmcp/mcpso#6439**, con el sí del
  cliente. Los cierran al procesarlos, en uno o dos días.
- **mcpservers.org**: enviado por el cliente con el plan gratuito (revisión en hasta dos semanas),
  categoría Design (no hay de imagen), con el nombre del registro `io.github.webcafeina/apolo`.
- **GitHub MCP Registry**: correo a partnerships@github.com enviado por el cliente el 2026-10-30
  (asunto «MCP Registry inclusion request: io.github.webcafeina/apolo»). Según el blog de GitHub,
  solo recoge solo las actualizaciones de los servidores que ya tiene, no los nuevos.

### Listas de aplicaciones (2026-10-30)

- **open-source-mac-os-apps#1878**: la entrada al final de `applications.json` (categorías `images`
  y `graphics`, icono de la web y dos capturas de macOS de `.github/readme/`). Fork en
  `webcafeina/open-source-mac-os-apps`.
- **awesome-mac#4070**: Design and Product › Other Tools, en los cuatro README (en, zh, ja y ko, que
  la lista exige sincronizados), en su orden alfabético. Fork en `webcafeina/awesome-mac`.
- **Awesome-Linux-Software**: descartada, el repositorio está archivado desde mayo de 2026.

### awesome-mcp-servers

**Glama (lo pide la etiqueta `missing-glama` del PR)**: **hecho el 2026-10-30**. El cliente envió
Apolo con «Add Server» (el botón está en la barra de ordenar, encima de la lista de
glama.ai/mcp/servers); lo aprobaron en el día, en `https://glama.ai/mcp/servers/webcafeina/Apolo`.
Los campos de Admin › Dockerfile (abajo, con la base `debian:trixie-slim`, la única que ofrece), la
prueba en verde, «Make Release» **desde esa prueba** en la página del Dockerfile (la pestaña Releases
solo lista) y nota **A**. Con la 1.0.1 (2026-10-30), los pasos pasaron a **bajar siempre la última
versión** (`releases/latest/download`) y comprobarla con el `SHA256SUMS.txt` de esa misma versión, así
que **Auto-Release** pone a Glama al día sola en cada versión. Hecho por el cliente el mismo día:
pasos nuevos, prueba en verde y la release **0.1.1** (la numeración es de Glama, no de Apolo; la
primera fue la 0.1.0). Auto-Release ya estaba encendido sin tocarlo: venía así, y solo esperaba a
una compilación válida. La ficha, con las 11 herramientas y nota A en licencia, mantenimiento y
calidad. La suma ya no va fija: protege de una descarga
rota, no de unos ficheros cambiados en la Release. Añadida al PR la insignia (commit `92dcc68`)
`[![webcafeina/Apolo MCP server](https://glama.ai/mcp/servers/webcafeina/Apolo/badges/score.svg)](https://glama.ai/mcp/servers/webcafeina/Apolo)`.
**Trampa**: la página `…/admin/dockerfile` de Glama **no admite un Dockerfile pegado**; tiene campos
y lo genera ella: `debian:bookworm-slim` con node y `mcp-proxy`, el repositorio clonado en `/app`,
cada paso como un `RUN` de root y `CMD ["mcp-proxy", …argumentos]`. La ficha quedó reclamada sola
(`author:claimed`) al darla de alta con la cuenta dueña del repositorio. Los valores, probados el
2026-10-30 con esa misma base (`initialize` + `tools/list`, 11 herramientas), y el paso de la última
versión también en una Debian trixie limpia (instala `apolo-mcp 1.0.1`). Sin comillas dobles ni
barras invertidas, para pegarlo en el JSON sin escapar nada:

- Build steps: `["set -e; cd /tmp; base=https://github.com/webcafeina/Apolo/releases/latest/download; curl -fsSL -o SHA256SUMS.txt $base/SHA256SUMS.txt; grep -E '  apolo-cli-[0-9.]+-linux-amd64[.]tar[.]gz$' SHA256SUMS.txt > suma.txt; test $(wc -l < suma.txt) -eq 1; f=$(awk '{print $2}' suma.txt); curl -fsSL -o $f $base/$f; sha256sum -c suma.txt; tar -xzf $f -C /usr/local/bin apolo apolo-mcp; rm -f $f SHA256SUMS.txt suma.txt"]`
  (antes, con la 1.0.0: tres pasos con la URL y la suma fijas)
- CMD arguments: `["/usr/local/bin/apolo-mcp"]`
- Variables de entorno: `{"type":"object","properties":{},"required":[]}`; parámetros de ejemplo, `{}`.

El Dockerfile equivalente, para probarlo en local o en otro directorio que sí lo admita:

```dockerfile
FROM debian:bookworm-slim
ARG APOLO_VERSION=1.0.0
ARG APOLO_SHA256=f92e86f1dfd81d7cb8f83ecc0097917239f16616173eccf4f970e26f5e8c23f4
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl libstdc++6 \
 && curl -fsSL -o /tmp/apolo.tar.gz \
    "https://github.com/webcafeina/Apolo/releases/download/v${APOLO_VERSION}/apolo-cli-${APOLO_VERSION}-linux-amd64.tar.gz" \
 && echo "${APOLO_SHA256}  /tmp/apolo.tar.gz" | sha256sum -c - \
 && tar -xzf /tmp/apolo.tar.gz -C /usr/local/bin apolo apolo-mcp \
 && rm /tmp/apolo.tar.gz \
 && apt-get purge -y curl && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home apolo
USER apolo
WORKDIR /home/apolo
ENTRYPOINT ["apolo-mcp"]
```

`punkpeye/awesome-mcp-servers`, sección **🎥 Multimedia Process**, en orden alfabético:

```markdown
- [webcafeina/Apolo](https://github.com/webcafeina/Apolo) 🦀 🏠 🍎 🪟 🐧 - Optimize, convert and compare images locally with the same engines as cwebp, cjpeg, oxipng, avifenc and cjxl: WebP, AVIF, JPEG XL, JPEG and PNG, quality by SSIMULACRA 2 score, whole folders, never overwrites. Desktop app, CLI and MCP server.
```

### winget

Un PR a `microsoft/winget-pkgs` con `Webcafeina.Apolo` (instalador NSIS, x64 y ARM64). Se genera
con **komac** (GPL-3.0, en Rust, funciona en Linux) a partir de las URL de la Release. Dos
trampas: pregunta cosas por pantalla aunque se le den todas las opciones (se lanza con `script` para
darle un terminal y se aceptan las respuestas por defecto), y el fork `webcafeina/winget-pkgs` hay
que crearlo antes (`gh repo fork microsoft/winget-pkgs --clone=false --default-branch-only`):

```sh
komac new Webcafeina.Apolo --version 1.0.0 \
  --urls https://github.com/webcafeina/Apolo/releases/download/v1.0.0/Apolo_1.0.0_x64-setup.exe \
         https://github.com/webcafeina/Apolo/releases/download/v1.0.0/Apolo_1.0.0_arm64-setup.exe
```

Publisher «Webcafeína», licencia GPL-3.0, descripción corta: «Lighter images, just as good: optimize
and convert images to WebP, AVIF, JPEG XL, JPEG and PNG». Sin firma de Microsoft se acepta, pero la
validación de winget pasa los instaladores por Defender: puede tardar. Después, cada versión con
`komac update` (o un workflow, cuando la 1.0 esté dentro).

## Con la cuenta del cliente

### Directorio de extensiones de Claude Desktop

**Cambiado (2026-10-27)**: según la documentación de Anthropic, el directorio ya no acepta `.mcpb`
sueltos; un servidor local entra solo dentro de un *plugin* de Claude. Las herramientas ya llevan
`title` y `readOnlyHint`/`destructiveHint`; faltarían la política de privacidad (sección en el README
y `privacy_policies` en el manifiesto) y el *plugin*. Trabajo aparte, a decidir con el cliente.
Lo de abajo era el plan anterior.

El formulario de Anthropic para el directorio de extensiones (`.mcpb`). Datos: el `.mcpb` de cada
sistema de la Release, la web de agentes como documentación, `SECURITY.md` y la política de
privacidad de webcafeina.com. Resumen:

> Apolo optimizes images on your computer: analyze, convert to WebP, AVIF, JPEG XL, JPEG or PNG,
> compare formats at the same perceived quality (SSIMULACRA 2), measure the loss, crop, resize and
> convert whole folders. It always writes new files and never overwrites or deletes. Free and open
> source (GPL-3.0).

### AlternativeTo

**Enviada a revisión el 2026-10-30** por el cliente, con su cuenta (info@webcafeina.com): la
descripción larga de abajo, la corta («Free, open-source image optimizer: compress and convert to
WebP, AVIF, JPEG XL, JPEG and PNG locally, and see the loss.»), licencia GPL-3.0, gratis, logo de
512 px y las cinco capturas en inglés. Sin vídeo, porque solo admite enlaces de YouTube. Casillas
marcadas: sin registro, sin rastreo, no destructiva, privacidad, sin anuncios, sin conexión, lotes,
modo oscuro, vista previa y CLI. **Pendiente**:
- cuando la aprueben, «Suggest alternative» en cada una de las seis de abajo;
- **los gestores de paquetes se dejaron vacíos** porque comprueba contra el catálogo oficial: cuando
  winget acepte el PR, editar la ficha y poner `Webcafeina.Apolo`. El tap de Homebrew no cuenta.

Ficha de Apolo, y marcarla como alternativa a **Squoosh, TinyPNG, ImageOptim, Caesium, XnConvert y
FileOptimizer**. Etiquetas: image-optimizer, image-compression, webp, avif, jpeg-xl, open-source.

> **Apolo** — Free, open-source image optimizer for macOS, Windows and Linux. Compress and convert
> images to WebP, AVIF, JPEG XL, JPEG and PNG on your own computer, seeing what you lose before you
> save: a slider, side by side or a difference map, with each result's SSIMULACRA 2 score. It gives
> the same file, byte for byte, as cwebp, cjpeg, oxipng, avifenc and cjxl. Whole folders in parallel,
> a command line and an MCP server for AI agents. Opens iPhone HEIC.

### Product Hunt (el día del lanzamiento)

**Programado el 2026-10-30 para el martes 3 de noviembre de 2026** (00:01 de California, 9:01 en
España), con la cuenta personal del cliente como *maker*. Con los textos de abajo, el logo de
240 px y una galería de cinco capturas a 1270×760 (fondo `#393941`), sin vídeo. Ese día: contestar
los comentarios desde la mañana y difundir el enlace; Reddit y Show HN siguen descartados por ahora.

- **Nombre**: Apolo
- **Tagline** (60 caracteres como mucho): `Lighter images, just as good — free, local, open source`
- **Descripción** (260 como mucho): `Compress and convert images to WebP, AVIF and JPEG XL on your Mac, Windows or Linux PC. See what you lose before you save, ask for a quality score instead of guessing, batch whole folders, or let your AI agent do it. Free and open source.`
- **Imágenes**: la social (1200×630), capturas de la web (`sitio/public/img/`) y el vídeo de la portada.
- **Primer comentario del autor**:

> Hi! We built Apolo because we wanted a desktop tool that compresses images with the official
> engines (libwebp, MozJPEG, OxiPNG, libavif, libjxl) and *shows* you the loss instead of making you
> guess a quality number. You ask for "no visible loss" (a SSIMULACRA 2 score) and it finds the
> lowest quality that gets there, photo by photo. In our tests on 24 Kodak photos, at the same
> perceived quality AVIF weighs 25% less than JPEG and JPEG XL up to 41% less. It's free, GPL-3.0,
> runs on macOS, Windows and Linux (x64 and ARM), and ships an MCP server so Claude or Cursor can
> optimize images for you. Feedback very welcome.

### Show HN

- **Título**: `Show HN: Apolo – Image optimizer that measures the loss (WebP, AVIF, JPEG XL)`
- **URL**: https://apolo.webcafeina.com/en/
- **Texto**: el primer comentario de Product Hunt, más un párrafo técnico: Tauri 2 y Rust, los
  códecs compilados dentro y llamados por su `main` (por eso el mismo fichero byte a byte que las
  herramientas oficiales, comprobado en CI con `make equivalencia`), y SSIMULACRA 2 por canal.

### Reddit

Cada comunidad tiene sus normas de autopromoción: leerlas antes y publicar como persona, no como
marca. Candidatas: r/webdev (la guía de optimizar imágenes para la web), r/macapps, r/linux (con los
repositorios APT y RPM), r/opensource y r/ClaudeAI (el servidor MCP). Texto base: el del primer
comentario de Product Hunt, adaptado a cada una.

## Descartados

- **awesome-tauri**: ya solo recoge plugins, plantillas y guías, no aplicaciones.
- **homebrew-core**: pide notoriedad (estrellas y forks) que el repositorio aún no tiene. Se sigue
  con el tap propio y se vuelve a mirar más adelante.
