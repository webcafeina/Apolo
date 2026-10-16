// Recortar con el ratón en el Estudio (lo pidió el cliente el 2026-10-15): la
// cuenta del marco, sin interfaz. Todo en píxeles de la imagen ya enderezada,
// que es la que recibe el proceso del núcleo (ADR 0020).

import type { Recorte } from "./opciones";

/** De dónde se tira: el marco entero o una de sus ocho asas. */
export type Asa = "mover" | "n" | "s" | "e" | "o" | "ne" | "no" | "se" | "so";

export const ASAS: Exclude<Asa, "mover">[] = ["no", "n", "ne", "e", "se", "s", "so", "o"];

/** Las proporciones del selector. `original` es la de la imagen. */
export const PROPORCIONES = ["libre", "original", "1:1", "4:3", "3:2", "16:9", "4:5"] as const;
export type Proporcion = (typeof PROPORCIONES)[number];

/** Ancho entre alto, o null si es libre. */
export function razon(p: Proporcion, ancho: number, alto: number): number | null {
  if (p === "libre") return null;
  if (p === "original") return ancho / alto;
  const [a, b] = p.split(":").map(Number);
  return a / b;
}

const entre = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v));

/** El marco que sale de tirar de `asa` desde `r0` (dx, dy en píxeles de la
 *  imagen), dentro de una imagen de W × H, con la proporción `prop` si la hay. */
export function aplicarAsa(asa: Asa, r0: Recorte, dx: number, dy: number, W: number, H: number, prop: number | null): Recorte {
  if (asa === "mover") {
    return {
      x: Math.round(entre(r0.x + dx, 0, W - r0.ancho)),
      y: Math.round(entre(r0.y + dy, 0, H - r0.alto)),
      ancho: r0.ancho,
      alto: r0.alto,
    };
  }
  let x1 = r0.x;
  let y1 = r0.y;
  let x2 = r0.x + r0.ancho;
  let y2 = r0.y + r0.alto;
  const oeste = asa === "o" || asa === "no" || asa === "so";
  const este = asa === "e" || asa === "ne" || asa === "se";
  const norte = asa === "n" || asa === "ne" || asa === "no";
  const sur = asa === "s" || asa === "se" || asa === "so";
  if (oeste) x1 = entre(x1 + dx, 0, x2 - 1);
  if (este) x2 = entre(x2 + dx, x1 + 1, W);
  if (norte) y1 = entre(y1 + dy, 0, y2 - 1);
  if (sur) y2 = entre(y2 + dy, y1 + 1, H);
  if (!prop) return redondo(x1, y1, x2 - x1, y2 - y1);

  // Con proporción: manda la dimensión de la que se tira (en una esquina, el
  // ancho), y el marco crece desde el lado contrario, que queda fijo. Por un
  // lado, se reparte por igual a los dos costados.
  let w = x2 - x1;
  let h = y2 - y1;
  if (norte && !este && !oeste) w = h * prop;
  else if (sur && !este && !oeste) w = h * prop;
  else h = w / prop;
  // Lo que cabe, según de dónde crece.
  const cx = (r0.x * 2 + r0.ancho) / 2;
  const cy = (r0.y * 2 + r0.alto) / 2;
  const maxW = oeste ? x2 : este ? W - x1 : 2 * Math.min(cx, W - cx);
  const maxH = norte ? y2 : sur ? H - y1 : 2 * Math.min(cy, H - cy);
  if (w > maxW) [w, h] = [maxW, maxW / prop];
  if (h > maxH) [w, h] = [maxH * prop, maxH];
  const x = oeste ? x2 - w : este ? x1 : cx - w / 2;
  const y = norte ? y2 - h : sur ? y1 : cy - h / 2;
  return redondo(x, y, w, h);
}

function redondo(x: number, y: number, w: number, h: number): Recorte {
  return { x: Math.round(x), y: Math.round(y), ancho: Math.max(1, Math.round(w)), alto: Math.max(1, Math.round(h)) };
}

/** El marco más grande con la proporción `prop`, centrado dentro de `r`. */
export function conProporcion(r: Recorte, prop: number | null): Recorte {
  if (!prop) return r;
  let w = r.ancho;
  let h = r.alto;
  if (w / h > prop) w = h * prop;
  else h = w / prop;
  return redondo(r.x + (r.ancho - w) / 2, r.y + (r.alto - h) / 2, w, h);
}

/** El marco dentro de la imagen (lo que se salga, fuera), o null si no queda nada. */
export function dentro(r: Recorte, W: number, H: number): Recorte | null {
  const x = entre(r.x, 0, W);
  const y = entre(r.y, 0, H);
  const ancho = Math.min(r.ancho, W - x);
  const alto = Math.min(r.alto, H - y);
  return ancho > 0 && alto > 0 ? { x, y, ancho, alto } : null;
}

/** Los píxeles de `img` dentro del marco: el original recortado, para
 *  compararlo con un resultado recortado a la misma proporción. */
export function recortar(img: ImageData, r: Recorte): ImageData {
  const d = dentro(r, img.width, img.height);
  if (!d) return img;
  const fuera = new Uint8ClampedArray(d.ancho * d.alto * 4);
  for (let fila = 0; fila < d.alto; fila++) {
    const desde = ((d.y + fila) * img.width + d.x) * 4;
    fuera.set(img.data.subarray(desde, desde + d.ancho * 4), fila * d.ancho * 4);
  }
  return new ImageData(fuera, d.ancho, d.alto);
}
