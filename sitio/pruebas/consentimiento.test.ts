import { describe, expect, it } from "vitest";
import { VERSION, deAnalitica, vigente } from "../src/scripts/consentimiento";

describe("la elección de cookies", () => {
  const ahora = Date.UTC(2026, 9, 26);
  const guardada = (analitica: boolean, hace: number, v = VERSION) => JSON.stringify({ v, analitica, fecha: ahora - hace });

  it("vale doce meses", () => {
    expect(vigente(guardada(true, 30 * 864e5), ahora)?.analitica).toBe(true);
    expect(vigente(guardada(false, 364 * 864e5), ahora)?.analitica).toBe(false);
    expect(vigente(guardada(true, 366 * 864e5), ahora)).toBeNull();
  });

  it("se vuelve a preguntar si cambia la versión o está rota", () => {
    expect(vigente(guardada(true, 0, VERSION + 1), ahora)).toBeNull();
    expect(vigente("{no es json", ahora)).toBeNull();
    expect(vigente(null, ahora)).toBeNull();
  });

  it("solo borra las cookies de Google Analytics", () => {
    expect(deAnalitica("_ga=GA1.1.2; otra=1; _ga_ABC123=GS1; _gid=GA1; _gat=1")).toEqual(["_ga", "_ga_ABC123", "_gid"]);
  });
});
