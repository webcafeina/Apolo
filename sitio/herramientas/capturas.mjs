// Las capturas de Apolo para la web (ADR 0033), pasadas por el propio Apolo:
// `node herramientas/capturas.mjs`, después de sacarlas con `make capturas-web`
// (frontend/capturas/…). Necesita target/release/apolo (`make cli`).
//
// Cada vista (macOS, Windows, GNOME, KDE) y pantalla, en claro y oscuro y en
// los dos idiomas si están, va a public/img/<vista>/[en/]<pantalla>-<tema>.webp,
// y sus medidas a src/datos/capturas.json. Las de inglés que falten se toman de
// las de español (y el JSON lo dice).
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const CAPTURAS = resolve(SITIO, "../frontend/capturas");
const APOLO = resolve(SITIO, "../target/release/apolo");

// Cada vista de la web, y la carpeta de capturas.spec.ts de la que sale.
export const VISTAS = { macos: "macos-vidrio", windows: "windows-vidrio", gnome: "linux-propia", kde: "linux-kde-propia" };
// Cada pantalla de la web, y la captura de la que sale.
export const PANTALLAS = { estudio: "estudio", diferencias: "mapa", lotes: "lotes-resumen", agentes: "agentes" };
const TEMAS = ["claro", "oscuro"];

if (!existsSync(APOLO)) throw new Error(`Falta ${APOLO}: make cli`);
const apolo = (...a) => {
  const r = spawnSync(APOLO, a, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`apolo ${a.join(" ")}:\n${r.stderr}`);
};
const medidas = (png) => {
  const b = readFileSync(png);
  return [b.readUInt32BE(16), b.readUInt32BE(20)];
};

const datos = { vistas: {} };
rmSync(join(SITIO, "public/img"), { recursive: true, force: true });
for (const [vista, carpeta] of Object.entries(VISTAS)) {
  datos.vistas[vista] = {};
  for (const [pantalla, captura] of Object.entries(PANTALLAS)) {
    const p = {};
    for (const idioma of ["es", "en"]) {
      const dir = idioma === "es" ? join(CAPTURAS, carpeta) : join(CAPTURAS, carpeta, "en");
      const propia = TEMAS.every((t) => existsSync(join(dir, `${captura}-${t}.png`)));
      const origen = propia ? dir : join(CAPTURAS, carpeta);
      const salida = join(SITIO, "public/img", vista, idioma === "es" ? "" : "en");
      mkdirSync(salida, { recursive: true });
      for (const tema of TEMAS) {
        const png = join(origen, `${captura}-${tema}.png`);
        if (!existsSync(png)) throw new Error(`Falta ${png}: make capturas-web`);
        apolo("webp", "-q", "85", "-m", "6", "-quiet", png, "-o", join(salida, `${pantalla}-${tema}.webp`));
      }
      const [ancho, alto] = medidas(join(origen, `${captura}-claro.png`));
      p[idioma] = { ancho, alto, propia, bytes: statSync(join(salida, `${pantalla}-claro.webp`)).size };
    }
    datos.vistas[vista][pantalla] = p;
  }
}
writeFileSync(join(SITIO, "src/datos/capturas.json"), JSON.stringify(datos, null, 2) + "\n");
console.log("capturas.json escrito");
