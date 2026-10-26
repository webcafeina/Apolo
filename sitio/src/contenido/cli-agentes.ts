// Las páginas de la línea de comandos y de los agentes de IA (ADR 0033), en los
// dos idiomas. Los ejemplos se han probado con la CLI de verdad.
import type { Idioma } from "../i18n/rutas";
import type { PaginaDescarga as PaginaSecciones } from "./descargar";

const CLAUDE_CODE = [
  { titulo: "macOS", nombre: "claude-mcp", lineas: ['claude mcp add --scope user apolo -- "/Applications/Apolo.app/Contents/MacOS/apolo-mcp"'] },
  { titulo: "Windows", nombre: "claude-mcp", lineas: ['claude mcp add --scope user apolo -- "%LOCALAPPDATA%\\Apolo\\apolo-mcp.exe"'] },
  { titulo: "Linux (.deb, .rpm, PKGBUILD)", nombre: "claude-mcp", lineas: ["claude mcp add --scope user apolo -- /usr/bin/apolo-mcp"] },
];

export const CLI: Record<Idioma, PaginaSecciones> = {
  es: {
    titulo: "Línea de comandos de Apolo · Optimiza imágenes desde la terminal",
    descripcion:
      "apolo, la línea de comandos de Apolo: las opciones de cwebp, cjpeg, oxipng, avifenc y cjxl, lotes en paralelo con calidad por nota, y la medida SSIMULACRA 2.",
    etiqueta: "Línea de comandos",
    h1: "Apolo en la terminal.",
    entrada: "La línea de comandos apolo hace lo mismo que la aplicación, para scripts y servidores: los mismos motores, las mismas opciones y los mismos presets.",
    secciones: [
      {
        titulo: "Instalar",
        parrafos: [
          "En macOS y Linux, con Homebrew, que trae también el servidor para agentes de IA. En los demás casos, baja el archivo de tu sistema desde las descargas, en «Otras descargas».",
        ],
        comandos: [{ nombre: "brew", lineas: ["brew install webcafeina/apolo/apolo"] }],
      },
      {
        titulo: "Las opciones de siempre",
        parrafos: ["Cada formato acepta las opciones de su herramienta oficial, y da el mismo fichero, byte a byte:"],
        comandos: [
          { titulo: "WebP, como cwebp", nombre: "apolo-webp", lineas: ["apolo webp -q 80 foto.png -o foto.webp"] },
          { titulo: "AVIF, como avifenc", nombre: "apolo-avif", lineas: ["apolo avif -q 60 foto.png foto.avif"] },
          { titulo: "JPEG XL, como cjxl", nombre: "apolo-jxl", lineas: ["apolo jxl foto.png foto.jxl -d 1"] },
          { titulo: "JPEG, como cjpeg (MozJPEG)", nombre: "apolo-jpeg", lineas: ["apolo jpeg -quality 80 -outfile foto.jpg foto.png"] },
          { titulo: "PNG, como oxipng", nombre: "apolo-png", lineas: ["apolo png -o 4 foto.png"] },
        ],
      },
      {
        titulo: "Carpetas enteras",
        parrafos: [
          "apolo lote convierte carpetas e imágenes en paralelo, con sus subcarpetas, y nunca sobrescribe: si un fichero ya existe, el nuevo lleva un número.",
        ],
        comandos: [
          { titulo: "WebP y AVIF, con la calidad más baja que no se nota", nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato webp,avif --objetivo 80"] },
          { titulo: "A 1920 px de lado mayor, solo el formato más ligero de cada foto", nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato webp,avif,jpeg --mas-ligero --lado-mayor 1920"] },
          { titulo: "Con un preset guardado en la aplicación", nombre: "apolo-lote", lineas: ['apolo lote fotos/ --preset "Para la web"'] },
        ],
      },
      {
        titulo: "Medir la pérdida",
        parrafos: ["SSIMULACRA 2, PSNR y SSIM de un resultado frente a su original. Con 90 o más no se distingue; con 70, es alta calidad; con 50, se nota."],
        comandos: [{ nombre: "apolo-medir", lineas: ["apolo medir foto.png foto.webp"] }],
      },
      {
        titulo: "En inglés",
        parrafos: ["Por defecto, en el idioma del sistema. Para cambiarlo, --idioma en (o es)."],
      },
    ],
  },
  en: {
    titulo: "Apolo's command line · Optimize images from the terminal",
    descripcion:
      "apolo, Apolo's command line: the options of cwebp, cjpeg, oxipng, avifenc and cjxl, parallel batches with quality by score, and the SSIMULACRA 2 measure.",
    etiqueta: "Command line",
    h1: "Apolo in the terminal.",
    entrada: "The apolo command line does what the app does, for scripts and servers: the same engines, the same options and the same presets.",
    secciones: [
      {
        titulo: "Install",
        parrafos: [
          "On macOS and Linux, with Homebrew, which also brings the server for AI agents. Otherwise, download your system's archive from the downloads, under “Other downloads”.",
        ],
        comandos: [{ nombre: "brew", lineas: ["brew install webcafeina/apolo/apolo"] }],
      },
      {
        titulo: "The usual options",
        parrafos: ["Each format takes the options of its official tool, and gives the same file, byte for byte:"],
        comandos: [
          { titulo: "WebP, like cwebp", nombre: "apolo-webp", lineas: ["apolo webp -q 80 photo.png -o photo.webp"] },
          { titulo: "AVIF, like avifenc", nombre: "apolo-avif", lineas: ["apolo avif -q 60 photo.png photo.avif"] },
          { titulo: "JPEG XL, like cjxl", nombre: "apolo-jxl", lineas: ["apolo jxl photo.png photo.jxl -d 1"] },
          { titulo: "JPEG, like cjpeg (MozJPEG)", nombre: "apolo-jpeg", lineas: ["apolo jpeg -quality 80 -outfile photo.jpg photo.png"] },
          { titulo: "PNG, like oxipng", nombre: "apolo-png", lineas: ["apolo png -o 4 photo.png"] },
        ],
      },
      {
        titulo: "Whole folders",
        parrafos: [
          "apolo lote converts folders and images in parallel, with their subfolders, and never overwrites: if a file exists, the new one gets a number.",
        ],
        comandos: [
          { titulo: "WebP and AVIF, at the lowest quality that doesn't show", nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato webp,avif --objetivo 80"] },
          { titulo: "At 1920 px on the longest side, only the lightest format per photo", nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato webp,avif,jpeg --mas-ligero --lado-mayor 1920"] },
          { titulo: "With a preset saved in the app", nombre: "apolo-lote", lineas: ['apolo lote photos/ --preset "For the web"'] },
        ],
      },
      {
        titulo: "Measure the loss",
        parrafos: ["SSIMULACRA 2, PSNR and SSIM of a result against its original. At 90 or more you can't tell them apart; at 70, it's high quality; at 50, it shows."],
        comandos: [{ nombre: "apolo-medir", lineas: ["apolo medir photo.png photo.webp"] }],
      },
      {
        titulo: "In Spanish",
        parrafos: ["By default, in your system's language. To change it, --idioma es (or en)."],
      },
    ],
  },
};

export const AGENTES: Record<Idioma, PaginaSecciones> = {
  es: {
    titulo: "Apolo para agentes de IA · Optimiza imágenes con Claude, Cursor o VS Code",
    descripcion:
      "El servidor MCP de Apolo: Claude, Cursor, VS Code y cualquier agente optimizan, comparan y convierten imágenes en tu ordenador, sin sobrescribir nada.",
    etiqueta: "Agentes de IA",
    h1: "Apolo, desde tu agente de IA.",
    entrada: "Claude, Cursor, VS Code y cualquier agente que hable MCP optimizan tus imágenes con los motores de Apolo, en tu ordenador.",
    secciones: [
      {
        titulo: "Qué puede hacer",
        lista: [
          "Analizar una imagen o una carpeta entera.",
          "Optimizar una imagen, con la orden equivalente de cwebp, cjpeg, oxipng, avifenc o cjxl.",
          "Comparar formatos a la misma calidad percibida, y medir la pérdida con SSIMULACRA 2.",
          "Recortar, redimensionar y mirar las imágenes.",
          "Convertir carpetas enteras en lotes, y guardar presets.",
        ],
        parrafos: ["Nunca sobrescribe ni borra: siempre escribe ficheros nuevos."],
      },
      {
        titulo: "Claude Desktop",
        parrafos: [
          "En Apolo, la fila MCP de la barra lateral crea la extensión, que se instala una vez. También está en las descargas, en «Otras descargas». El servidor va dentro de la aplicación y se actualiza con ella: después de actualizar Apolo, cierra y vuelve a abrir Claude Desktop.",
        ],
      },
      {
        titulo: "Claude Code",
        comandos: CLAUDE_CODE,
      },
      {
        titulo: "Cursor, VS Code y los demás",
        parrafos: ["En Apolo, la fila MCP te da lo que hay que copiar en cada uno."],
      },
      {
        titulo: "Privacidad",
        parrafos: [
          "Las imágenes se procesan en tu ordenador. Lo que el agente pide (rutas, nombres, miniaturas) entra en su conversación, y lo ve el proveedor del modelo.",
        ],
      },
    ],
  },
  en: {
    titulo: "Apolo for AI agents · Optimize images with Claude, Cursor or VS Code",
    descripcion:
      "Apolo's MCP server: Claude, Cursor, VS Code and any agent optimize, compare and convert images on your computer, without overwriting anything.",
    etiqueta: "AI agents",
    h1: "Apolo, from your AI agent.",
    entrada: "Claude, Cursor, VS Code and any agent that speaks MCP optimize your images with Apolo's engines, on your computer.",
    secciones: [
      {
        titulo: "What it can do",
        lista: [
          "Analyze an image or a whole folder.",
          "Optimize an image, with the equivalent cwebp, cjpeg, oxipng, avifenc or cjxl command.",
          "Compare formats at the same perceived quality, and measure the loss with SSIMULACRA 2.",
          "Crop, resize and look at images.",
          "Convert whole folders in batches, and save presets.",
        ],
        parrafos: ["It never overwrites or deletes: it always writes new files."],
      },
      {
        titulo: "Claude Desktop",
        parrafos: [
          "In Apolo, the MCP row in the sidebar creates the extension, which you install once. It's also in the downloads, under “Other downloads”. The server lives inside the app and updates with it: after updating Apolo, quit and reopen Claude Desktop.",
        ],
      },
      {
        titulo: "Claude Code",
        comandos: CLAUDE_CODE,
      },
      {
        titulo: "Cursor, VS Code and the rest",
        parrafos: ["In Apolo, the MCP row gives you what to copy into each one."],
      },
      {
        titulo: "Privacy",
        parrafos: [
          "Images are processed on your computer. What the agent asks for (paths, names, thumbnails) goes into its conversation, and the model provider sees it.",
        ],
      },
    ],
  },
};
