<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Apolo">
</p>

<h1 align="center">Apolo</h1>

<p align="center">
  Optimize images on your desktop with Squoosh's engines.<br>
  macOS · Windows · Linux
</p>

<p align="center"><a href="README.md">Español</a></p>

<p align="center">
  <a href="https://github.com/webcafeina/Apolo/releases/latest"><img src="https://img.shields.io/github/v/release/webcafeina/Apolo?label=version" alt="Version"></a>
  <a href="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml"><img src="https://github.com/webcafeina/Apolo/actions/workflows/comprobar.yml/badge.svg" alt="Checks"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="GPLv3"></a>
</p>

---

## Downloads

Everything is in the **[latest release](https://github.com/webcafeina/Apolo/releases/latest)**. Find
your system in this table:

| System | App | Command line | Claude Desktop |
|---|---|---|---|
| **macOS 11+**, Apple Silicon and Intel | `Apolo_X.Y.Z_universal.dmg` | `apolo-cli-X.Y.Z-macos-universal.tar.gz` | `Apolo-X.Y.Z-macos.mcpb` |
| **Windows 10/11**, x64 | `Apolo_X.Y.Z_x64-setup.exe` | `apolo-cli-X.Y.Z-windows-x64.zip` | `Apolo-X.Y.Z-windows-x64.mcpb` |
| **Windows 10/11**, ARM64 | `Apolo_X.Y.Z_arm64-setup.exe` | `apolo-cli-X.Y.Z-windows-arm64.zip` | `Apolo-X.Y.Z-windows-arm64.mcpb` |
| **Debian 12+ / Ubuntu 22.04+**, amd64 | `Apolo_X.Y.Z_amd64.deb` or `.AppImage` | `apolo-cli-X.Y.Z-linux-amd64.tar.gz` | — |
| **Debian 12+ / Ubuntu 22.04+**, arm64 | `Apolo_X.Y.Z_arm64.deb` or `.AppImage` | `apolo-cli-X.Y.Z-linux-arm64.tar.gz` | — |

`SHA256SUMS.txt` has the checksum of every file.

**It updates itself**: when there's a new version, Apolo tells you and installs it with one click.

### The first time

The installers **aren't signed**, so each system warns you the first time:

- **macOS**: open Apolo and, in System Settings › Privacy & Security, click “Open Anyway”. If it says
  the app is damaged: `xattr -dr com.apple.quarantine /Applications/Apolo.app`.
- **Windows**: in SmartScreen, “More info” › “Run anyway”.
- **Linux**:
  - with the package: `sudo apt install ./Apolo_X.Y.Z_amd64.deb`;
  - with the AppImage: `chmod +x` and open it.

## What it is

Apolo compresses images to **WebP**, **AVIF**, **JPEG XL**, **MozJPEG**, **PNG** (OxiPNG) and
**QOI**, and **you see the result right away**. You compare the original and the compressed image
with a slider, see how much it weighs and where quality is lost, and adjust until you're happy.

- **The same file as the official tools**, byte for byte: `cwebp`, `cjpeg` (MozJPEG), `oxipng`,
  `avifenc` and `cjxl` are built into Apolo. You can use all their options while you watch, and copy
  the equivalent command.
- **Quality measured, not guessed**:
  - SSIMULACRA 2, PSNR and SSIM;
  - a map of where detail is lost;
  - **quality by score**: Apolo finds the lowest quality that can't be told apart from the
    original.
- **Studio and batches.** You fine-tune one image, save the setting as a preset and apply it to
  whole folders, with subfolders and without overwriting anything.
- **Resize, crop with the mouse and reduce the palette**, like in Squoosh.
- **Opens iPhone photos** (HEIC), as well as PNG, JPEG, WebP, AVIF, JPEG XL, TIFF, GIF, BMP, QOI and
  PNM.
- **`apolo` command line**, with the same presets, for scripts.
- **In English and Spanish**: the window, the command line and the agent. By default, the system's
  language; change it in Settings › General, with `apolo --idioma en`, or with `APOLO_IDIOMA=en` for
  the agent.
- **Native, light and offline**: your images never leave your computer. With 24-megapixel photos it
  stays around 2 GB of memory, even in batches of hundreds: it fits on an 8 GB Mac
  ([docs/rendimiento.md](docs/rendimiento.md), in Spanish).

## With an AI agent

Apolo can be used from **Claude, Cursor, VS Code** and any agent that speaks
[MCP](https://modelcontextprotocol.io). The agent analyzes, optimizes, compares formats at the same
perceived quality, measures the loss, crops, looks at images and converts folders, with Apolo's
engines and on your computer. **It never overwrites or deletes**: it always writes new files.

The server, `apolo-mcp`, comes with the app and **updates with it**. In Apolo, the **MCP** row in
the sidebar has everything you need: what to copy for each agent, and the Claude Desktop extension,
which you install once. After updating Apolo, quit and reopen Claude Desktop.

For Claude Code, depending on your system:

```sh
# macOS
claude mcp add --scope user apolo -- "/Applications/Apolo.app/Contents/MacOS/apolo-mcp"
# Windows
claude mcp add --scope user apolo -- "%LOCALAPPDATA%\Apolo\apolo-mcp.exe"
# Linux (.deb)
claude mcp add --scope user apolo -- /usr/bin/apolo-mcp
```

The tools are listed in [docs/mcp/catalogo.md](docs/mcp/catalogo.md) (in Spanish; the agent gets
them in its language). What the agent asks for (paths, names, thumbnails) goes into its
conversation, and the model provider sees it.

## Building

You need:
- stable Rust, Node 22 and pnpm;
- CMake, nasm and clang (libjxl is built with clang, like the official `cjxl`);
- on Linux, also `libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev pkg-config`.

Clone with submodules: `git clone --recurse-submodules`.

```sh
make comprobar     # formatting, clippy, tests and UI
make equivalencia  # the same file as cwebp, cjpeg, oxipng, qoiconv, avifenc and cjxl
make cli           # the command line
make mcp           # the agent server, apolo-mcp
make app           # the app (with apolo-mcp inside)
```

The code, its comments and the internal documentation (`docs/`) are in Spanish.

## License

[GPLv3](LICENSE). Apolo is free software: you can use, study, change and share it. The engines inside
have their own licenses, all compatible:

| License | Engines |
|---|---|
| BSD | libwebp, MozJPEG, libavif, aom, libyuv and libjxl |
| MIT | OxiPNG |
| GPLv3 | libimagequant |
| LGPLv3 | libheif and libde265 |
| Apache-2.0 | highway, sjpeg and rmcp |
| MIT or Apache-2.0 | sys-locale |

Made by [Webcafeína](https://webcafeina.com).
