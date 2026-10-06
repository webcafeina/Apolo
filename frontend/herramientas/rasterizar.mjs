// Convierte los SVG de la marca a los PNG que hacen falta (make iconos).
//
// Con el Chromium de Playwright, como Esfinge: en la máquina de desarrollo no hay
// ningún conversor de SVG y Playwright ya trae un navegador para las pruebas.
// Los PNG se suben al repositorio: CI no rasteriza.
//
// Después, `pnpm tauri icon empaquetado/icono.png` saca de icono.png los
// tamaños de cada sistema (src-tauri/icons/).
import { chromium } from "@playwright/test";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
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
  // El icono del volumen montado, con los tamaños y nombres que pide iconutil.
  // En el Mac de CI, iconutil los une en disco.icns y se mete en el .dmg.
  ...[16, 32, 128, 256, 512].flatMap((n) => [
    ["empaquetado/macos/disco.svg", `empaquetado/macos/disco.iconset/icon_${n}x${n}.png`, n],
    ["empaquetado/macos/disco.svg", `empaquetado/macos/disco.iconset/icon_${n}x${n}@2x.png`, n * 2],
  ]),
  // Las imágenes del instalador de Windows. NSIS solo acepta BMP de 24 bits.
  ["empaquetado/windows/lateral.svg", "empaquetado/windows/lateral.bmp", 164],
  ["empaquetado/windows/cabecera.svg", "empaquetado/windows/cabecera.bmp", 150],
];

// BMP de 24 bits, sin comprimir, de abajo arriba y con cada fila rellena hasta
// un múltiplo de 4 bytes: lo que NSIS sabe leer.
function bmp(ancho, alto, rgba) {
  const fila = Math.ceil((ancho * 3) / 4) * 4;
  const datos = fila * alto;
  const b = Buffer.alloc(54 + datos);
  b.write("BM", 0);
  b.writeUInt32LE(54 + datos, 2);
  b.writeUInt32LE(54, 10);
  b.writeUInt32LE(40, 14);
  b.writeInt32LE(ancho, 18);
  b.writeInt32LE(alto, 22);
  b.writeUInt16LE(1, 26);
  b.writeUInt16LE(24, 28);
  b.writeUInt32LE(datos, 34);
  b.writeInt32LE(2835, 38);
  b.writeInt32LE(2835, 42);
  for (let y = 0; y < alto; y++) {
    const destino = 54 + (alto - 1 - y) * fila;
    for (let x = 0; x < ancho; x++) {
      const o = (y * ancho + x) * 4;
      b[destino + x * 3] = rgba[o + 2];
      b[destino + x * 3 + 1] = rgba[o + 1];
      b[destino + x * 3 + 2] = rgba[o];
    }
  }
  return b;
}

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
  if (destino.endsWith(".bmp")) {
    // Se rasteriza a PNG y se leen los píxeles en el propio navegador.
    const png = await pagina.locator("svg").first().screenshot();
    const rgba = await pagina.evaluate(async ([b64, w, h]) => {
      const img = new Image();
      img.src = `data:image/png;base64,${b64}`;
      await img.decode();
      const c = document.createElement("canvas");
      c.width = w;
      c.height = h;
      const g = c.getContext("2d");
      g.drawImage(img, 0, 0);
      return Array.from(g.getImageData(0, 0, w, h).data);
    }, [png.toString("base64"), ancho, alto]);
    writeFileSync(resolve(raiz, destino), bmp(ancho, alto, rgba));
  } else {
    await pagina.locator("svg").first().screenshot({ path: resolve(raiz, destino), omitBackground: true });
  }
  console.log(`${destino} (${ancho}×${alto})`);
}
await navegador.close();

for (const [origen, destino] of copias) {
  mkdirSync(dirname(resolve(raiz, destino)), { recursive: true });
  copyFileSync(resolve(raiz, origen), resolve(raiz, destino));
  console.log(destino);
}
