// El tamaño de la imagen (ADR 0032): la cuenta y el texto, sin interfaz. La
// cuenta es la de `destino` en crates/nucleo/src/proceso.rs, y las dos se
// prueban con la misma tabla (tamano.test.ts y las pruebas de proceso.rs).

import type { Filtro, ModoTamano, Tamano } from "./opciones";

/** Los atajos de la ventanita: anchos habituales de lado mayor, y porcentajes. */
export const ATAJOS_LADO = [3840, 2560, 1920, 1280, 800] as const;
export const ATAJOS_PORCENTAJE = [75, 50, 25] as const;
export const MODOS: ModoTamano[] = ["lado_mayor", "porcentaje", "medidas"];
export const FILTROS: Filtro[] = ["lanczos3", "mitchell", "catmull_rom", "bilineal", "vecino"];

/** Una redimensión nueva: sin agrandar, con Lanczos 3 y en RGB lineal. */
export function nueva(cambio: Partial<Tamano> = {}): Tamano {
  return {
    modo: "lado_mayor",
    ancho: null,
    alto: null,
    lado_mayor: null,
    porcentaje: null,
    filtro: "lanczos3",
    lineal: true,
    solo_reducir: true,
    ...cambio,
  };
}

/** Las medidas que salen de redimensionar una imagen de `w` × `h`. */
export function medidasTras(r: Tamano, w: number, h: number): [number, number] {
  const prop = (a: number, de: number, aDe: number) => Math.max(1, Math.round((a * aDe) / de));
  let n: [number, number];
  if (r.modo === "lado_mayor" && r.lado_mayor) {
    const l = Math.max(1, r.lado_mayor);
    n = w >= h ? [l, prop(l, w, h)] : [prop(l, h, w), l];
  } else if (r.modo === "porcentaje" && r.porcentaje) {
    const a = (v: number) => Math.max(1, Math.round((v * r.porcentaje!) / 100));
    n = [a(w), a(h)];
  } else if (r.modo === "medidas" && (r.ancho || r.alto)) {
    n = r.ancho && r.alto ? [r.ancho, r.alto] : r.ancho ? [r.ancho, prop(r.ancho, w, h)] : [prop(r.alto!, h, w), r.alto!];
  } else {
    n = [w, h];
  }
  if (r.solo_reducir && (n[0] > w || n[1] > h)) return [w, h];
  return n;
}

/** Si la redimensión no pide nada todavía (el modo sin su número). */
export function vacia(r: Tamano): boolean {
  if (r.modo === "lado_mayor") return !r.lado_mayor;
  if (r.modo === "porcentaje") return !r.porcentaje;
  return !r.ancho && !r.alto;
}

/** Un número con hasta un decimal, con la coma o el punto del idioma. */
export const numero = (n: number, lengua: string) => n.toLocaleString(lengua, { maximumFractionDigits: 1 });

/** La clave i18n y los datos del texto corto de una redimensión, para el
 *  botón de la barra, Lotes y Presets: «1920 px de lado mayor», «Al 50 %»,
 *  «800 × auto px». */
export function textoTamano(r: Tamano, lengua: string): [string, Record<string, string | number>] {
  if (r.modo === "lado_mayor") return ["tamano.texto.ladoMayor", { lado: r.lado_mayor ?? "…" }];
  if (r.modo === "porcentaje") return ["tamano.texto.porcentaje", { porcentaje: r.porcentaje ? numero(r.porcentaje, lengua) : "…" }];
  return ["tamano.texto.medidas", { ancho: r.ancho ?? "auto", alto: r.alto ?? "auto" }];
}
