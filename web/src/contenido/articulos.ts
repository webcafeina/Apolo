// Las guías y las comparativas (ADR 0034): lo común. Cada una es una página de
// secciones con su fecha, un resumen para el índice y las páginas
// relacionadas. Las cifras salen de src/datos/medidas.json, que mide el propio
// Apolo (herramientas/medidas.mjs): ninguna está escrita a mano.
import medidas from "../datos/medidas.json";
import type { Idioma, Pagina } from "../i18n/rutas";
import type { PaginaDescarga, Seccion } from "./descargar";

export type { Seccion };

export interface Articulo extends PaginaDescarga {
  pagina: Pagina;
  /** Para el índice y las tarjetas: una o dos frases. */
  resumen: string;
  /** Cuándo se escribió y cuándo se revisó (AAAA-MM-DD). */
  publicado: string;
  revisado: string;
  /** La cifra que resume la página: en su cabeza y en su tarjeta del índice. */
  destacado: { valor: string; texto: string };
  /** En las comparativas, la otra herramienta: la cabeza enseña «Apolo frente a …». */
  rival?: string;
  relacionadas: Pagina[];
}

export const M = medidas;

/** Cuánto menos pesa `a` que `b`, en tanto por ciento redondeado (positivo si pesa menos). */
export const menos = (a: number, b: number) => Math.round(100 * (1 - a / b));

/** Bytes en KB o MB, a la manera de cada idioma. */
export function peso(bytes: number, idioma: Idioma): string {
  const n = (x: number, d: number) => x.toLocaleString(idioma === "es" ? "es-ES" : "en-US", { maximumFractionDigits: d });
  return bytes >= 1024 * 1024 ? `${n(bytes / 1024 / 1024, 1)} MB` : `${n(bytes / 1024, 0)} KB`;
}

/** Una fecha AAAA-MM-DD, escrita: «27 de octubre de 2026», «October 27, 2026». */
export const dia = (d: string, idioma: Idioma) =>
  new Date(`${d}T12:00:00Z`).toLocaleDateString(idioma === "es" ? "es-ES" : "en-US", { day: "numeric", month: "long", year: "numeric" });

/** Las 24 fotos de Kodak a una nota: el peso medio de cada formato y lo que ahorra frente a JPEG. */
export function fotos(nota: 80 | 90) {
  const n = M.fotos.notas[nota];
  const media = (b: number) => b / M.fotos.imagenes;
  return {
    avif: { media: media(n.avif), frenteJpeg: menos(n.avif, n.jpeg), frentePng: menos(n.avif, M.fotos.original) },
    jxl: { media: media(n.jxl), frenteJpeg: menos(n.jxl, n.jpeg), frentePng: menos(n.jxl, M.fotos.original) },
    webp: { media: media(n.webp), frenteJpeg: menos(n.webp, n.jpeg), frentePng: menos(n.webp, M.fotos.original) },
    jpeg: { media: media(n.jpeg), frenteJpeg: 0, frentePng: menos(n.jpeg, M.fotos.original) },
  };
}

/** La tabla de las fotos de Kodak a una nota, en cada idioma. */
export function tablaFotos(nota: 80 | 90, idioma: Idioma): Seccion["tabla"] {
  const f = fotos(nota);
  const es = idioma === "es";
  const fila = (nombre: string, x: { media: number; frenteJpeg: number; frentePng: number }) => [
    nombre,
    peso(x.media, idioma),
    x.frenteJpeg === 0 ? "—" : `${x.frenteJpeg > 0 ? "−" : "+"}${Math.abs(x.frenteJpeg)} %`,
    `−${x.frentePng} %`,
  ];
  return {
    cabeza: es ? ["Formato", "Peso medio", "Frente a JPEG", "Frente al PNG"] : ["Format", "Average size", "Against JPEG", "Against the PNG"],
    filas: [fila("AVIF", f.avif), fila("JPEG XL", f.jxl), fila("WebP", f.webp), fila("JPEG (MozJPEG)", f.jpeg)],
    nota: es
      ? `Las 24 fotos del Kodak Lossless True Color Image Suite (${M.fotos.ancho}×${M.fotos.alto}, PNG de ${peso(M.fotos.original / M.fotos.imagenes, idioma)} de media), cada una con la calidad más baja que llega a la nota ${nota} de SSIMULACRA 2. Medido con ${M.version}, el ${dia(M.fecha, idioma)}.`
      : `The 24 photos of the Kodak Lossless True Color Image Suite (${M.fotos.ancho}×${M.fotos.alto}, PNG of ${peso(M.fotos.original / M.fotos.imagenes, idioma)} on average), each at the lowest quality that reaches a SSIMULACRA 2 score of ${nota}. Measured with ${M.version}, on ${dia(M.fecha, idioma)}.`,
  };
}

/** Las fotos de Kodak a una nota, como gráfico de barras: del más ligero al más pesado, el primero en oro. */
export function barrasFotos(nota: 80 | 90, idioma: Idioma): NonNullable<Seccion["barras"]> {
  const f = fotos(nota);
  const es = idioma === "es";
  const filas = (
    [
      ["AVIF", f.avif],
      ["JPEG XL", f.jxl],
      ["WebP", f.webp],
      ["JPEG", f.jpeg],
    ] as const
  )
    .map(([etiqueta, x]) => ({
      etiqueta,
      valor: x.media,
      texto: x.frenteJpeg === 0 ? peso(x.media, idioma) : `${peso(x.media, idioma)} · −${x.frenteJpeg} % ${es ? "frente a JPEG" : "vs JPEG"}`,
    }))
    .sort((a, b) => a.valor - b.valor);
  return { filas, destacar: 0, nota: tablaFotos(nota, idioma)!.nota };
}

/** Capturas y logos: el PNG original, optimizado con OxiPNG y en WebP sin pérdida. */
export function barrasGraficos(idioma: Idioma): NonNullable<Seccion["barras"]> {
  const es = idioma === "es";
  const g = M.graficos;
  return {
    filas: [
      { etiqueta: es ? "PNG original" : "Original PNG", valor: g.original, texto: peso(g.original, idioma) },
      { etiqueta: "OxiPNG", valor: g.png, texto: `${peso(g.png, idioma)} · −${menos(g.png, g.original)} %` },
      { etiqueta: es ? "WebP sin pérdida" : "Lossless WebP", valor: g.webpSinPerdida, texto: `${peso(g.webpSinPerdida, idioma)} · −${menos(g.webpSinPerdida, g.original)} %` },
    ],
    destacar: 2,
    nota: es
      ? "Tres capturas de pantalla y tres logos con transparencia, los mismos píxeles en los tres casos."
      : "Three screenshots and three logos with transparency, the same pixels in all three cases.",
  };
}

/** La foto HEIC de ejemplo de libheif, a JPEG, WebP y AVIF con la nota 80. */
export function barrasHeic(idioma: Idioma): NonNullable<Seccion["barras"]> {
  const h = M.heic;
  const fila = (etiqueta: string, b: number) => ({ etiqueta, valor: b, texto: `${peso(b, idioma)} · −${menos(b, h.original)} %` });
  return {
    filas: [{ etiqueta: "HEIC", valor: h.original, texto: peso(h.original, idioma) }, fila("JPEG", h.jpeg), fila("WebP", h.webp), fila("AVIF", h.avif)],
    destacar: 3,
    nota:
      idioma === "es"
        ? "La foto de ejemplo de libheif, a una nota de 80 de SSIMULACRA 2 frente al HEIC abierto."
        : "libheif's sample photo, at a SSIMULACRA 2 score of 80 against the opened HEIC.",
  };
}

/** «AVIF: en los navegadores actuales…» → una tarjeta con «AVIF» de título y el resto de texto. */
export function tarjetas(lista: string[]): NonNullable<Seccion["columnas"]> {
  return lista.map((x) => {
    const corte = x.indexOf(": ");
    const resto = x.slice(corte + 2);
    return { titulo: x.slice(0, corte), lista: [resto.charAt(0).toUpperCase() + resto.slice(1)] };
  });
}
