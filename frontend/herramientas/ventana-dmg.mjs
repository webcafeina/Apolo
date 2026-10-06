// Simula la ventana del .dmg sin un Mac (make ventana-dmg): el fondo a 2x con
// el icono de Apolo y una carpeta Aplicaciones encima, en las posiciones y al
// tamaño que pone el Finder (los de tauri.macos.conf.json; Tauri usa iconos de
// 128 px y nombres de 16 px). Sirve para ver que nada pisa los iconos.
import { chromium } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..", "..");
const conf = JSON.parse(readFileSync(resolve(raiz, "src-tauri/tauri.macos.conf.json"), "utf8"));
const dmg = conf.bundle.macOS.dmg;
const b64 = (f) => readFileSync(resolve(raiz, f)).toString("base64");
const icono = (x, y, img, nombre) => `
  <div style="position:absolute;left:${x - 64}px;top:${y - 64}px;width:128px;text-align:center">
    ${img}
    <div style="font:16px -apple-system,system-ui,sans-serif;color:#fff;margin-top:6px;text-shadow:0 1px 2px #000">${nombre}</div>
  </div>`;
const carpeta = `<div style="width:128px;height:104px;margin-top:12px;border-radius:12px;background:linear-gradient(#7cc4f5,#3a8fd8)"></div>`;
const html = `<body style="margin:0">
  <div style="position:relative;width:${dmg.windowSize.width}px;height:${dmg.windowSize.height}px;background:url(data:image/png;base64,${b64("empaquetado/macos/fondo-dmg@2x.png")}) 0 0/100% 100%">
    ${icono(dmg.appPosition.x, dmg.appPosition.y, `<img src="data:image/png;base64,${b64("empaquetado/icono.png")}" width="128" height="128">`, "Apolo")}
    ${icono(dmg.applicationFolderPosition.x, dmg.applicationFolderPosition.y, carpeta, "Applications")}
  </div></body>`;
const navegador = await chromium.launch();
const pagina = await navegador.newPage({ viewport: { width: dmg.windowSize.width, height: dmg.windowSize.height }, deviceScaleFactor: 2 });
await pagina.setContent(html);
const destino = process.argv[2] ?? resolve(raiz, "target/ventana-dmg.png");
await pagina.screenshot({ path: destino });
await navegador.close();
console.log(destino);
