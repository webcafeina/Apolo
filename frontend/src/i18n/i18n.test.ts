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

describe("i18n de los controles", () => {
  it("cada control tiene etiqueta, explicación y sus valores", async () => {
    const { CONTROLES } = await import("../estudio/opciones");
    const faltan: string[] = [];
    for (const c of CONTROLES) {
      for (const k of [`opcion.${c.clave}`, `opcion.${c.clave}Ayuda`]) if (typeof buscar(es, k) !== "string") faltan.push(k);
      // Un porQue que depende del ajuste se prueba en sus dos casos: sin
      // pérdida y con nota objetivo, que es de lo que dependen.
      const casos = [
        { avif: { sin_perdida: true }, objetivo: null },
        { avif: { sin_perdida: false }, objetivo: 80 },
      ] as unknown as Parameters<Extract<typeof c.porQue, (a: never) => string>>[0][];
      const porQues = typeof c.porQue === "function" ? casos.map((x) => (c.porQue as (a: unknown) => string)(x)) : c.porQue ? [c.porQue] : [];
      for (const k of porQues) if (typeof buscar(es, k) !== "string") faltan.push(k);
      if (c.tipo === "lista")
        for (const v of c.valores) if (typeof buscar(es, `valor.${c.clave}.${v}`) !== "string") faltan.push(`valor.${c.clave}.${v}`);
    }
    expect(faltan).toEqual([]);
  });

  it("los presets de cwebp, los niveles y los formatos tienen nombre", () => {
    const claves = [
      ...["default", "photo", "picture", "drawing", "icon", "text"].map((p) => `presetCwebp.${p}`),
      ...["basico", "avanzado", "experto"].map((n) => `nivel.${n}`),
      ...["avif", "jxl", "mozjpeg", "oxipng", "qoi"].map((f) => `formato.${f}`),
      ...["deslizador", "ladoALado"].map((m) => `comparador.${m}`),
      ...["x", "y", "ancho", "alto", "exif", "icc", "xmp"].map((k) => `panel.${k}`),
    ];
    expect(claves.filter((k) => typeof buscar(es, k) !== "string")).toEqual([]);
  });
});
