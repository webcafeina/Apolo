// La privacidad de Apolo, el programa (ADR 0036): la aplicación, la CLI, el servidor MCP y el plugin
// de Claude. La pide el directorio de plugins de Claude. Los datos de
// Webcafeína como responsable y los derechos, en su política de privacidad; las cookies de esta
// web, en la suya. Si Apolo empieza a conectarse a algo más, esta página cambia con él.
import type { Idioma } from "../i18n/rutas";
import type { PaginaDescarga as PaginaSecciones } from "./descargar";

export const PRIVACIDAD: Record<Idioma, PaginaSecciones> = {
  es: {
    titulo: "Privacidad · Apolo",
    descripcion: "Qué datos usa Apolo: tus imágenes no salen de tu ordenador, la aplicación solo consulta GitHub para buscar actualizaciones y el servidor MCP no se conecta a nada.",
    etiqueta: "Privacidad",
    h1: "Tus imágenes no salen de tu ordenador.",
    entrada: "Qué datos usan la aplicación, la línea de comandos, el servidor MCP y el plugin de Claude, y a quién llegan. Actualizada el 3 de noviembre de 2026.",
    secciones: [
      {
        titulo: "La aplicación",
        parrafos: [
          "Apolo trabaja con tus imágenes en tu ordenador: las abre, las convierte y guarda los resultados en tu disco. No las sube a ningún sitio, no pide cuenta ni registro y no lleva estadísticas de uso ni informes de errores.",
          "Solo se conecta a un sitio: GitHub, una vez al día, para saber si hay una versión nueva, y para bajarla si le dices que la instale. GitHub ve, como en cualquier descarga, tu dirección IP. Se puede apagar en Ajustes. Si la instalaste con un gestor de paquetes (APT, DNF, Homebrew, Chocolatey…), las actualizaciones las busca el gestor en su repositorio.",
        ],
      },
      {
        titulo: "La línea de comandos y el servidor MCP",
        parrafos: [
          "apolo y apolo-mcp no se conectan a nada: leen las imágenes de las rutas que les des y escriben ficheros nuevos junto a ellas, sin sobrescribir ni borrar.",
        ],
      },
      {
        titulo: "Con un agente de IA",
        parrafos: [
          "Cuando un agente (Claude, Cursor, VS Code…) usa Apolo, lo que Apolo le responde entra en su conversación: rutas, nombres de fichero, medidas, pesos, notas de calidad y, si el agente pide verla, una miniatura de la imagen. Esa conversación la procesa el proveedor del modelo que uses, con su propia política de privacidad. Webcafeína no recibe nada.",
        ],
      },
      {
        titulo: "El plugin de Claude",
        parrafos: [
          "El plugin de Claude solo lleva un lanzador que busca el apolo-mcp de la aplicación instalada y le pasa la conversación. No se conecta a nada ni baja ningún programa. Si Apolo no está instalado, el agente te explica cómo instalarlo.",
          "Instalarlo es cosa de Claude, que lo descarga de GitHub con sus propias condiciones.",
        ],
      },
      {
        titulo: "Esta web",
        parrafos: [
          "apolo.webcafeina.com la sirven GitHub Pages y Cloudflare, que, como cualquier servidor, ven la dirección IP de quien la visita. Solo si lo aceptas, mide visitas con Google Analytics: lo explica la política de cookies.",
        ],
      },
      {
        titulo: "Quién es el responsable",
        parrafos: [
          "Webcafeína, que hace Apolo. Sus datos y cómo ejercer tus derechos están en su política de privacidad, en webcafeina.com/politica-privacidad. Para cualquier duda sobre Apolo: info@webcafeina.com.",
        ],
      },
    ],
  },
  en: {
    titulo: "Privacy · Apolo",
    descripcion: "What data Apolo uses: your images never leave your computer, the app only asks GitHub for updates and the MCP server connects to nothing.",
    etiqueta: "Privacy",
    h1: "Your images never leave your computer.",
    entrada: "What data the app, the command line, the MCP server and the Claude plugin use, and who gets it. Updated on November 3, 2026.",
    secciones: [
      {
        titulo: "The app",
        parrafos: [
          "Apolo works with your images on your computer: it opens them, converts them and saves the results to your disk. It uploads them nowhere, needs no account or sign-up, and has no usage statistics or crash reports.",
          "It connects to one place only: GitHub, once a day, to know whether there's a new version, and to download it if you tell it to install it. Like any download, GitHub sees your IP address. You can turn it off in Settings. If you installed it with a package manager (APT, DNF, Homebrew, Chocolatey…), the package manager looks for updates in its repository.",
        ],
      },
      {
        titulo: "The command line and the MCP server",
        parrafos: [
          "apolo and apolo-mcp connect to nothing: they read the images at the paths you give them and write new files next to them, never overwriting or deleting.",
        ],
      },
      {
        titulo: "With an AI agent",
        parrafos: [
          "When an agent (Claude, Cursor, VS Code…) uses Apolo, what Apolo answers goes into its conversation: paths, file names, dimensions, sizes, quality scores and, if the agent asks to see it, a thumbnail of the image. That conversation is processed by the provider of the model you use, under its own privacy policy. Webcafeína receives nothing.",
        ],
      },
      {
        titulo: "The Claude plugin",
        parrafos: [
          "The Claude plugin only holds a launcher that finds the apolo-mcp of the installed app and hands the conversation over to it. It connects to nothing and downloads no program. If Apolo isn't installed, the agent explains how to install it.",
          "Installing it is up to Claude, which downloads it from GitHub under its own terms.",
        ],
      },
      {
        titulo: "This website",
        parrafos: [
          "apolo.webcafeina.com is served by GitHub Pages and Cloudflare, which, like any server, see the IP address of whoever visits. Only if you accept, it measures visits with Google Analytics: the cookie policy explains it.",
        ],
      },
      {
        titulo: "Who is responsible",
        parrafos: [
          "Webcafeína, who makes Apolo. Its details and how to exercise your rights are in its privacy policy, at webcafeina.com/politica-privacidad. For any question about Apolo: info@webcafeina.com.",
        ],
      },
    ],
  },
};
