import { describe, expect, it } from "vitest";
import { aplicarAsa, conProporcion, dentro, razon, recortar } from "./recorte";

const r0 = { x: 100, y: 100, ancho: 200, alto: 100 };

describe("el marco del recorte", () => {
  it("se mueve sin salirse de la imagen", () => {
    expect(aplicarAsa("mover", r0, 50, 20, 1000, 800, null)).toEqual({ x: 150, y: 120, ancho: 200, alto: 100 });
    expect(aplicarAsa("mover", r0, 5000, -5000, 1000, 800, null)).toEqual({ x: 800, y: 0, ancho: 200, alto: 100 });
  });

  it("crece y encoge desde cada asa, sin pasar del borde", () => {
    expect(aplicarAsa("se", r0, 40, 30, 1000, 800, null)).toEqual({ x: 100, y: 100, ancho: 240, alto: 130 });
    expect(aplicarAsa("no", r0, -40, -30, 1000, 800, null)).toEqual({ x: 60, y: 70, ancho: 240, alto: 130 });
    expect(aplicarAsa("e", r0, 9000, 0, 1000, 800, null)).toEqual({ x: 100, y: 100, ancho: 900, alto: 100 });
    // Nunca menos de un píxel, aunque se tire de más.
    expect(aplicarAsa("o", r0, 9000, 0, 1000, 800, null).ancho).toBe(1);
  });

  it("mantiene la proporción, desde el lado contrario", () => {
    const r = aplicarAsa("se", { x: 0, y: 0, ancho: 100, alto: 100 }, 100, 0, 1000, 800, 1);
    expect(r).toEqual({ x: 0, y: 0, ancho: 200, alto: 200 });
    // Si no cabe, lo que cabe con esa proporción.
    const grande = aplicarAsa("se", { x: 0, y: 700, ancho: 50, alto: 50 }, 500, 0, 1000, 800, 1);
    expect(grande).toEqual({ x: 0, y: 700, ancho: 100, alto: 100 });
    expect(razon("16:9", 1, 1)).toBeCloseTo(16 / 9);
    expect(razon("original", 1200, 800)).toBeCloseTo(1.5);
    expect(razon("libre", 1, 1)).toBeNull();
  });

  it("al elegir una proporción, el marco más grande que cabe, centrado", () => {
    expect(conProporcion({ x: 0, y: 0, ancho: 400, alto: 200 }, 1)).toEqual({ x: 100, y: 0, ancho: 200, alto: 200 });
    expect(dentro({ x: 900, y: 0, ancho: 300, alto: 10 }, 1000, 800)).toEqual({ x: 900, y: 0, ancho: 100, alto: 10 });
    expect(dentro({ x: 2000, y: 0, ancho: 10, alto: 10 }, 1000, 800)).toBeNull();
  });

  it("recorta los píxeles del original", () => {
    const datos = new Uint8ClampedArray(4 * 3 * 4);
    for (let i = 0; i < 12; i++) datos[i * 4] = i; // el rojo dice qué píxel es
    const img = { data: datos, width: 4, height: 3 } as ImageData;
    globalThis.ImageData ??= class {
      constructor(
        public data: Uint8ClampedArray,
        public width: number,
        public height: number,
      ) {}
    } as never;
    const c = recortar(img, { x: 1, y: 1, ancho: 2, alto: 2 });
    expect([c.width, c.height]).toEqual([2, 2]);
    expect([0, 1, 2, 3].map((i) => c.data[i * 4])).toEqual([5, 6, 9, 10]);
  });
});
