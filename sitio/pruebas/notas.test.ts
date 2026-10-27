import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { compararVersiones, enlaceSeguro, leerNotas, recortar } from "../src/datos/notas";

const NOTAS = join(import.meta.dirname, "../../.github/notas");

describe("las novedades", () => {
  it("compara versiones por números, no por letras", () => {
    expect(compararVersiones("0.10.0", "0.9.1")).toBeGreaterThan(0);
    expect(compararVersiones("0.18.3", "0.18.3")).toBe(0);
  });

  it("recorta cada idioma a lo que cambia", () => {
    const md = "**Lema.** Llega.\n\n*English below.*\n\n## Qué cambia\n\n- Uno.\n\n## Descargas\n\n| a |\n\n---\n\n## In English\n\n**Motto.**\n\n- One.\n\n**Downloads**: the same.\n";
    const { es, en } = recortar(md);
    expect(es).toBe("**Lema.** Llega.\n\n\n\n- Uno.");
    expect(en).toBe("**Motto.**\n\n- One.");
  });

  it("lo que traiga una nota no se ejecuta: HTML escapado y enlaces raros en texto", () => {
    const dir = mkdtempSync(join(tmpdir(), "notas-"));
    writeFileSync(
      join(dir, "v0.1.0.md"),
      '<script>alert(1)</script>\n\nTexto <img src=x onerror="alert(1)"> y [malo](javascript:alert(1)), [bueno](https://apolo.webcafeina.com/).\n',
    );
    const [n] = leerNotas(dir, "0.1.0");
    expect(n.es).not.toMatch(/<script|<img|javascript:/i);
    expect(n.es).toContain("&lt;script&gt;");
    expect(n.es).toContain("malo");
    expect(n.es).toContain('<a href="https://apolo.webcafeina.com/">bueno</a>');
    expect(enlaceSeguro("JavaScript:alert(1)")).toBe(false);
    expect(enlaceSeguro("data:text/html,x")).toBe(false);
    expect(enlaceSeguro("/descargar/mac/")).toBe(true);
  });

  it("las notas de verdad: hasta la versión publicada, de la más nueva a la más vieja", () => {
    const n = leerNotas(NOTAS, "0.18.3");
    expect(n[0].version).toBe("0.18.3");
    expect(n.at(-1)?.version).toBe("0.2.0");
    expect(n.every((x) => !x.es.includes("Descargas") && !x.es.includes("SHA256SUMS"))).toBe(true);
    expect(n.find((x) => x.version === "0.18.3")?.en).toContain("macOS");
    expect(n.find((x) => x.version === "0.5.0")?.en).toBeNull();
    expect(leerNotas(NOTAS, "0.9.0").some((x) => x.version === "0.18.3")).toBe(false);
  });
});
