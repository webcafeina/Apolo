// Vigila la ADR 0007: toda clave que se pide en un componente existe en
// es.json, y ningún texto está vacío.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import es from "./es.json";

const raiz = join(__dirname, "..");

function ficheros(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const ruta = join(dir, f);
    if (statSync(ruta).isDirectory()) return ficheros(ruta);
    return /\.tsx?$/.test(f) && !f.endsWith(".test.ts") ? [ruta] : [];
  });
}

function buscar(obj: unknown, clave: string): unknown {
  return clave.split(".").reduce<unknown>((o, k) => (o as Record<string, unknown> | undefined)?.[k], obj);
}

function hojas(obj: object, prefijo = ""): [string, unknown][] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === "object" && v !== null ? hojas(v, `${prefijo}${k}.`) : [[`${prefijo}${k}`, v]],
  );
}

describe("i18n", () => {
  it("cada t(\"…\") del código existe en es.json", () => {
    const faltan: string[] = [];
    for (const f of ficheros(raiz)) {
      for (const m of readFileSync(f, "utf8").matchAll(/\bt\("([\w.]+)"\)/g)) {
        if (typeof buscar(es, m[1]) !== "string") faltan.push(`${m[1]} (${f})`);
      }
    }
    expect(faltan).toEqual([]);
  });

  it("ningún texto está vacío", () => {
    const vacios = hojas(es).filter(([, v]) => typeof v !== "string" || v.trim() === "");
    expect(vacios).toEqual([]);
  });
});
