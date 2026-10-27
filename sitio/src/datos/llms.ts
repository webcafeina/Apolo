// llms.txt y llms-full.txt (ADR 0034): la web en Markdown para las IAs que la
// leen al responder (llmstxt.org). Salen del mismo contenido que las páginas,
// así que no se desfasan: el índice con un enlace por página y lo que es Apolo
// en pocas líneas, y el completo con el texto de las páginas, en los dos
// idiomas (el inglés primero: es el que más leen).
import type { Articulo } from "../contenido/articulos";
import { AGENTES, CLI } from "../contenido/cli-agentes";
import { COMPARATIVAS, INDICE } from "../contenido/comparativas";
import { DESCARGAR, type PaginaDescarga, type Sistema } from "../contenido/descargar";
import { GUIAS } from "../contenido/guias";
import { PREGUNTAS } from "../contenido/preguntas";
import { textos } from "../i18n";
import { type Idioma, type Pagina, ruta } from "../i18n/rutas";
import { release } from "./version";

const IDIOMAS: Idioma[] = ["en", "es"];
const SISTEMAS: Sistema[] = ["macos", "windows", "linux"];
const PAGINA_DE: Record<Sistema, Pagina> = { macos: "descargarMac", windows: "descargarWindows", linux: "descargarLinux" };

const HECHOS: Record<Idioma, string[]> = {
  en: [
    "Free and open source (GPL-3.0), made by Webcafeína (https://webcafeina.com).",
    "Desktop app for macOS 11+ (Apple Silicon and Intel), Windows 10 and 11 (x64 and ARM) and Linux (x64 and ARM: .deb, .rpm, AppImage, APT and RPM repositories).",
    "Writes WebP, AVIF, JPEG XL, JPEG (MozJPEG), PNG (OxiPNG) and QOI; also opens HEIC.",
    "Gives the same file, byte for byte, as the official tools cwebp, cjpeg, oxipng, avifenc and cjxl, with the same options.",
    "Shows the original and the result side by side, with a difference map and the SSIMULACRA 2 score; it can find the lowest quality that reaches a score.",
    "Batches of whole folders in parallel; a command line (`apolo`) and an MCP server for AI agents (Claude, Cursor, VS Code).",
    "Works offline: images never leave the computer. It only asks GitHub for updates.",
  ],
  es: [
    "Gratis y de código abierto (GPL-3.0), hecho por Webcafeína (https://webcafeina.com).",
    "Aplicación de escritorio para macOS 11 o posterior (Apple Silicon e Intel), Windows 10 y 11 (x64 y ARM) y Linux (x64 y ARM: .deb, .rpm, AppImage y repositorios APT y RPM).",
    "Escribe WebP, AVIF, JPEG XL, JPEG (MozJPEG), PNG (OxiPNG) y QOI; también abre HEIC.",
    "Da el mismo fichero, byte a byte, que las herramientas oficiales cwebp, cjpeg, oxipng, avifenc y cjxl, con sus mismas opciones.",
    "Enseña el original y el resultado lado a lado, con un mapa de diferencias y la nota SSIMULACRA 2; puede buscar la calidad más baja que llega a una nota.",
    "Lotes de carpetas enteras en paralelo; línea de comandos (`apolo`) y servidor MCP para agentes de IA (Claude, Cursor, VS Code).",
    "Funciona sin conexión: las imágenes no salen del ordenador. Solo pregunta a GitHub por las actualizaciones.",
  ],
};

const url = (site: URL, pagina: Pagina, idioma: Idioma) => new URL(ruta(pagina, idioma), site).href;

/** El índice: qué es Apolo y un enlace por página, en inglés y en español. */
export function indice(site: URL): string {
  const partes = [`# Apolo\n\n> ${textos("en").sitio.descripcion} ${textos("es").sitio.descripcion}\n`];
  partes.push(`Current version: ${release.version} (${release.fecha.slice(0, 10)}). Source code: https://github.com/webcafeina/Apolo\n`);
  for (const i of IDIOMAS) {
    partes.push(`## ${i === "en" ? "Apolo in brief" : "Apolo, en breve"}\n\n` + HECHOS[i].map((h) => `- ${h}`).join("\n") + "\n");
    const enlace = (p: Pagina, titulo: string, descripcion: string) => `- [${titulo}](${url(site, p, i)}): ${descripcion}`;
    partes.push(
      [
        `## ${i === "en" ? "Pages (English)" : "Páginas (español)"}\n`,
        enlace("inicio", textos(i).sitio.titulo, textos(i).sitio.descripcion),
        ...SISTEMAS.map((s) => enlace(PAGINA_DE[s], DESCARGAR[i][s].titulo, DESCARGAR[i][s].descripcion)),
        enlace("preguntas", PREGUNTAS[i].titulo, PREGUNTAS[i].descripcion),
        enlace("cli", CLI[i].titulo, CLI[i].descripcion),
        enlace("agentes", AGENTES[i].titulo, AGENTES[i].descripcion),
        enlace("novedades", i === "en" ? "What's new" : "Novedades", i === "en" ? "What changes in each version." : "Lo que cambia en cada versión."),
      ].join("\n") + "\n",
    );
    const articulos = (indice: Pagina, cabeza: { titulo: string; descripcion: string; h1: string }, lista: Articulo[]) =>
      [`## ${cabeza.h1.replace(/\.$/, "")}\n`, enlace(indice, cabeza.titulo, cabeza.descripcion), ...lista.map((x) => enlace(x.pagina, x.titulo, x.descripcion))].join("\n") +
      "\n";
    partes.push(articulos("guias", textos(i).articulos.guias, GUIAS[i]));
    partes.push(articulos("comparativas", INDICE[i], COMPARATIVAS[i]));
  }
  partes.push(`## Optional\n\n- [llms-full.txt](${new URL("/llms-full.txt", site).href}): the text of these pages in one file.\n`);
  return partes.join("\n");
}

function seccionesMd(p: PaginaDescarga): string {
  return p.secciones
    .map((s) =>
      [
        `### ${s.titulo}`,
        ...(s.parrafos ?? []),
        ...(s.pasos ?? []).map((x, n) => `${n + 1}. ${x}`),
        ...(s.lista ?? []).map((x) => `- ${x}`),
        ...(s.tabla
          ? [
              [s.tabla.cabeza, s.tabla.cabeza.map(() => "---"), ...s.tabla.filas].map((f) => `| ${f.join(" | ")} |`).join("\n") +
                (s.tabla.nota ? `\n\n${s.tabla.nota}` : ""),
            ]
          : []),
        ...(s.cifras ? [s.cifras.map((c) => `- **${c.valor}**: ${c.texto}`).join("\n")] : []),
        ...(s.barras ? [s.barras.filas.map((f) => `- ${f.etiqueta}: ${f.texto}`).join("\n") + (s.barras.nota ? `\n\n${s.barras.nota}` : "")] : []),
        ...(s.peticiones ? [s.peticiones.map((x) => `- ${x}`).join("\n")] : []),
        ...(s.columnas ?? []).map((c) => `**${c.titulo}**\n\n${c.lista.map((x) => `- ${x}`).join("\n")}`),
        ...(s.codigo ? [`\`\`\`html\n${s.codigo.join("\n")}\n\`\`\``] : []),
        ...(s.comandos ?? []).map((k) => `${k.titulo ? `${k.titulo}:\n\n` : ""}\`\`\`sh\n${k.lineas.join("\n")}\n\`\`\``),
      ].join("\n\n"),
    )
    .join("\n\n");
}

/** El texto de las páginas, en Markdown: inglés y español. */
export function completo(site: URL): string {
  const partes = [indice(site)];
  for (const i of IDIOMAS) {
    partes.push(`# ${i === "en" ? "Apolo, in English" : "Apolo, en español"}`);
    const t = textos(i);
    partes.push(`## ${t.queHace.titulo}\n\n${t.queHace.puntos.map((p) => `- **${p.titulo}.** ${p.texto}`).join("\n")}`);
    for (const s of SISTEMAS) {
      const p = DESCARGAR[i][s];
      partes.push(`## ${p.h1}\n\n${url(site, PAGINA_DE[s], i)}\n\n${p.entrada}\n\n${seccionesMd(p)}`);
    }
    for (const [pagina, p] of [["cli", CLI[i]], ["agentes", AGENTES[i]]] as const) {
      partes.push(`## ${p.h1}\n\n${url(site, pagina, i)}\n\n${p.entrada}\n\n${seccionesMd(p)}`);
    }
    for (const x of [...GUIAS[i], ...COMPARATIVAS[i]]) {
      partes.push(`## ${x.h1}\n\n${url(site, x.pagina, i)}\n\n${x.entrada}\n\n${seccionesMd(x)}`);
    }
    const q = PREGUNTAS[i];
    partes.push(`## ${q.h1}\n\n${url(site, "preguntas", i)}\n\n${q.lista.map((x) => `### ${x.p}\n\n${x.r.join("\n\n")}`).join("\n\n")}`);
  }
  return partes.join("\n\n") + "\n";
}
