import { describe, expect, it } from "vitest";
import type { ModoTamano } from "./opciones";
import { medidasTras, nueva } from "./tamano";

const con = (modo: ModoTamano, v: number, soloReducir = false) =>
  nueva({ modo, solo_reducir: soloReducir, ...(modo === "lado_mayor" ? { lado_mayor: v } : { porcentaje: v }) });

// La misma tabla que `lado_mayor_y_porcentaje` en crates/nucleo/src/proceso.rs.
describe("el tamaño tras redimensionar", () => {
  it("el lado mayor, apaisada, vertical o cuadrada", () => {
    expect(medidasTras(con("lado_mayor", 1920), 4000, 3000)).toEqual([1920, 1440]);
    expect(medidasTras(con("lado_mayor", 1920), 3000, 4000)).toEqual([1440, 1920]);
    expect(medidasTras(con("lado_mayor", 800), 1000, 1000)).toEqual([800, 800]);
  });

  it("el porcentaje, con su redondeo y como mínimo 1 px", () => {
    expect(medidasTras(con("porcentaje", 50), 4000, 3000)).toEqual([2000, 1500]);
    expect(medidasTras(con("porcentaje", 33.3), 100, 75)).toEqual([33, 25]);
    expect(medidasTras(con("porcentaje", 10), 3, 3)).toEqual([1, 1]);
    expect(medidasTras(con("porcentaje", 150), 128, 128)).toEqual([192, 192]);
  });

  it("sin agrandar, lo que ya cabe se queda", () => {
    expect(medidasTras(con("lado_mayor", 1920, true), 1200, 800)).toEqual([1200, 800]);
    expect(medidasTras(con("porcentaje", 150, true), 128, 128)).toEqual([128, 128]);
    expect(medidasTras(con("lado_mayor", 1920, true), 4000, 3000)).toEqual([1920, 1440]);
  });

  it("ancho y alto, o uno solo en proporción", () => {
    const m = (ancho: number | null, alto: number | null) => nueva({ modo: "medidas", ancho, alto, solo_reducir: false });
    expect(medidasTras(m(50, null), 200, 100)).toEqual([50, 25]);
    expect(medidasTras(m(null, 50), 200, 100)).toEqual([100, 50]);
    expect(medidasTras(m(100, 50), 400, 400)).toEqual([100, 50]);
    // Sin número todavía, nada.
    expect(medidasTras(nueva(), 640, 480)).toEqual([640, 480]);
  });
});
