import { existsSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { RUTAS } from "../src/i18n/rutas";

const PAGINAS = join(import.meta.dirname, "../src/pages");
// «/descargar/mac/» vive en pages/descargar/mac.astro o en pages/descargar/mac/index.astro.
const existe = (ruta: string) =>
  ruta === "/" || ruta === "/en/"
    ? existsSync(join(PAGINAS, ruta, "index.astro"))
    : existsSync(join(PAGINAS, `${ruta.slice(0, -1)}.astro`)) || existsSync(join(PAGINAS, ruta, "index.astro"));

describe("cada página tiene su pareja en el otro idioma", () => {
  it.each(Object.entries(RUTAS))("%s", (_, { es, en }) => {
    expect(es.startsWith("/en/")).toBe(false);
    expect(en.startsWith("/en/")).toBe(true);
    expect(existe(es), es).toBe(true);
    expect(existe(en), en).toBe(true);
  });
});
