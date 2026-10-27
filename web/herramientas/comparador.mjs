// Las fotos del comparador de la portada (ADR 0033), hechas con el propio
// Apolo: `node herramientas/comparador.mjs` (necesita target/release/apolo,
// `make cli`, y red para bajar las fotos la primera vez).
//
// Cada foto se baja de su origen (CC0 o dominio público), se deja en 1600 px de
// lado mayor y se convierte a AVIF, WebP, JPEG XL y JPEG buscando la nota 80 de
// SSIMULACRA 2: «no se distingue puesta al lado», lo que enseña el deslizador.
// El original se sirve como WebP sin pérdida (los mismos píxeles, menos peso),
// pero la web dice lo que pesa el PNG. Salen public/comparador/<id>/ y
// src/datos/comparador.json; los dos se suben a git, porque CI no compila Apolo.
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const APOLO = resolve(SITIO, "../target/release/apolo");
const CACHE = join(process.env.XDG_CACHE_HOME ?? join(process.env.HOME ?? tmpdir(), ".cache"), "apolo-comparador");
const NOTA = 80;
const LADO = 1600;

const FOTOS = [
  {
    id: "mascara",
    titulo: { es: "Máscara funeraria de oro, Micenas, siglo XVI a. C.", en: "Gold funerary mask, Mycenae, 16th century BC" },
    autor: "Jebulon",
    licencia: "CC0",
    origen: "https://commons.wikimedia.org/wiki/File:Mycene_gold_mask_1_NAMA_Athens_Greece.jpg",
    url: "https://upload.wikimedia.org/wikipedia/commons/3/3b/Mycene_gold_mask_1_NAMA_Athens_Greece.jpg",
  },
];

// Los formatos, como los nombra `apolo lote` en su resumen, y su extensión.
const FORMATOS = { avif: ["AVIF", "avif"], webp: ["WebP", "webp"], jxl: ["JPEG XL", "jxl"], jpeg: ["JPEG", "jpg"] };

if (!existsSync(APOLO)) throw new Error(`Falta ${APOLO}: make cli`);
mkdirSync(CACHE, { recursive: true });
// Lo que dice Apolo, por las dos salidas: el progreso de cada imagen va por la de errores.
const apolo = (...a) => {
  const r = spawnSync(APOLO, a, { encoding: "utf8", env: { ...process.env, LC_ALL: "C" } });
  if (r.status !== 0) throw new Error(`apolo ${a.join(" ")}:\n${r.stderr}`);
  return r.stdout + r.stderr;
};

const datos = [];
for (const f of FOTOS) {
  const bajada = join(CACHE, `${f.id}.jpg`);
  if (!existsSync(bajada)) {
    const r = await fetch(f.url, { headers: { "User-Agent": "ApoloWeb/1.0 (https://apolo.webcafeina.com)" } });
    if (!r.ok) throw new Error(`${f.url}: ${r.status}`);
    writeFileSync(bajada, Buffer.from(await r.arrayBuffer()));
  }
  const tmp = join(tmpdir(), `apolo-comparador-${f.id}`);
  rmSync(tmp, { recursive: true, force: true });
  // El original: PNG de 1600 px, sin pérdida.
  apolo("lote", bajada, "--formato", "png", "--lado-mayor", String(LADO), "-s", join(tmp, "original"));
  const png = join(tmp, "original", `${f.id}.png`);
  // Cada formato, a la nota 80.
  const salida = apolo("lote", png, "--formato", Object.keys(FORMATOS).join(","), "--objetivo", String(NOTA), "--medir", "-s", join(tmp, "formatos"));
  const destino = join(SITIO, "public/comparador", f.id);
  rmSync(destino, { recursive: true, force: true });
  mkdirSync(destino, { recursive: true });
  apolo("webp", "-lossless", "-z", "6", "-quiet", png, "-o", join(destino, "original.webp"));
  const formatos = {};
  for (const [clave, [nombre, ext]] of Object.entries(FORMATOS)) {
    const m = salida.match(new RegExp(`${nombre} [\\d.]+ [KM]B −\\d+% quality (\\d+) score ([\\d.]+)`));
    if (!m) throw new Error(`No encuentro ${nombre} en:\n${salida}`);
    const fichero = `${f.id}.${ext}`;
    writeFileSync(join(destino, fichero), readFileSync(join(tmp, "formatos", fichero)));
    formatos[clave] = { fichero, bytes: statSync(join(destino, fichero)).size, calidad: Number(m[1]), nota: Number(m[2]) };
  }
  // Las medidas, de la cabecera IHDR del PNG.
  const cabecera = readFileSync(png);
  const [ancho, alto] = [cabecera.readUInt32BE(16), cabecera.readUInt32BE(20)];
  const bytes = statSync(png).size;
  datos.push({ ...f, ancho, alto, original: { bytes }, formatos, nota: NOTA });
  rmSync(tmp, { recursive: true, force: true });
  console.log(`${f.id}: PNG ${bytes} B →`, Object.entries(formatos).map(([k, v]) => `${k} ${v.bytes} B (${v.nota})`).join(", "));
}
writeFileSync(join(SITIO, "src/datos/comparador.json"), JSON.stringify({ fotos: datos }, null, 2) + "\n");
