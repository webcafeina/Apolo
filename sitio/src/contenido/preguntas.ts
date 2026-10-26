// Las preguntas frecuentes (ADR 0033), en los dos idiomas. Van también como
// datos estructurados (FAQPage), con el texto sin marcas.
import type { Idioma } from "../i18n/rutas";

export interface Pregunta {
  p: string;
  r: string[];
}

export const PREGUNTAS: Record<Idioma, { titulo: string; descripcion: string; etiqueta: string; h1: string; entrada: string; lista: Pregunta[] }> = {
  es: {
    titulo: "Preguntas frecuentes · Apolo",
    descripcion: "Lo que suele preguntarse de Apolo: si es gratis, si sube tus imágenes, por qué avisa el sistema al abrirla, qué formatos lee y escribe, y cómo se actualiza.",
    etiqueta: "Preguntas",
    h1: "Preguntas frecuentes.",
    entrada: "Lo que suele preguntarse antes de instalar Apolo, y después.",
    lista: [
      {
        p: "¿Apolo es gratis?",
        r: ["Sí. Es software libre, con licencia GPLv3: se puede usar, copiar y cambiar, también en el trabajo. El código está en GitHub."],
      },
      {
        p: "¿Sube mis imágenes a algún sitio?",
        r: [
          "No. Todo pasa en tu ordenador: las imágenes no salen de él.",
          "Lo único que Apolo hace fuera es preguntar a GitHub, una vez al día, cuál es la última versión. No manda tus imágenes, ni quién eres, ni cuántas veces la usas. Se puede apagar en Ajustes.",
        ],
      },
      {
        p: "¿Por qué mi sistema avisa al abrirla la primera vez?",
        r: [
          "Porque Apolo no está firmada con un certificado de Apple ni de Microsoft. En macOS, ve a Ajustes del Sistema › Privacidad y seguridad y pulsa «Abrir igualmente». En Windows, en SmartScreen, pulsa «Más información» y «Ejecutar de todos modos».",
        ],
      },
      {
        p: "¿Qué formatos lee y escribe?",
        r: [
          "Escribe WebP, AVIF, JPEG XL, JPEG (con MozJPEG), PNG (con OxiPNG) y QOI.",
          "Lee esos mismos y además HEIC (las fotos del iPhone), TIFF, GIF, BMP y PNM.",
        ],
      },
      {
        p: "¿Da el mismo resultado que las herramientas oficiales?",
        r: [
          "Sí, byte a byte: cwebp, cjpeg (MozJPEG), oxipng, avifenc y cjxl van dentro de Apolo, con todas sus opciones. Apolo te dice la orden equivalente para repetirla donde quieras, y se comprueba en cada versión.",
        ],
      },
      {
        p: "¿Qué es la nota SSIMULACRA 2?",
        r: [
          "Una medida de cuánto se nota la pérdida, pensada para parecerse a lo que ve una persona. Con 90 no se distingue ni alternando las dos imágenes; con 80, no se distingue puesta al lado; por debajo de 50, se nota.",
          "Con «Calidad por nota», Apolo busca la calidad más baja que llega a la nota que le pidas, foto a foto.",
        ],
      },
      {
        p: "¿Funciona en un Mac con Intel, en Windows ARM o en Linux ARM?",
        r: ["Sí. El .dmg de macOS sirve para Apple Silicon e Intel, y Windows y Linux tienen su versión para x64 y para ARM."],
      },
      {
        p: "¿Cuánta memoria necesita?",
        r: ["Con fotos de 24 megapíxeles no pasa de unos 2 GB, ni en lotes de cientos de fotos: cabe en un ordenador de 8 GB."],
      },
      {
        p: "¿Cómo se actualiza?",
        r: [
          "Sola: cuando hay una versión nueva, Apolo lo avisa y se instala con un clic. En Linux, con el repositorio, se actualiza con el resto del sistema.",
        ],
      },
      {
        p: "¿Se puede usar desde la terminal?",
        r: ["Sí, con la línea de comandos apolo, que acepta las opciones de cwebp, cjpeg, oxipng, avifenc y cjxl, y convierte carpetas enteras con apolo lote."],
      },
      {
        p: "¿Funciona con agentes de IA?",
        r: [
          "Sí. Apolo lleva un servidor MCP para Claude, Cursor, VS Code y cualquier agente que lo hable: analiza, optimiza, compara formatos y convierte carpetas en tu ordenador, y nunca sobrescribe ni borra nada.",
        ],
      },
      {
        p: "¿Quién hace Apolo?",
        r: ["Webcafeína, un estudio de diseño y desarrollo web."],
      },
    ],
  },
  en: {
    titulo: "FAQ · Apolo",
    descripcion: "What people usually ask about Apolo: whether it's free, whether it uploads your images, why your system warns you when you open it, which formats it reads and writes, and how it updates.",
    etiqueta: "FAQ",
    h1: "Frequently asked questions.",
    entrada: "What people usually ask before installing Apolo, and after.",
    lista: [
      {
        p: "Is Apolo free?",
        r: ["Yes. It's free software under the GPLv3: you can use, copy and change it, at work too. The code is on GitHub."],
      },
      {
        p: "Does it upload my images anywhere?",
        r: [
          "No. Everything happens on your computer: your images never leave it.",
          "The only thing Apolo does outside is ask GitHub, once a day, which version is the latest. It doesn't send your images, who you are, or how often you use it. You can turn it off in Settings.",
        ],
      },
      {
        p: "Why does my system warn me the first time I open it?",
        r: [
          "Because Apolo isn't signed with an Apple or Microsoft certificate. On macOS, go to System Settings › Privacy & Security and click “Open Anyway”. On Windows, in SmartScreen, click “More info” and “Run anyway”.",
        ],
      },
      {
        p: "Which formats does it read and write?",
        r: [
          "It writes WebP, AVIF, JPEG XL, JPEG (with MozJPEG), PNG (with OxiPNG) and QOI.",
          "It reads those, plus HEIC (iPhone photos), TIFF, GIF, BMP and PNM.",
        ],
      },
      {
        p: "Does it give the same result as the official tools?",
        r: [
          "Yes, byte for byte: cwebp, cjpeg (MozJPEG), oxipng, avifenc and cjxl are inside Apolo, with all their options. Apolo shows you the equivalent command so you can run it anywhere, and it's checked in every version.",
        ],
      },
      {
        p: "What is the SSIMULACRA 2 score?",
        r: [
          "A measure of how visible the loss is, designed to match what a person sees. At 90 you can't tell them apart even flipping between both images; at 80, you can't tell them apart side by side; below 50, it shows.",
          "With “Quality by score”, Apolo finds the lowest quality that reaches the score you ask for, photo by photo.",
        ],
      },
      {
        p: "Does it work on an Intel Mac, Windows on ARM or Linux on ARM?",
        r: ["Yes. The macOS .dmg works on Apple silicon and Intel, and Windows and Linux have their own versions for x64 and ARM."],
      },
      {
        p: "How much memory does it need?",
        r: ["With 24-megapixel photos it stays around 2 GB, even in batches of hundreds of photos: it fits a computer with 8 GB."],
      },
      {
        p: "How does it update?",
        r: ["By itself: when there's a new version, Apolo tells you and installs it with one click. On Linux, with the repository, it updates with the rest of your system."],
      },
      {
        p: "Can I use it from the terminal?",
        r: ["Yes, with the apolo command line, which takes the options of cwebp, cjpeg, oxipng, avifenc and cjxl, and converts whole folders with apolo lote."],
      },
      {
        p: "Does it work with AI agents?",
        r: [
          "Yes. Apolo includes an MCP server for Claude, Cursor, VS Code and any agent that speaks it: it analyzes, optimizes, compares formats and converts folders on your computer, and it never overwrites or deletes anything.",
        ],
      },
      {
        p: "Who makes Apolo?",
        r: ["Webcafeína, a web design and development studio."],
      },
    ],
  },
};
