// Las cifras de las guías (ADR 0034), medidas con el propio Apolo:
// `node herramientas/medidas.mjs` (necesita target/release/apolo, `make cli`,
// y red la primera vez). Escribe src/datos/medidas.json, que se sube a git.
//
// Fotos: las 24 del Kodak Lossless True Color Image Suite (768×512, PNG sin
// pérdida; Kodak las publicó para uso sin restricciones), convertidas a AVIF,
// JPEG XL, WebP y JPEG buscando, foto a foto, la calidad más baja que da la
// nota 80 de SSIMULACRA 2 («no se distingue puesta al lado») y la 90 («no se
// distingue ni de cerca»). Capturas, logos y el HEIC del iPhone: los de la
// carpeta de prueba de Lotes (pruebas/lotes/generar.sh).
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const RAIZ = resolve(SITIO, "..");
const APOLO = join(RAIZ, "target/release/apolo");
const CACHE = join(process.env.XDG_CACHE_HOME ?? join(process.env.HOME, ".cache"), "apolo-guias");
const KODAK = join(CACHE, "kodak");
const LOTES = join(CACHE, "lotes");

if (!existsSync(APOLO)) throw new Error(`Falta ${APOLO}: make cli`);
mkdirSync(KODAK, { recursive: true });
for (let i = 1; i <= 24; i++) {
  const n = `kodim${String(i).padStart(2, "0")}.png`;
  if (!existsSync(join(KODAK, n))) {
    const r = spawnSync("curl", ["-fsS", "-o", join(KODAK, n), `https://r0k.us/graphics/kodak/kodak/${n}`]);
    if (r.status !== 0) throw new Error(`No se pudo bajar ${n}`);
  }
}
if (!existsSync(join(LOTES, "apolo-pruebas-lotes", "vacaciones"))) {
  const r = spawnSync("bash", [join(RAIZ, "pruebas/lotes/generar.sh"), LOTES], { stdio: "inherit" });
  if (r.status !== 0) throw new Error("generar.sh falló");
}

// Las opciones de Apolo, delante de «--»: lo de detrás es de la herramienta (cwebp…).
const apolo = (...a) => {
  const corte = a.includes("--") ? a.indexOf("--") : a.length;
  const r = spawnSync(APOLO, [...a.slice(0, corte), "-q", "--idioma", "es", ...a.slice(corte)], { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`apolo ${a.join(" ")}\n${r.stderr}`);
};
const bytes = (dir, filtro = () => true) =>
  readdirSync(dir, { recursive: true })
    .map((f) => join(dir, f))
    .filter((f) => statSync(f).isFile() && filtro(f));
const suma = (fs) => fs.reduce((s, f) => s + statSync(f).size, 0);
const EXT = { avif: "avif", jxl: "jxl", webp: "webp", jpeg: "jpg" };

const original = suma(bytes(KODAK));
const fotos = { imagenes: 24, ancho: 768, alto: 512, original, notas: {} };
for (const nota of [80, 90]) {
  const salida = join(CACHE, `n${nota}`);
  rmSync(salida, { recursive: true, force: true });
  apolo("lote", KODAK, "--formato", "avif,jxl,webp,jpeg", "--objetivo", String(nota), "-s", salida);
  fotos.notas[nota] = Object.fromEntries(Object.entries(EXT).map(([f, e]) => [f, suma(bytes(salida, (x) => x.endsWith(`.${e}`)))]));
  console.log(`nota ${nota}:`, fotos.notas[nota]);
}

// generar.sh deja la carpeta como apolo-pruebas-lotes/ dentro del destino.
const v = join(LOTES, "apolo-pruebas-lotes", "vacaciones");
const heicSalida = join(CACHE, "heic");
rmSync(heicSalida, { recursive: true, force: true });
apolo("lote", join(v, "iphone"), "--formato", "jpeg,webp,avif", "--objetivo", "80", "-s", heicSalida);
const heic = {
  original: suma(bytes(join(v, "iphone"))),
  ...Object.fromEntries(["jpeg", "webp", "avif"].map((f) => [f, suma(bytes(heicSalida, (x) => x.endsWith(`.${EXT[f]}`)))])),
};

const graficos = {};
for (const [nombre, formato, extra] of [
  ["png", "png", []],
  ["webpSinPerdida", "webp", ["--", "-lossless", "-z", "9"]],
]) {
  const salida = join(CACHE, `graficos-${nombre}`);
  rmSync(salida, { recursive: true, force: true });
  apolo("lote", join(v, "capturas"), join(v, "logos"), "--formato", formato, "-s", salida, ...extra);
  graficos[nombre] = suma(bytes(salida));
}
graficos.original = suma(bytes(join(v, "capturas"))) + suma(bytes(join(v, "logos")));

const version = spawnSync(APOLO, ["--version"], { encoding: "utf8" }).stdout.split("\n")[0];
const fecha = new Date().toISOString().slice(0, 10);
writeFileSync(join(SITIO, "src/datos/medidas.json"), JSON.stringify({ version, fecha, fotos, heic, graficos }, null, 2) + "\n");
console.log("src/datos/medidas.json");
