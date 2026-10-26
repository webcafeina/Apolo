import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { leerRelease, leerSumas, megas } from "../src/datos/release";

const datos = join(import.meta.dirname, "../src/datos");
const json = JSON.parse(readFileSync(join(datos, "release.json"), "utf8"));
const sumas = readFileSync(join(datos, "sumas.txt"), "utf8");

describe("los ficheros de la versión", () => {
  it("encuentra los 19 que enlaza la web, con su tamaño y su suma", () => {
    const r = leerRelease(json, sumas);
    expect(Object.keys(r.ficheros)).toHaveLength(19);
    expect(r.version).toMatch(/^\d+\.\d+\.\d+$/);
    for (const f of Object.values(r.ficheros)) {
      expect(f.url).toBe(`https://github.com/webcafeina/Apolo/releases/download/v${r.version}/${f.nombre}`);
      expect(f.bytes).toBeGreaterThan(1_000_000);
      expect(f.sha256).toMatch(/^[0-9a-f]{64}$/);
    }
    expect(r.ficheros["dmg"].nombre).toBe(`Apolo_${r.version}_universal.dmg`);
    expect(r.ficheros["mcpb-linux-arm64"].nombre).toBe(`Apolo-${r.version}-linux-arm64.mcpb`);
  });

  it("falla si falta uno: mejor que un enlace roto", () => {
    const sin = { ...json, assets: json.assets.filter((a: { name: string }) => !a.name.endsWith(".dmg")) };
    expect(() => leerRelease(sin, sumas)).toThrow(/dmg/);
  });

  it("lee las sumas, con o sin asterisco", () => {
    const s = leerSumas(`${"a".repeat(64)}  uno.deb\n${"b".repeat(64)} *dos.exe\n`);
    expect(s.get("uno.deb")).toBe("a".repeat(64));
    expect(s.get("dos.exe")).toBe("b".repeat(64));
  });

  it("los tamaños, en megas decimales de cada idioma", () => {
    expect(megas(43690478, "es")).toBe("43,7 MB");
    expect(megas(43690478, "en")).toBe("43.7 MB");
  });
});
