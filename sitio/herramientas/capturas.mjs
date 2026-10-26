// Las capturas de Apolo para la web (ADR 0033), pasadas por el propio Apolo:
// `node herramientas/capturas.mjs`, que `make capturas-sitio` corre después de
// sacarlas (frontend/capturas/…, a doble densidad). Necesita target/release/apolo.
//
// Cada vista (macOS, Windows, GNOME, KDE), pantalla, tema e idioma sale en AVIF a
// 1280 y 2560 px de ancho, y en WebP a 1280 (para los navegadores sin AVIF, que
// son pocos: a 2560 pesaba 19 MB más), buscando la nota 90 de SSIMULACRA 2 (con
// texto, menos se nota). Van a public/img/<vista>/[en/]<pantalla>-<tema>-<ancho>.<formato>,
// con las medidas y los pesos en src/datos/capturas.json. Si falta la de un
// idioma, se toma la del otro y el JSON lo dice.
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const CAPTURAS = resolve(SITIO, "../frontend/capturas");
const APOLO = resolve(SITIO, "../target/release/apolo");
const NOTA = "90";

// Cada vista de la web, y la carpeta de capturas.spec.ts de la que sale.
const VISTAS = { macos: "macos-vidrio", windows: "windows-vidrio", gnome: "linux-propia", kde: "linux-kde-propia" };
// Cada pantalla de la web, y la captura de la que sale.
const PANTALLAS = { estudio: "estudio", diferencias: "mapa", lotes: "lotes-resumen", agentes: "agentes" };
const TEMAS = ["claro", "oscuro"];
const ANCHOS = [1280, 2560];

if (!existsSync(APOLO)) throw new Error(`Falta ${APOLO}: make cli`);
const apolo = (...a) => {
  const r = spawnSync(APOLO, a, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`apolo ${a.join(" ")}:\n${r.stderr}`);
};
const medidas = (png) => {
  const b = readFileSync(png);
  return [b.readUInt32BE(16), b.readUInt32BE(20)];
};

const datos = { nota: Number(NOTA), anchos: ANCHOS, vistas: {} };
rmSync(join(SITIO, "public/img"), { recursive: true, force: true });
for (const [vista, carpeta] of Object.entries(VISTAS)) {
  datos.vistas[vista] = {};
  for (const idioma of ["es", "en"]) {
    // Las de esta vista e idioma, con su nombre de la web, en una carpeta aparte para el lote.
    const tmp = join(tmpdir(), `apolo-capturas-${vista}-${idioma}`);
    rmSync(tmp, { recursive: true, force: true });
    mkdirSync(join(tmp, "png"), { recursive: true });
    const propias = {};
    for (const [pantalla, captura] of Object.entries(PANTALLAS)) {
      const dir = idioma === "es" ? join(CAPTURAS, carpeta) : join(CAPTURAS, carpeta, "en");
      propias[pantalla] = TEMAS.every((t) => existsSync(join(dir, `${captura}-${t}.png`)));
      const origen = propias[pantalla] ? dir : join(CAPTURAS, carpeta);
      for (const tema of TEMAS) {
        const png = join(origen, `${captura}-${tema}.png`);
        if (!existsSync(png)) throw new Error(`Falta ${png}: make capturas-sitio`);
        copyFileSync(png, join(tmp, "png", `${pantalla}-${tema}.png`));
      }
    }
    // Las dos medidas, en paralelo, con la nota buscada en cada imagen.
    for (const ancho of ANCHOS) {
      const formatos = ancho === 1280 ? "avif,webp" : "avif";
      apolo("lote", join(tmp, "png"), "--formato", formatos, "--objetivo", NOTA, "--lado-mayor", String(ancho), "-s", join(tmp, String(ancho)));
    }
    const salida = join(SITIO, "public/img", vista, idioma === "es" ? "" : "en");
    mkdirSync(salida, { recursive: true });
    for (const pantalla of Object.keys(PANTALLAS)) {
      const [ancho, alto] = medidas(join(tmp, "png", `${pantalla}-claro.png`));
      const pesos = {};
      for (const tema of TEMAS) {
        for (const a of ANCHOS) {
          for (const formato of a === 1280 ? ["avif", "webp"] : ["avif"]) {
            const nombre = `${pantalla}-${tema}-${a}.${formato}`;
            copyFileSync(join(tmp, String(a), `${pantalla}-${tema}.${formato}`), join(salida, nombre));
            pesos[nombre] = statSync(join(salida, nombre)).size;
          }
        }
      }
      (datos.vistas[vista][pantalla] ??= {})[idioma] = { ancho, alto, propia: propias[pantalla], pesos };
    }
    rmSync(tmp, { recursive: true, force: true });
    console.log(`${vista} ${idioma}`);
  }
}
writeFileSync(join(SITIO, "src/datos/capturas.json"), JSON.stringify(datos, null, 2) + "\n");
console.log("capturas.json escrito");
