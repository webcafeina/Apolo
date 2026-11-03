# Apolo for Claude

Optimize, convert and compare images on your own computer from Claude Code and Cowork, with
[Apolo](https://apolo.webcafeina.com/en/): WebP, AVIF, JPEG XL, JPEG and PNG (it also reads iPhone
HEIC), with the same engines and the same output as cwebp, cjpeg, oxipng, avifenc and cjxl, and the
quality loss measured with SSIMULACRA 2. It always writes new files: it never overwrites or deletes
your originals.

## What you need

- **The Apolo app**, free and open source (GPL-3.0), for macOS, Windows or Linux:
  https://apolo.webcafeina.com/en/. The MCP server, `apolo-mcp`, ships inside it and updates with it.
  On Linux, the `.deb`, the `.rpm` or the PKGBUILD (the AppImage doesn't leave it at a fixed path).
- **Node.js 18 or later** on your `PATH`: the plugin starts the server through a small launcher,
  `servidor/apolo-mcp.mjs`.

## Install

```
/plugin marketplace add webcafeina/Apolo
/plugin install apolo@webcafeina
```

Then restart Claude Code and ask, for example: *"Convert the photos in ~/Desktop/trip to AVIF for my
website, without visible loss"*.

## What's inside

- `.mcp.json` declares the `apolo` MCP server, started with `node servidor/apolo-mcp.mjs`.
- `servidor/apolo-mcp.mjs` looks for the `apolo-mcp` that the Apolo app installed (inside
  `Apolo.app` on macOS, `%LOCALAPPDATA%\Apolo` on Windows, `/usr/bin` on Linux, Homebrew, or your
  `PATH`; set `APOLO_MCP` for any other path) and hands the conversation over to it. It downloads and
  runs nothing else. If Apolo isn't installed, it answers as a minimal MCP server that explains how
  to install it.
- `skills/optimizar-imagenes` tells Claude when to use Apolo and how to get the best results.

## Privacy

Apolo works on your computer and sends your images nowhere. What the agent reads through it (paths,
file names, thumbnails it asks to see) goes into its conversation, and the model provider sees it.
Details: https://apolo.webcafeina.com/en/privacy/

## License and support

GPL-3.0-or-later, by [Webcafeína](https://webcafeina.com). Source and issues:
https://github.com/webcafeina/Apolo
