<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Apolo">
</p>

<h1 align="center">Apolo</h1>

<p align="center">
  Optimiza imágenes en tu escritorio con los motores de Squoosh.<br>
  macOS · Windows · Linux
</p>

<p align="center">
  <a href="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml"><img src="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml/badge.svg" alt="Comprobar"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-GPLv3-blue" alt="GPLv3"></a>
</p>

---

> **En construcción.** Todavía no hay versión publicada. El plan, por entregas, está en
> [docs/siguiente.md](docs/siguiente.md).

## Qué es

Apolo comprime imágenes a **WebP**, **AVIF**, **JPEG XL**, **MozJPEG**, **PNG** (OxiPNG) y **QOI**,
y el resultado se ve **al momento**: comparas el original y el comprimido con un deslizador, ves cuánto
pesa y dónde se pierde calidad, y ajustas hasta que te convence.

- **El mismo motor que `cwebp`.** libwebp va dentro de Apolo, y con las mismas opciones da **el mismo
  fichero, byte a byte**, que el `cwebp` oficial. Puedes usar todas sus opciones viéndolas, y copiar la
  orden `cwebp` equivalente.
- **Estudio y lotes.** Afinas una imagen y guardas el ajuste como preset; luego lo aplicas a carpetas
  enteras.
- **Redimensionar, recortar y reducir paleta**, como en Squoosh.
- **Mapa de diferencias** con PSNR y SSIM.
- **Línea de comandos `apolo`**, con los mismos presets, para scripts.
- Nativo, ligero y sin conexión: tus imágenes no salen de tu ordenador.

## Plataformas

| Sistema | Mínimo | Arquitecturas |
|---|---|---|
| macOS | 11 Big Sur | Apple Silicon e Intel |
| Windows | 10 | x64 y ARM64 |
| Linux | Debian 12 / Ubuntu 22.04 | amd64 y arm64 |

Los instaladores todavía **no están firmados**. La primera vez, macOS dirá que el desarrollador no
está identificado (clic derecho → Abrir) y Windows enseñará SmartScreen (Más información → Ejecutar
de todos modos).

## Compilar

Hace falta Rust (estable), Node 22 y pnpm. En Linux, además,
`libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev pkg-config`.

```sh
make comprobar   # pruebas
make cli         # la línea de comandos
make app         # la aplicación
```

## Licencia

[GPLv3](LICENSE). Apolo es software libre: puedes usarlo, estudiarlo, cambiarlo y compartirlo.
Los motores que lleva dentro tienen sus propias licencias, todas compatibles: libwebp, MozJPEG,
libavif y libjxl (BSD), OxiPNG (MIT) y libimagequant (GPLv3).

Hecho por [Webcafeína](https://webcafeina.com).
