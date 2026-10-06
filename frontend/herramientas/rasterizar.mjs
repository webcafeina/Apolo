// Convierte los SVG de la marca a los PNG que hacen falta (make iconos).
//
// Con el Chromium de Playwright, como Esfinge: en la máquina de desarrollo no hay
// ningún conversor de SVG y Playwright ya trae un navegador para las pruebas.
// Los PNG se suben al repositorio: CI no rasteriza.
//
// Después, `pnpm tauri icon empaquetado/icono.png` saca de icono.png los
// tamaños de cada sistema (src-tauri/icons/).
import { chromium } from "@playwright/test";
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..", "..");

// [origen SVG, destino PNG, ancho]. El alto sale del viewBox.
const trabajos = [
  ["empaquetado/icono.svg", "empaquetado/icono.png", 1024],
  ["empaquetado/icono.svg", "frontend/public/favicon.png", 64],
  // El fondo del .dmg a 1x y 2x; en el Mac de CI, tiffutil los une en un TIFF
  // HiDPI (publicar.yml). Con un PNG suelto macOS lo escala y se ve borroso.
  ["empaquetado/macos/fondo-dmg.svg", "empaquetado/macos/fondo-dmg.png", 660],
  ["empaquetado/macos/fondo-dmg.svg", "empaquetado/macos/fondo-dmg@2x.png", 1320],
];

// La interfaz importa los SVG de la marca desde aquí: Vite no sirve ficheros
// de fuera de frontend/.
const copias = [
  ["empaquetado/icono.svg", "frontend/src/marca/icono.svg"],
  ["empaquetado/marca.svg", "frontend/src/marca/marca.svg"],
];

const navegador = await chromium.launch();
const pagina = await navegador.newPage();
for (const [origen, destino, ancho] of trabajos) {
  const svg = readFileSync(resolve(raiz, origen), "utf8");
  const [, , w, h] = svg.match(/viewBox="([^"]+)"/)[1].split(/\s+/).map(Number);
  const alto = Math.round((ancho * h) / w);
  await pagina.setViewportSize({ width: ancho, height: alto });
  await pagina.setContent(
    `<style>html,body{margin:0;background:transparent}svg{display:block;width:${ancho}px;height:${alto}px}</style>${svg}`,
    { waitUntil: "networkidle" },
  );
  await pagina.evaluate(() => document.fonts.ready);
  mkdirSync(dirname(resolve(raiz, destino)), { recursive: true });
  await pagina.locator("svg").first().screenshot({ path: resolve(raiz, destino), omitBackground: true });
  console.log(`${destino} (${ancho}×${alto})`);
}
await navegador.close();

for (const [origen, destino] of copias) {
  mkdirSync(dirname(resolve(raiz, destino)), { recursive: true });
  copyFileSync(resolve(raiz, origen), resolve(raiz, destino));
  console.log(destino);
}
