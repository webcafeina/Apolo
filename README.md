<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Apolo">
</p>

<h1 align="center">Apolo</h1>

<p align="center">
  Optimiza imágenes en tu escritorio con los motores de Squoosh.<br>
  macOS · Windows · Linux
</p>

<p align="center"><a href="https://apolo.webcafeina.com">apolo.webcafeina.com</a> · <a href="README.en.md">English</a></p>

<p align="center">
  <a href="https://github.com/webcafeina/Apolo/releases/latest"><img src="https://img.shields.io/github/v/release/webcafeina/Apolo?label=versi%C3%B3n" alt="Versión"></a>
  <a href="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml"><img src="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml/badge.svg" alt="Comprobar"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-GPLv3-blue" alt="GPLv3"></a>
</p>

---

## Descargas

Todo está en la **[última versión](https://github.com/webcafeina/Apolo/releases/latest)**. Busca tu
sistema en esta tabla:

| Sistema | Aplicación | Línea de órdenes | Claude Desktop |
|---|---|---|---|
| **macOS 11+**, Apple Silicon e Intel | `Apolo_X.Y.Z_universal.dmg` | `apolo-cli-X.Y.Z-macos-universal.tar.gz` | `Apolo-X.Y.Z-macos.mcpb` |
| **Windows 10/11**, x64 | `Apolo_X.Y.Z_x64-setup.exe` | `apolo-cli-X.Y.Z-windows-x64.zip` | `Apolo-X.Y.Z-windows-x64.mcpb` |
| **Windows 10/11**, ARM64 | `Apolo_X.Y.Z_arm64-setup.exe` | `apolo-cli-X.Y.Z-windows-arm64.zip` | `Apolo-X.Y.Z-windows-arm64.mcpb` |
| **Debian 12+ / Ubuntu 22.04+**, amd64 | `Apolo_X.Y.Z_amd64.deb` o `.AppImage` | `apolo-cli-X.Y.Z-linux-amd64.tar.gz` | — |
| **Debian 12+ / Ubuntu 22.04+**, arm64 | `Apolo_X.Y.Z_arm64.deb` o `.AppImage` | `apolo-cli-X.Y.Z-linux-arm64.tar.gz` | — |

`SHA256SUMS.txt` tiene la suma de cada fichero.

**Con Homebrew**, en macOS y Linux (con el nombre completo: así Homebrew confía en el tap):

```sh
brew install webcafeina/apolo/apolo          # la línea de órdenes y apolo-mcp
brew install --cask webcafeina/apolo/apolo   # la aplicación (macOS)
```

**Se actualiza sola**: cuando hay una versión nueva, Apolo lo avisa y se instala con un clic.

### La primera vez

Los instaladores **no están firmados**, así que cada sistema avisa la primera vez:

- **macOS**: abre Apolo y, en Ajustes del Sistema › Privacidad y seguridad, pulsa «Abrir
  igualmente». Si dice que está dañada: `xattr -dr com.apple.quarantine /Applications/Apolo.app`.
- **Windows**: en SmartScreen, «Más información» › «Ejecutar de todos modos».
- **Linux**:
  - con el paquete: `sudo apt install ./Apolo_X.Y.Z_amd64.deb`;
  - con la AppImage: `chmod +x` y ábrela.

## Qué es

Apolo comprime imágenes a **WebP**, **AVIF**, **JPEG XL**, **MozJPEG**, **PNG** (OxiPNG) y **QOI**,
y **el resultado se ve al momento**. Comparas el original y el comprimido con un deslizador, ves cuánto
pesa y dónde se pierde calidad, y ajustas hasta que te convence.

- **El mismo fichero que las herramientas oficiales**, byte a byte: `cwebp`, `cjpeg` (MozJPEG),
  `oxipng`, `avifenc` y `cjxl` van dentro de Apolo. Puedes usar todas sus opciones viéndolas, y
  copiar la orden equivalente.
- **Calidad medida, no a ojo**:
  - SSIMULACRA 2, PSNR y SSIM;
  - un mapa de dónde se pierde detalle;
  - **la calidad por nota**: Apolo busca la más baja que no se distingue del original.
- **Estudio y lotes.** Afinas una imagen, guardas el ajuste como preset y lo aplicas a carpetas
  enteras, con subcarpetas y sin sobrescribir nada.
- **Redimensionar, recortar con el ratón y reducir la paleta**, como en Squoosh.
- **Abre las fotos del iPhone** (HEIC), además de PNG, JPEG, WebP, AVIF, JPEG XL, TIFF, GIF, BMP,
  QOI y PNM.
- **Línea de órdenes `apolo`**, con los mismos presets, para scripts.
- **En español y en inglés**: la ventana, la línea de órdenes y el agente. Por defecto, el idioma
  del sistema; se cambia en Ajustes › General, con `apolo --idioma en` o con `APOLO_IDIOMA=en` para
  el agente.
- **Nativo, ligero y sin conexión**: tus imágenes no salen de tu ordenador. Con fotos de 24
  megapíxeles no pasa de unos 2 GB de memoria, ni en lotes de cientos: cabe en un Mac de 8 GB
  ([docs/rendimiento.md](docs/rendimiento.md)).

## Con un agente de IA

Apolo se puede usar desde **Claude, Cursor, VS Code** y cualquier agente que hable
[MCP](https://modelcontextprotocol.io). El agente analiza, optimiza, compara formatos a la misma
calidad percibida, mide la pérdida, recorta, mira las imágenes y convierte carpetas, con los motores
de Apolo y en tu ordenador. **Nunca sobrescribe ni borra**: siempre escribe ficheros nuevos.

El servidor, `apolo-mcp`, va dentro de la aplicación y **se actualiza con ella**. En Apolo, la fila
**MCP** de la barra lateral lleva a todo lo necesario: lo que hay que copiar para cada agente, y la
extensión de Claude Desktop, que se instala una vez. Después de actualizar Apolo, cierra y vuelve a
abrir Claude Desktop.

Para Claude Code, según el sistema:

```sh
# macOS
claude mcp add --scope user apolo -- "/Applications/Apolo.app/Contents/MacOS/apolo-mcp"
# Windows
claude mcp add --scope user apolo -- "%LOCALAPPDATA%\Apolo\apolo-mcp.exe"
# Linux (.deb)
claude mcp add --scope user apolo -- /usr/bin/apolo-mcp
```

Las herramientas están en [docs/mcp/catalogo.md](docs/mcp/catalogo.md). Lo que el agente pide (rutas,
nombres, miniaturas) entra en su conversación, y lo ve el proveedor del modelo.

## Compilar

Hace falta:
- Rust estable, Node 22 y pnpm;
- CMake, nasm y clang (libjxl se compila con clang, como el `cjxl` oficial);
- en Linux, además, `libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev pkg-config`.

Clona con submódulos: `git clone --recurse-submodules`.

```sh
make comprobar     # formato, clippy, pruebas e interfaz
make equivalencia  # el mismo fichero que cwebp, cjpeg, oxipng, qoiconv, avifenc y cjxl
make cli           # la línea de órdenes
make mcp           # el servidor para agentes, apolo-mcp
make app           # la aplicación (con apolo-mcp dentro)
```

## Licencia

[GPLv3](LICENSE). Apolo es software libre: puedes usarlo, estudiarlo, cambiarlo y compartirlo. Los
motores que lleva dentro tienen sus propias licencias, todas compatibles:

| Licencia | Motores |
|---|---|
| BSD | libwebp, MozJPEG, libavif, aom, libyuv y libjxl |
| MIT | OxiPNG |
| GPLv3 | libimagequant |
| LGPLv3 | libheif y libde265 |
| Apache-2.0 | highway, sjpeg y rmcp |
| MIT o Apache-2.0 | sys-locale |

Hecho por [Webcafeína](https://webcafeina.com).
