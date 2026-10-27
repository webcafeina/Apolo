# El lanzamiento de la 1.0.0: directorios y anuncios

Lo que se envía y se publica fuera cuando sale la 1.0.0 con la web nueva (ADR 0034, S5). **Cada
envío, con el sí del cliente**: los que van con su cuenta los hace él, y en los demás se le pide
permiso antes de abrir el PR o enviar el formulario. Los textos van en inglés donde el público es
internacional.

## Solo, al publicar la versión

| Dónde | Cómo |
|---|---|
| Registro oficial de MCP | `.github/workflows/mcp.yml`, al publicar la Release: `io.github.webcafeina/apolo` con los cinco `.mcpb` (`empaquetado/mcp/servidor.py`). Validado con `mcp-publisher validate` el 2026-10-27 con los datos de la 0.18.3 |
| Glama, PulseMCP, Smithery, mcp.so | Recogen el registro oficial y GitHub solos. Comprobar a la semana que aparece, y reclamar la ficha si lo piden |
| Bing (y ChatGPT) | IndexNow en `web.yml` tras desplegar |
| Search Console y Bing Webmaster Tools | El cliente envía `https://apolo.webcafeina.com/sitemap.xml` en los dos. Bing ya está dado de alta (importado desde Search Console el 2026-10-27; tarda unas 48 horas en tener datos) |
| Homebrew, APT, RPM, PKGBUILD | Como en cada versión |

## Con un PR desde la cuenta del proyecto (con su sí)

**Estado el 2026-10-27**: enviados los dos, con el sí del cliente, desde forks en la cuenta
webcafeina: **awesome-mcp-servers#17812** (marcado con 🤖🤖🤖, como pide la lista a los agentes) y
**microsoft/winget-pkgs#463314** (`Webcafeina.Apolo` 1.0.0, x64 y ARM64, sumas comprobadas con
`SHA256SUMS.txt`). Pendientes de que los revisen.

### awesome-mcp-servers

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

Ficha de Apolo, y marcarla como alternativa a **Squoosh, TinyPNG, ImageOptim, Caesium, XnConvert y
FileOptimizer**. Etiquetas: image-optimizer, image-compression, webp, avif, jpeg-xl, open-source.

> **Apolo** — Free, open-source image optimizer for macOS, Windows and Linux. Compress and convert
> images to WebP, AVIF, JPEG XL, JPEG and PNG on your own computer, seeing what you lose before you
> save: a slider, side by side or a difference map, with each result's SSIMULACRA 2 score. It gives
> the same file, byte for byte, as cwebp, cjpeg, oxipng, avifenc and cjxl. Whole folders in parallel,
> a command line and an MCP server for AI agents. Opens iPhone HEIC.

### Product Hunt (el día del lanzamiento)

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
