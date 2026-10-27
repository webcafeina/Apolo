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
