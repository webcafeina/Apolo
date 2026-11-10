// Lo que se pide al vectorizar y al exportar (ADR 0039): el `AjusteVector` del núcleo
// (crates/nucleo/src/vector.rs), en JSON. Cada opción está en docs/cobertura-vectorizar.md.

export type TipoImagen = "auto" | "logotipo" | "ilustracion" | "by_n" | "pixel_art" | "foto";
export type Estilo = "rellenar" | "trazar_formas" | "trazar_bordes";
export type NivelDxf = "rectas" | "arcos" | "splines";
export type FormatoVector = "svg" | "pdf" | "eps" | "dxf";

export const TIPOS: TipoImagen[] = ["auto", "logotipo", "ilustracion", "by_n", "pixel_art", "foto"];
export const ESTILOS: Estilo[] = ["rellenar", "trazar_formas", "trazar_bordes"];
export const NIVELES_DXF: NivelDxf[] = ["rectas", "arcos", "splines"];
export const FORMATOS_VECTOR: FormatoVector[] = ["svg", "pdf", "eps", "dxf"];

export interface AjusteVector {
  tipo: TipoImagen;
  /** Colores como mucho (null: los que tenga). Con un máximo o una paleta, todo plano. */
  colores: number | null;
  /** Paleta fija, `#rrggbb`. */
  paleta: string[];
  /** 0–100 (75). Con objetivo, el mínimo de la búsqueda. */
  detalle: number;
  /** La nota SSIMULACRA 2 que buscar: el dibujo más ligero que llega. */
  objetivo: number | null;
  degradados: boolean;
  area_minima: number;
  estilo: Estilo;
  grosor: number;
  color_trazo: string | null;
  nivel_dxf: NivelDxf;
}

export const AJUSTE_INICIAL: AjusteVector = {
  tipo: "auto",
  colores: null,
  paleta: [],
  detalle: 75,
  objetivo: null,
  degradados: true,
  area_minima: 0.8,
  estilo: "rellenar",
  grosor: 1,
  color_trazo: null,
  nivel_dxf: "splines",
};

/** Lo que cambia el dibujo (y obliga a volver a vectorizar); el resto es de exportar. */
export const parteDibujo = (a: AjusteVector) => ({
  tipo: a.tipo,
  colores: a.colores,
  paleta: a.paleta,
  detalle: a.detalle,
  objetivo: a.objetivo,
  degradados: a.degradados,
  area_minima: a.area_minima,
});

/** `#rrggbb`, o `null` si no lo es. */
export const colorValido = (t: string): string | null => {
  const s = t.trim();
  const c = s.startsWith("#") ? s : `#${s}`;
  return /^#[0-9a-fA-F]{6}$/.test(c) ? c.toLowerCase() : null;
};

/** La paleta escrita a mano («#1e46c8, #ffffff»): los colores válidos, y si sobra algo que no lo es. */
export function leerPaleta(texto: string): { colores: string[]; malos: string[] } {
  const partes = texto
    .split(/[\s,;]+/)
    .map((x) => x.trim())
    .filter(Boolean);
  const colores: string[] = [];
  const malos: string[] = [];
  for (const p of partes) {
    const c = colorValido(p);
    if (c) colores.push(c);
    else malos.push(p);
  }
  return { colores, malos };
}
