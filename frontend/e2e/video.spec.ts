// El vídeo de la web (ADR 0033): Apolo en marcha, unos segundos. Solo corre
// con VIDEO=1 (make video-web). En vez de la grabación de Playwright (VP8,
// borrosa), los fotogramas del screencast de Chromium a doble densidad, con su
// hora; sitio/herramientas/video.mjs los junta con ffmpeg. Playwright no pinta
// el puntero: va uno dibujado que sigue al ratón.
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "@playwright/test";

const FOTO = process.env.VIDEO_FOTO ?? "";
const DESTINO = process.env.VIDEO_DESTINO ?? "";
const SISTEMA = process.env.VIDEO_SISTEMA ?? "sistema=macos&vidrio=si";
const INGLES = process.env.VIDEO_IDIOMA === "en";

// Los textos, en el idioma del vídeo: la clave en es.json, el texto en en.json.
const catalogo = (i: string) => JSON.parse(readFileSync(join(import.meta.dirname, `../src/i18n/${i}.json`), "utf8"));
function hojas(o: Record<string, unknown>, p = ""): [string, string][] {
  return Object.entries(o).flatMap(([k, v]) =>
    typeof v === "object" && v ? hojas(v as Record<string, unknown>, `${p}${k}.`) : [[`${p}${k}`, String(v)] as [string, string]],
  );
}
const [ES, EN] = [hojas(catalogo("es")), new Map(hojas(catalogo("en")))];
const n = (texto: string) => (INGLES ? (EN.get(ES.find(([, v]) => v === texto)?.[0] ?? "") ?? texto) : texto);

test.skip(!process.env.VIDEO, "solo con VIDEO=1");
test.use({ testIdAttribute: "data-prueba", deviceScaleFactor: 2 });

test("video", async ({ page }, info) => {
  test.skip(info.project.name !== "claro");
  test.setTimeout(180_000);
  rmSync(DESTINO, { recursive: true, force: true });
  mkdirSync(DESTINO, { recursive: true });
  await page.addInitScript(() => {
    addEventListener("DOMContentLoaded", () => {
      const c = document.createElement("div");
      c.innerHTML =
        '<svg width="22" height="28" viewBox="0 0 22 28"><path d="M2 2v20l5.5-5 3.6 8.3 3.3-1.4-3.5-8.1H18z" fill="#111" stroke="#fff" stroke-width="1.6" stroke-linejoin="round"/></svg>';
      Object.assign(c.style, { position: "fixed", left: "0", top: "0", zIndex: "99999", pointerEvents: "none", transform: "translate(-100px,-100px)" });
      document.body.append(c);
      addEventListener("mousemove", (e) => (c.style.transform = `translate(${e.clientX - 2}px,${e.clientY - 2}px)`), true);
    });
  });
  await page.goto(`/?${SISTEMA}`);
  await page.mouse.move(700, 500);
  await page.waitForTimeout(500);

  // El screencast: cada fotograma con su hora, para darle su duración al juntarlos.
  const cdp = await page.context().newCDPSession(page);
  const fotogramas: { fichero: string; t: number }[] = [];
  cdp.on("Page.screencastFrame", ({ data, metadata, sessionId }) => {
    const fichero = `f${String(fotogramas.length).padStart(5, "0")}.png`;
    writeFileSync(join(DESTINO, fichero), Buffer.from(data, "base64"));
    fotogramas.push({ fichero, t: metadata.timestamp ?? Date.now() / 1000 });
    // Puede llegar uno con la página ya cerrada: da igual.
    cdp.send("Page.screencastFrameAck", { sessionId }).catch(() => {});
  });
  await cdp.send("Page.startScreencast", { format: "png", everyNthFrame: 1 });

  const abrir = page.getByRole("button", { name: n("Abrir una imagen…") });
  const b = (await abrir.boundingBox())!;
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 25 });
  await page.waitForTimeout(300);
  const elegir = page.waitForEvent("filechooser");
  await abrir.click();
  await (await elegir).setFiles(FOTO);
  await page.getByTestId("lienzo").waitFor();
  await page.waitForTimeout(1800);
  // El deslizador, a un lado y a otro.
  const l = (await page.getByTestId("lienzo").boundingBox())!;
  const y = l.y + l.height * 0.55;
  await page.mouse.move(l.x + l.width / 2, y, { steps: 30 });
  await page.mouse.down();
  await page.mouse.move(l.x + l.width * 0.22, y, { steps: 40 });
  await page.mouse.move(l.x + l.width * 0.78, y, { steps: 60 });
  await page.mouse.move(l.x + l.width * 0.5, y, { steps: 40 });
  await page.mouse.up();
  await page.waitForTimeout(500);
  // La calidad baja y el peso con ella.
  const calidad = page.locator("#control-calidad");
  const q = (await calidad.boundingBox())!;
  await page.mouse.move(q.x + q.width / 2, q.y + q.height / 2, { steps: 30 });
  await page.waitForTimeout(300);
  for (const v of ["70", "60", "50", "40"]) {
    await calidad.fill(v);
    await page.waitForTimeout(900);
  }
  // Diferencias: dónde se pierde. Y vuelta al deslizador.
  for (const [nombre, espera] of [["Diferencias", 2500], ["Deslizador", 1500]] as const) {
    const r = page.getByRole("radio", { name: n(nombre) });
    const c = (await r.boundingBox())!;
    await page.mouse.move(c.x + c.width / 2, c.y + c.height / 2, { steps: 25 });
    await r.click();
    await page.waitForTimeout(espera);
  }
  await cdp.send("Page.stopScreencast");
  writeFileSync(join(DESTINO, "fotogramas.json"), JSON.stringify(fotogramas));
});
