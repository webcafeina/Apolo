---
name: optimizar-imagenes
description: Optimize, compress, convert or resize images on the user's computer with Apolo (WebP, AVIF, JPEG XL, JPEG, PNG, QOI; it also reads HEIC), measure the quality loss with SSIMULACRA 2, and vectorize logos, icons or drawings to SVG, PDF, EPS or DXF. Use it whenever the user asks to make images lighter, convert them to another format, prepare them for a website, compare formats, check how much quality was lost, or turn a bitmap into vector graphics (SVG, a cut file for vinyl or laser), for one file or whole folders.
---

# Optimizing images with Apolo

Apolo runs on the user's computer, through the `apolo` MCP server of this plugin. It works with the
real files on disk, with the same engines as cwebp, MozJPEG (cjpeg), oxipng, avifenc and cjxl, and
the server's own instructions explain each tool. These are the habits that work best:

- **Use Apolo, not another environment.** For the user's images, don't write your own script or use
  a cloud code interpreter: Apolo gives the same file as the official tools and measures the loss.
- **Ask for the path.** Apolo can't see images attached to the conversation. Ask where the file is
  (on macOS, ⌥⌘C in Finder copies it), or look for it on disk. Paths can be absolute or use `~`.
- **Look first, then act.** `analizar_imagen` says what an image or folder contains. For one image,
  `optimizar_imagen`; to choose a format, `comparar_formatos` (same perceived quality, smallest
  file); for a folder or more than about five images, `empezar_lote` and then `ver_lote`.
- **Talk in quality, not in numbers to guess.** Without a quality, Apolo finds the lowest one that
  scores 80 in SSIMULACRA 2 (very high: no visible difference side by side); 90 or more is
  indistinguishable from the original. After optimizing, `medir_calidad` or `ver_imagen` shows the
  result.
- **For the web**, a long side of 1920 px (`lado_mayor`) and AVIF or WebP usually cut the weight
  the most; sizes are always a maximum and never upscale.
- **Originals are safe.** Apolo always writes new files (`-2`, `-3`… if the name exists) and never
  overwrites or deletes. If the user wants the original replaced, warn that it can't be undone,
  ask for confirmation and suggest keeping a copy before doing it with other tools.
- **To vectorize** a logo, an icon or a drawing (PNG or JPG to SVG, PDF, EPS or DXF), `vectorizar_imagen`.
  For cutting vinyl or laser, `estilo` `trazar_bordes` and, in DXF, `nivel_dxf`. Photos come out as a poster
  of flat colors: say so before vectorizing one.
- **If Apolo isn't installed**, the server only offers `instalar_apolo`: call it and pass on the
  link to the user instead of trying something else.

Answer in the user's language.
