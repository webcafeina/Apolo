// Las comparativas (ADR 0034): Apolo frente a Squoosh, TinyPNG, ImageOptim y
// Caesium, en los dos idiomas. Honestas y con fecha: lo de cada herramienta se
// comprobó en su web y en su repositorio el día de COMPROBADO, y se dice lo
// que hace mejor cada una. Son el único sitio público donde se nombra Squoosh
// (publico.test.ts lo vigila).
import type { Idioma } from "../i18n/rutas";
import type { Articulo, Seccion } from "./articulos";

/** La cabeza del índice de comparativas (aquí, y no en i18n, porque nombra a Squoosh). */
export const INDICE: Record<Idioma, { titulo: string; descripcion: string; etiqueta: string; h1: string; entrada: string }> = {
  es: {
    titulo: "Apolo frente a Squoosh, TinyPNG, ImageOptim y Caesium · Apolo",
    descripcion:
      "Apolo comparado con otras herramientas para comprimir imágenes: Squoosh, TinyPNG, ImageOptim y Caesium. Lo que hace mejor cada una, con datos comprobados y fecha.",
    etiqueta: "Comparativas",
    h1: "Comparativas.",
    entrada: "Apolo frente a otras herramientas para comprimir imágenes, con lo que hace mejor cada una.",
  },
  en: {
    titulo: "Apolo vs Squoosh, TinyPNG, ImageOptim and Caesium · Apolo",
    descripcion:
      "Apolo compared with other image compression tools: Squoosh, TinyPNG, ImageOptim and Caesium. What each one does better, with checked, dated facts.",
    etiqueta: "Comparisons",
    h1: "Comparisons.",
    entrada: "Apolo against other image compression tools, with what each one does better.",
  },
};

const COMPROBADO = "2026-10-27";
const FILAS = {
  es: ["Precio", "Licencia", "Dónde funciona", "Tus imágenes", "Formatos que escribe", "Carpetas enteras", "Línea de comandos", "Agentes de IA (MCP)", "Mide la pérdida"],
  en: ["Price", "License", "Where it runs", "Your images", "Formats it writes", "Whole folders", "Command line", "AI agents (MCP)", "Measures the loss"],
};
const APOLO = {
  es: [
    "Gratis",
    "GPL-3.0 (código abierto)",
    "macOS, Windows y Linux, en x64 y ARM",
    "No salen de tu ordenador",
    "WebP, AVIF, JPEG XL, JPEG (MozJPEG), PNG (OxiPNG) y QOI",
    "Sí, en paralelo y con subcarpetas",
    "Sí, con las opciones de cwebp, cjpeg, oxipng, avifenc y cjxl",
    "Sí, once herramientas",
    "Sí: SSIMULACRA 2, PSNR y SSIM, y calidad por nota",
  ],
  en: [
    "Free",
    "GPL-3.0 (open source)",
    "macOS, Windows and Linux, on x64 and ARM",
    "Never leave your computer",
    "WebP, AVIF, JPEG XL, JPEG (MozJPEG), PNG (OxiPNG) and QOI",
    "Yes, in parallel and with subfolders",
    "Yes, with the options of cwebp, cjpeg, oxipng, avifenc and cjxl",
    "Yes, eleven tools",
    "Yes: SSIMULACRA 2, PSNR and SSIM, and quality by score",
  ],
};

function tabla(idioma: Idioma, nombre: string, otra: string[]): Seccion["tabla"] {
  return {
    cabeza: [idioma === "es" ? "Qué" : "What", "Apolo", nombre],
    filas: FILAS[idioma].map((f, i) => [f, APOLO[idioma][i], otra[i]]),
    nota:
      idioma === "es"
        ? `Lo de ${nombre}, comprobado en su web y en su repositorio el 27 de octubre de 2026. Si algo ha cambiado, escríbenos.`
        : `${nombre}'s details checked on its website and repository on October 27, 2026. If something has changed, let us know.`,
  };
}

interface Rival {
  id: "frenteSquoosh" | "frenteTinypng" | "frenteImageoptim" | "frenteCaesium";
  nombre: string;
  datos: Record<Idioma, string[]>;
  texto: Record<
    Idioma,
    { titulo: string; descripcion: string; entrada: string; resumen: string; corto: string[]; mejorElla: string[]; mejorApolo: string[]; elegir: string[] }
  >;
}

const RIVALES: Rival[] = [
  {
    id: "frenteSquoosh",
    nombre: "Squoosh",
    datos: {
      es: [
        "Gratis",
        "Apache-2.0 (código abierto)",
        "En el navegador, sin instalar nada",
        "No salen de tu ordenador: las procesa el navegador",
        "WebP, AVIF, JPEG XL, MozJPEG, OxiPNG, QOI y otros",
        "No: una imagen cada vez",
        "No",
        "No",
        "No: se compara a ojo",
      ],
      en: [
        "Free",
        "Apache-2.0 (open source)",
        "In the browser, nothing to install",
        "Never leave your computer: the browser processes them",
        "WebP, AVIF, JPEG XL, MozJPEG, OxiPNG, QOI and others",
        "No: one image at a time",
        "No",
        "No",
        "No: you compare by eye",
      ],
    },
    texto: {
      es: {
        titulo: "Apolo frente a Squoosh: lo mismo, en tu escritorio y por carpetas · Apolo",
        descripcion:
          "Apolo y Squoosh usan los mismos motores (libwebp, MozJPEG, OxiPNG, AVIF, JPEG XL). Squoosh funciona en el navegador, imagen a imagen; Apolo es una aplicación con lotes, CLI y agentes de IA.",
        entrada: "Los dos comprimen con los mismos motores y enseñan el antes y el después. La diferencia está en todo lo demás.",
        resumen: "Los mismos motores; Squoosh en el navegador y de una en una, Apolo en el escritorio, por carpetas y midiendo la pérdida.",
        corto: [
          "Squoosh es la web de compresión de imágenes de Google Chrome Labs: funciona en el navegador, sin subir nada, y enseña el original y el resultado con un deslizador. Es excelente para una imagen suelta.",
          "Apolo hace lo mismo en una aplicación nativa, y añade lo que Squoosh no tiene: carpetas enteras, una nota de calidad que se puede pedir, línea de comandos y un servidor para agentes de IA.",
        ],
        mejorElla: [
          "No hay que instalar nada: se abre en cualquier navegador, también en un ordenador que no es tuyo.",
          "Tiene algún códec experimental más, como WebP 2.",
        ],
        mejorApolo: [
          "Carpetas enteras, en paralelo y con subcarpetas, sin sobrescribir nada.",
          "Mide la pérdida con SSIMULACRA 2 y busca la calidad más baja que llega a la nota que pidas.",
          "Da el mismo fichero, byte a byte, que cwebp, cjpeg, oxipng, avifenc y cjxl, con la orden equivalente para copiarla.",
          "Abre HEIC del iPhone, trae línea de comandos y servidor MCP, y se actualiza sola. El código de Squoosh no cambia desde agosto de 2024.",
        ],
        elegir: [
          "Una imagen de vez en cuando, en un ordenador cualquiera: Squoosh.",
          "Muchas imágenes, carpetas, un flujo de trabajo o un agente de IA: Apolo.",
        ],
      },
      en: {
        titulo: "Apolo vs Squoosh: the same, on your desktop and for whole folders · Apolo",
        descripcion:
          "Apolo and Squoosh use the same engines (libwebp, MozJPEG, OxiPNG, AVIF, JPEG XL). Squoosh runs in the browser, one image at a time; Apolo is an app with batches, a CLI and AI agents.",
        entrada: "Both compress with the same engines and show the before and after. The difference is in everything else.",
        resumen: "The same engines; Squoosh in the browser and one by one, Apolo on the desktop, by folders and measuring the loss.",
        corto: [
          "Squoosh is Google Chrome Labs' image compression web app: it runs in the browser, uploads nothing, and shows the original and the result with a slider. It's excellent for a single image.",
          "Apolo does the same in a native app, and adds what Squoosh doesn't have: whole folders, a quality score you can ask for, a command line and a server for AI agents.",
        ],
        mejorElla: [
          "Nothing to install: it opens in any browser, even on a computer that isn't yours.",
          "It has an extra experimental codec or two, like WebP 2.",
        ],
        mejorApolo: [
          "Whole folders, in parallel and with subfolders, without overwriting anything.",
          "It measures the loss with SSIMULACRA 2 and finds the lowest quality that reaches the score you ask for.",
          "It gives the same file, byte for byte, as cwebp, cjpeg, oxipng, avifenc and cjxl, with the equivalent command to copy.",
          "It opens iPhone HEIC, ships a command line and an MCP server, and updates itself. Squoosh's code hasn't changed since August 2024.",
        ],
        elegir: ["An image now and then, on any computer: Squoosh.", "Many images, folders, a workflow or an AI agent: Apolo."],
      },
    },
  },
  {
    id: "frenteTinypng",
    nombre: "TinyPNG",
    datos: {
      es: [
        "Gratis con límites; Web Pro y Web Ultra, de pago",
        "Propietaria",
        "En el navegador, con API, CDN y plugin de WordPress",
        "Se suben a sus servidores, que las borran a las 48 horas",
        "AVIF, WebP, PNG y JPEG",
        "Gratis, hasta 20 imágenes de 5 MB cada vez",
        "No; API de pago, con 500 compresiones gratis al mes",
        "No",
        "No",
      ],
      en: [
        "Free with limits; Web Pro and Web Ultra are paid",
        "Proprietary",
        "In the browser, with an API, a CDN and a WordPress plugin",
        "Uploaded to their servers, which delete them after 48 hours",
        "AVIF, WebP, PNG and JPEG",
        "Free, up to 20 images of 5 MB at a time",
        "No; a paid API, with 500 free compressions a month",
        "No",
        "No",
      ],
    },
    texto: {
      es: {
        titulo: "Apolo frente a TinyPNG: comprimir imágenes sin subirlas · Apolo",
        descripcion:
          "TinyPNG comprime en sus servidores, con límites en la versión gratis. Apolo es gratis y sin límites, trabaja en tu ordenador y escribe también JPEG XL. Lo que hace mejor cada uno.",
        entrada: "TinyPNG es la web de compresión más conocida. Apolo hace el trabajo en tu ordenador, sin límites y sin subir nada.",
        resumen: "TinyPNG en sus servidores y con límites; Apolo en tu ordenador, gratis y sin límites.",
        corto: [
          "TinyPNG (de Tinify) comprime y convierte imágenes en su web: las subes, las procesa en sus servidores y las borra a las 48 horas. Gratis hasta 20 imágenes de 5 MB cada vez, y la conversión de formato, para unas pocas; para más, planes de pago.",
          "Apolo hace el trabajo en tu ordenador: sin subir nada, sin límite de imágenes ni de tamaño, y viendo lo que se pierde en cada una.",
        ],
        mejorElla: [
          "No hay que instalar nada, y funciona desde el móvil.",
          "Tiene plugin de WordPress, CDN de imágenes e integraciones con Figma, Make y Zapier.",
        ],
        mejorApolo: [
          "Tus imágenes no salen de tu ordenador: importa con fotos de clientes, documentos o cualquier cosa privada.",
          "Gratis sin límites, también para carpetas enteras.",
          "Escribe JPEG XL y QOI, y abre HEIC.",
          "Enseña el resultado frente al original y mide la pérdida con SSIMULACRA 2.",
        ],
        elegir: [
          "Unas pocas imágenes desde cualquier dispositivo, o comprimir dentro de WordPress: TinyPNG.",
          "Imágenes privadas, muchas, o sin pagar por volumen: Apolo.",
        ],
      },
      en: {
        titulo: "Apolo vs TinyPNG: compress images without uploading them · Apolo",
        descripcion:
          "TinyPNG compresses on its servers, with limits on the free version. Apolo is free with no limits, works on your computer and also writes JPEG XL. What each one does better.",
        entrada: "TinyPNG is the best-known compression website. Apolo does the work on your computer, with no limits and nothing uploaded.",
        resumen: "TinyPNG on its servers and with limits; Apolo on your computer, free and unlimited.",
        corto: [
          "TinyPNG (by Tinify) compresses and converts images on its website: you upload them, it processes them on its servers and deletes them after 48 hours. Free for up to 20 images of 5 MB at a time, and format conversion for a few; beyond that, paid plans.",
          "Apolo does the work on your computer: nothing uploaded, no limit on images or size, and seeing what you lose in each one.",
        ],
        mejorElla: ["Nothing to install, and it works from a phone.", "It has a WordPress plugin, an image CDN and integrations with Figma, Make and Zapier."],
        mejorApolo: [
          "Your images never leave your computer: it matters with client photos, documents or anything private.",
          "Free with no limits, including whole folders.",
          "It writes JPEG XL and QOI, and opens HEIC.",
          "It shows the result against the original and measures the loss with SSIMULACRA 2.",
        ],
        elegir: [
          "A few images from any device, or compressing inside WordPress: TinyPNG.",
          "Private images, lots of them, or without paying by volume: Apolo.",
        ],
      },
    },
  },
  {
    id: "frenteImageoptim",
    nombre: "ImageOptim",
    datos: {
      es: [
        "Gratis",
        "GPL-2.0 o posterior (código abierto)",
        "Solo macOS (11 o posterior)",
        "No salen de tu ordenador",
        "Optimiza JPEG, PNG, GIF y SVG sin cambiar de formato; no escribe WebP ni AVIF",
        "Sí, arrastrando varias",
        "Sí",
        "No",
        "No; sin pérdida por defecto",
      ],
      en: [
        "Free",
        "GPL-2.0 or later (open source)",
        "macOS only (11 or later)",
        "Never leave your computer",
        "Optimizes JPEG, PNG, GIF and SVG without changing format; doesn't write WebP or AVIF",
        "Yes, by dragging several",
        "Yes",
        "No",
        "No; lossless by default",
      ],
    },
    texto: {
      es: {
        titulo: "Apolo frente a ImageOptim: optimizar o convertir, en Mac y más allá · Apolo",
        descripcion:
          "ImageOptim aligera JPEG, PNG, GIF y SVG en el Mac sin cambiarlos de formato. Apolo los convierte además a WebP, AVIF o JPEG XL, en macOS, Windows y Linux. Cuándo usar cada uno.",
        entrada: "ImageOptim es un clásico del Mac para aligerar imágenes sin tocarlas. Apolo además las convierte a los formatos nuevos.",
        resumen: "ImageOptim aligera sin cambiar de formato, solo en Mac; Apolo convierte a WebP, AVIF y JPEG XL en los tres sistemas.",
        corto: [
          "ImageOptim es una aplicación gratuita y libre para Mac: arrastras imágenes y las aligera sin cambiarlas de formato, sin pérdida por defecto. También trata GIF animados y SVG.",
          "Apolo, además de optimizar JPEG y PNG, convierte a WebP, AVIF y JPEG XL, que pesan bastante menos, y funciona también en Windows y Linux.",
        ],
        mejorElla: [
          "Optimiza SVG y GIF animados, que Apolo no escribe.",
          "Se integra con el Finder: arrastrar y listo, sin decidir nada.",
        ],
        mejorApolo: [
          "Convierte a WebP, AVIF y JPEG XL: a la misma calidad percibida, mucho más ligeros que el JPEG o el PNG optimizados.",
          "Enseña el resultado y mide la pérdida con SSIMULACRA 2, con calidad por nota.",
          "Funciona en macOS, Windows y Linux, y trae servidor para agentes de IA.",
        ],
        elegir: [
          "Aligerar JPEG, PNG, GIF o SVG sin cambiar nada, en el Mac: ImageOptim.",
          "Pasar a formatos modernos, en cualquier sistema, o ver y medir la pérdida: Apolo.",
        ],
      },
      en: {
        titulo: "Apolo vs ImageOptim: optimize or convert, on Mac and beyond · Apolo",
        descripcion:
          "ImageOptim makes JPEG, PNG, GIF and SVG lighter on the Mac without changing their format. Apolo also converts them to WebP, AVIF or JPEG XL, on macOS, Windows and Linux. When to use each.",
        entrada: "ImageOptim is a Mac classic for making images lighter without changing them. Apolo also converts them to the new formats.",
        resumen: "ImageOptim makes images lighter without changing format, on Mac only; Apolo converts to WebP, AVIF and JPEG XL on all three systems.",
        corto: [
          "ImageOptim is a free, open-source Mac app: you drag images onto it and it makes them lighter without changing their format, lossless by default. It also handles animated GIFs and SVG.",
          "Apolo, besides optimizing JPEG and PNG, converts to WebP, AVIF and JPEG XL, which weigh considerably less, and also runs on Windows and Linux.",
        ],
        mejorElla: ["It optimizes SVG and animated GIFs, which Apolo doesn't write.", "It plugs into the Finder: drag and done, nothing to decide."],
        mejorApolo: [
          "It converts to WebP, AVIF and JPEG XL: at the same perceived quality, much lighter than an optimized JPEG or PNG.",
          "It shows the result and measures the loss with SSIMULACRA 2, with quality by score.",
          "It runs on macOS, Windows and Linux, and ships a server for AI agents.",
        ],
        elegir: [
          "Making JPEG, PNG, GIF or SVG lighter without changing anything, on a Mac: ImageOptim.",
          "Moving to modern formats, on any system, or seeing and measuring the loss: Apolo.",
        ],
      },
    },
  },
  {
    id: "frenteCaesium",
    nombre: "Caesium",
    datos: {
      es: [
        "Gratis",
        "GPL-3.0 (código abierto)",
        "Windows 10 y macOS 12 o posteriores; en Linux, compilándolo",
        "No salen de tu ordenador (tiene también una versión web)",
        "JPG, PNG, WebP y TIFF",
        "Sí, varias a la vez",
        "Sí, aparte (CaesiumCLT)",
        "No",
        "No; vista previa del resultado",
      ],
      en: [
        "Free",
        "GPL-3.0 (open source)",
        "Windows 10 and macOS 12 or later; on Linux, building it yourself",
        "Never leave your computer (it also has a web version)",
        "JPG, PNG, WebP and TIFF",
        "Yes, several at once",
        "Yes, separately (CaesiumCLT)",
        "No",
        "No; a preview of the result",
      ],
    },
    texto: {
      es: {
        titulo: "Apolo frente a Caesium: dos compresores libres de escritorio · Apolo",
        descripcion:
          "Caesium y Apolo son compresores de imágenes gratuitos y libres para el escritorio. Caesium trabaja con JPG, PNG, WebP y TIFF; Apolo añade AVIF, JPEG XL, HEIC, ARM y agentes de IA.",
        entrada: "Los dos son libres, gratuitos y trabajan en tu ordenador. Cambian los formatos, los sistemas y cómo se mide el resultado.",
        resumen: "Dos compresores libres de escritorio: Caesium con JPG, PNG, WebP y TIFF; Apolo con AVIF, JPEG XL y medida de la pérdida.",
        corto: [
          "Caesium Image Compressor es una aplicación gratuita y libre para Windows y Mac (en Linux hay que compilarla) que comprime JPG, PNG, WebP y TIFF por lotes, con vista previa. Su última versión, la 2.8.5, es de mayo de 2025, y prepara la 3.",
          "Apolo hace también lotes en el escritorio, y añade AVIF y JPEG XL, la nota de calidad SSIMULACRA 2, versiones para ARM y Linux listas para instalar, y servidor para agentes de IA.",
        ],
        mejorElla: ["Trabaja con TIFF, que Apolo lee pero no escribe.", "Tiene una versión web para comprimir unas pocas sin instalar nada."],
        mejorApolo: [
          "Escribe AVIF y JPEG XL, que pesan bastante menos que JPG y WebP a la misma calidad percibida.",
          "Mide la pérdida con SSIMULACRA 2 y busca la calidad más baja que llega a una nota.",
          "Paquetes para Linux (.deb, .rpm, AppImage y repositorios), y para Windows y Linux en ARM.",
          "Abre HEIC del iPhone y trae servidor MCP para agentes de IA.",
        ],
        elegir: ["JPG, PNG o TIFF con una herramienta sencilla, en Windows o Mac: Caesium.", "Formatos modernos, Linux o ARM, o medir la pérdida: Apolo."],
      },
      en: {
        titulo: "Apolo vs Caesium: two free desktop image compressors · Apolo",
        descripcion:
          "Caesium and Apolo are free, open-source desktop image compressors. Caesium handles JPG, PNG, WebP and TIFF; Apolo adds AVIF, JPEG XL, HEIC, ARM and AI agents.",
        entrada: "Both are free, open source and work on your computer. What changes is the formats, the systems and how the result is measured.",
        resumen: "Two free desktop compressors: Caesium with JPG, PNG, WebP and TIFF; Apolo with AVIF, JPEG XL and loss measurement.",
        corto: [
          "Caesium Image Compressor is a free, open-source app for Windows and Mac (on Linux you build it yourself) that compresses JPG, PNG, WebP and TIFF in batches, with a preview. Its latest release, 2.8.5, is from May 2025, and version 3 is in the works.",
          "Apolo also does batches on the desktop, and adds AVIF and JPEG XL, the SSIMULACRA 2 quality score, ready-to-install ARM and Linux builds, and a server for AI agents.",
        ],
        mejorElla: ["It handles TIFF, which Apolo reads but doesn't write.", "It has a web version to compress a few images without installing anything."],
        mejorApolo: [
          "It writes AVIF and JPEG XL, which weigh considerably less than JPG and WebP at the same perceived quality.",
          "It measures the loss with SSIMULACRA 2 and finds the lowest quality that reaches a score.",
          "Packages for Linux (.deb, .rpm, AppImage and repositories), and for Windows and Linux on ARM.",
          "It opens iPhone HEIC and ships an MCP server for AI agents.",
        ],
        elegir: ["JPG, PNG or TIFF with a simple tool, on Windows or Mac: Caesium.", "Modern formats, Linux or ARM, or measuring the loss: Apolo."],
      },
    },
  },
];

const ENCABEZADOS = {
  es: { corto: "En corto", tabla: "Lado a lado", mejorElla: (n: string) => `Lo que hace mejor ${n}`, mejorApolo: "Lo que hace mejor Apolo", elegir: "Cuál elegir" },
  en: { corto: "In short", tabla: "Side by side", mejorElla: (n: string) => `What ${n} does better`, mejorApolo: "What Apolo does better", elegir: "Which to choose" },
};

export const COMPARATIVAS: Record<Idioma, Articulo[]> = {
  es: RIVALES.map((r) => articulo(r, "es")),
  en: RIVALES.map((r) => articulo(r, "en")),
};

function articulo(r: Rival, idioma: Idioma): Articulo {
  const t = r.texto[idioma];
  const e = ENCABEZADOS[idioma];
  return {
    pagina: r.id,
    publicado: COMPROBADO,
    revisado: COMPROBADO,
    titulo: t.titulo,
    descripcion: t.descripcion,
    etiqueta: idioma === "es" ? "Comparativa" : "Comparison",
    h1: idioma === "es" ? `Apolo frente a ${r.nombre}.` : `Apolo vs ${r.nombre}.`,
    entrada: t.entrada,
    resumen: t.resumen,
    relacionadas: (["frenteSquoosh", "frenteTinypng", "frenteImageoptim", "frenteCaesium"] as const).filter((p) => p !== r.id),
    secciones: [
      { titulo: e.corto, parrafos: t.corto },
      { titulo: e.tabla, tabla: tabla(idioma, r.nombre, r.datos[idioma]) },
      { titulo: e.mejorElla(r.nombre), lista: t.mejorElla },
      { titulo: e.mejorApolo, lista: t.mejorApolo },
      { titulo: e.elegir, lista: t.elegir, enlaces: [{ texto: idioma === "es" ? "AVIF, WebP o JPEG XL: las medidas" : "AVIF, WebP or JPEG XL: the measurements", pagina: "guiaFormatos" }] },
    ],
  };
}
