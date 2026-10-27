// Los iconos de la web y la imagen para las redes (ADR 0033), con el Chromium
// de Playwright: `node herramientas/iconos.mjs`. Salen a public/ y se suben a
// git (CI no rasteriza): icono.svg, favicon.ico (16 y 32 px), apple-touch-icon,
// los del manifiesto (192, 512 y el de máscara, con margen) y social/apolo-es.png
// y apolo-en.png, de 1200 × 630, con la frase y el comparador.
import { spawnSync } from "node:child_process";
import { chromium } from "@playwright/test";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const PUBLICO = join(SITIO, "public");
const ICONO = resolve(SITIO, "../empaquetado/icono.svg");
const svg = readFileSync(ICONO, "utf8").replace(/<!--[\s\S]*?-->\s*/, "");
const fuente = (f) => `data:font/woff2;base64,${readFileSync(join(PUBLICO, "fuentes", f)).toString("base64")}`;
const datos = JSON.parse(readFileSync(join(SITIO, "src/datos/comparador.json"), "utf8")).fotos[0];

copyFileSync(ICONO, join(PUBLICO, "icono.svg"));
const navegador = await chromium.launch();
const pagina = await navegador.newPage();

async function png(lado, margen = 0, fondo = "transparent") {
  await pagina.setViewportSize({ width: lado, height: lado });
  const dentro = lado - 2 * margen;
  await pagina.setContent(
    `<style>html,body{margin:0;background:${fondo}}div{width:${lado}px;height:${lado}px;display:grid;place-items:center}svg{width:${dentro}px;height:${dentro}px;display:block}</style><div>${svg}</div>`,
  );
  return pagina.screenshot({ omitBackground: fondo === "transparent" });
}

// El .ico con los PNG dentro (lo admiten todos los navegadores de hoy).
function ico(imagenes) {
  const cabecera = Buffer.alloc(6 + 16 * imagenes.length);
  cabecera.writeUInt16LE(0, 0);
  cabecera.writeUInt16LE(1, 2);
  cabecera.writeUInt16LE(imagenes.length, 4);
  let desplazamiento = cabecera.length;
  imagenes.forEach(([lado, datos], i) => {
    const o = 6 + 16 * i;
    cabecera.writeUInt8(lado, o);
    cabecera.writeUInt8(lado, o + 1);
    cabecera.writeUInt16LE(1, o + 4);
    cabecera.writeUInt16LE(32, o + 6);
    cabecera.writeUInt32LE(datos.length, o + 8);
    cabecera.writeUInt32LE(desplazamiento, o + 12);
    desplazamiento += datos.length;
  });
  return Buffer.concat([cabecera, ...imagenes.map(([, d]) => d)]);
}

writeFileSync(join(PUBLICO, "favicon.ico"), ico([[16, await png(16)], [32, await png(32)]]));
writeFileSync(join(PUBLICO, "apple-touch-icon.png"), await png(180));
writeFileSync(join(PUBLICO, "icono-192.png"), await png(192));
writeFileSync(join(PUBLICO, "icono-512.png"), await png(512));
// El de máscara: el sistema lo recorta en círculo; con margen y el fondo de la placa.
writeFileSync(join(PUBLICO, "icono-mascara-512.png"), await png(512, 72, "#2b2b31"));

// La imagen para las redes, en cada idioma.
mkdirSync(join(PUBLICO, "social"), { recursive: true });
const original = `data:image/webp;base64,${readFileSync(join(PUBLICO, "comparador", datos.id, "original.webp")).toString("base64")}`;
const avif = datos.formatos.avif;
const ahorro = (i) =>
  new Intl.NumberFormat(i === "es" ? "es-ES" : "en-US", { maximumFractionDigits: 0 }).format(100 * (1 - avif.bytes / datos.original.bytes));
const TEXTO = {
  es: { frase: "Imágenes más ligeras, igual de buenas.", sub: "Gratis para macOS, Windows y Linux", menos: "menos, sin que se note" },
  en: { frase: "Lighter images, just as good.", sub: "Free for macOS, Windows and Linux", menos: "smaller, and it doesn't show" },
};
await pagina.setViewportSize({ width: 1200, height: 630 });
for (const idioma of ["es", "en"]) {
  const t = TEXTO[idioma];
  await pagina.setContent(`<style>
    @font-face{font-family:Marcellus;src:url(${fuente("marcellus-latin-400.woff2")})}
    @font-face{font-family:Figtree;src:url(${fuente("figtree-latin-variable.woff2")});font-weight:300 900}
    html,body{margin:0}
    .l{width:1200px;height:630px;display:grid;grid-template-columns:560px 1fr;background:radial-gradient(70% 90% at 0% 0%,#3a3a42,#1c1c1e);color:#f5f5f7;font-family:Figtree,sans-serif;overflow:hidden}
    .t{padding:64px 0 64px 72px;display:grid;align-content:center;gap:22px}
    .m{display:flex;align-items:center;gap:16px;font-family:Marcellus,serif;font-size:44px}
    .m svg{width:64px;height:64px}
    h1{margin:0;font-size:58px;line-height:1.02;letter-spacing:-.035em;font-weight:800;text-wrap:balance}
    p{margin:0;font-size:24px;color:#c7c7cc}
    .f{position:relative;margin:56px 0 56px 24px;border-radius:24px 0 0 24px;overflow:hidden;background:url(${original}) center/cover}
    .g{position:absolute;left:28px;bottom:28px;padding:16px 20px;border-radius:16px;background:rgb(20 20 24 / .78);font-variant-numeric:tabular-nums}
    .g b{display:block;font-size:46px;color:#ffd56b;letter-spacing:-.02em}
    .g span{font-size:19px;color:#e5e5ea}
  </style>
  <div class="l"><div class="t"><div class="m">${svg}<span>Apolo</span></div><h1>${t.frase}</h1><p>${t.sub}</p></div>
  <div class="f"><div class="g"><b>−${ahorro(idioma)} %</b><span>AVIF · ${t.menos}</span></div></div></div>`);
  await pagina.evaluate(() => document.fonts.ready);
  await pagina.screenshot({ path: join(PUBLICO, "social", `apolo-${idioma}.png`) });
}
await navegador.close();
// Los PNG, optimizados con el propio Apolo (oxipng), sin pérdida.
const APOLO = resolve(SITIO, "../target/release/apolo");
for (const f of ["apple-touch-icon.png", "icono-192.png", "icono-512.png", "icono-mascara-512.png", "social/apolo-es.png", "social/apolo-en.png"])
  spawnSync(APOLO, ["png", "-o", "4", "--strip", "safe", join(PUBLICO, f)]);
console.log("iconos y social escritos en public/");
