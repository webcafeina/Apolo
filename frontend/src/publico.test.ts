// Lo que ve el público, vigilado (ADR 0033): Squoosh no se nombra en ningún
// texto que llegue a la gente, y la descripción del paquete de Linux va en
// ASCII, que el Centro de aplicaciones de Ubuntu cambia las tildes por «?»
// (docs/trampas.md). También los README y toda la web nueva (sitio/src).
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const RAIZ = join(import.meta.dirname, "../..");
const de = (carpeta: string, filtro: (f: string) => boolean = () => true) =>
  readdirSync(join(RAIZ, carpeta))
    .filter(filtro)
    .map((f) => join(carpeta, f));

// Todos los ficheros de una carpeta, con las subcarpetas.
const todos = (carpeta: string): string[] =>
  readdirSync(join(RAIZ, carpeta)).flatMap((f) =>
    statSync(join(RAIZ, carpeta, f)).isDirectory() ? todos(join(carpeta, f)) : [join(carpeta, f)],
  );

const PUBLICOS = [
  "README.md",
  "README.en.md",
  ...todos("sitio/src"),
  ...de("frontend/src/i18n", (f) => f.endsWith(".json")),
  ...de("src-tauri", (f) => /^tauri.*\.json$/.test(f)),
  ...de("empaquetado/linux"),
  "empaquetado/aur/PKGBUILD",
  ...de("empaquetado/homebrew", (f) => f.endsWith(".rb")),
  "empaquetado/macos/fondo-dmg.svg",
  ...de(".github/notas", (f) => f.endsWith(".md")),
];

describe("lo que ve el público", () => {
  it.each(PUBLICOS)("%s no nombra a Squoosh", (f) => {
    expect(readFileSync(join(RAIZ, f), "utf8").toLowerCase()).not.toContain("squoosh");
  });

  it("la descripción del paquete de Linux va en ASCII", () => {
    const { bundle } = JSON.parse(readFileSync(join(RAIZ, "src-tauri/tauri.linux.conf.json"), "utf8"));
    expect(bundle.shortDescription).toMatch(/^[\x20-\x7e]+$/);
    expect(bundle.longDescription).toMatch(/^[\x20-\x7e]+$/);
  });
});
