// Capturas para mirar, no para comprobar: solo corren con CAPTURAS=1 y
// quedan en frontend/capturas/. Con CAPTURAS_IDIOMA=en, la interfaz va en
// inglés y quedan en frontend/capturas/en/ (las usa la web en inglés).
import { test } from "@playwright/test";
import { mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";

const INGLES = process.env.CAPTURAS_IDIOMA === "en";
// La foto y la carpeta del lote: las del corpus de pruebas, o, para la web (`make capturas-web`),
// una foto de Kodak y una carpeta con varias, que se ven mejor que una imagen de 128 × 128.
const FOTO = process.env.CAPTURAS_FOTO ?? join(import.meta.dirname, "../../pruebas/corpus/foto.webp");
const NOMBRE_FOTO = basename(FOTO);
const LOTE = process.env.CAPTURAS_LOTE ?? join(import.meta.dirname, "../../pruebas/corpus");
// La calidad del Estudio: 10 para que se vea la pérdida en la imagen pequeña; en la web, 50.
const CALIDAD = process.env.CAPTURAS_FOTO ? "50" : "10";
// El aspecto de otro sistema (ADR 0027): `CAPTURAS_SISTEMA=windows` (o macos, linux) y
// `CAPTURAS_VIDRIO=si` piden `?sistema=` y `?vidrio=si`, que el navegador simula (puente.ts).
// Van a capturas/<sistema>[-vidrio]/ (y en/ dentro, en inglés).
const SISTEMA = process.env.CAPTURAS_SISTEMA ?? "";
const VIDRIO = process.env.CAPTURAS_VIDRIO === "si";
const BASE = SISTEMA ? `capturas/${SISTEMA}${VIDRIO ? "-vidrio" : ""}` : "capturas";
const CARPETA = INGLES ? `${BASE}/en` : BASE;
function raiz(extra = ""): string {
  const q = new URLSearchParams(extra);
  if (SISTEMA) q.set("sistema", SISTEMA);
  if (VIDRIO) q.set("vidrio", "si");
  const s = q.toString();
  return s ? `/?${s}` : "/";
}
mkdirSync(CARPETA, { recursive: true });

// Los textos de los botones, en el idioma de las capturas: se busca la clave
// del texto en es.json y se toma la misma de en.json. Si no está (un nombre
// de fichero, «VS Code»), se queda igual.
const catalogo = (idioma: string) => JSON.parse(readFileSync(join(import.meta.dirname, `../src/i18n/${idioma}.json`), "utf8"));
const [ES, EN] = [catalogo("es"), catalogo("en")];
function hojas(o: Record<string, unknown>, p = ""): [string, string][] {
  return Object.entries(o).flatMap(([k, v]) => (typeof v === "object" && v ? hojas(v as Record<string, unknown>, `${p}${k}.`) : [[`${p}${k}`, String(v)] as [string, string]]));
}
const enPorClave = new Map(hojas(EN));
const claves = hojas(ES);
function n(texto: string): string {
  if (!INGLES) return texto;
  const clave = claves.find(([, v]) => v === texto)?.[0];
  return (clave && enPorClave.get(clave)) ?? texto;
}
// Igual, para los nombres que se buscan por su principio («Convertir 4 imágenes»).
function empieza(texto: string): RegExp {
  if (!INGLES) return new RegExp(`^${texto}`);
  const clave = claves.find(([, v]) => v.startsWith(texto))?.[0];
  const en = (clave && enPorClave.get(clave)) ?? texto;
  return new RegExp(`^${en.split(/\s|\{/)[0]}`);
}

test.skip(!process.env.CAPTURAS, "solo con CAPTURAS=1");
test.use({ testIdAttribute: "data-prueba" });

test("estudio", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.locator("#control-calidad").fill(CALIDAD);
  await page.waitForTimeout(800);
  await page.getByTestId("nivel-avanzado").locator("summary").click();
  await page.screenshot({ path: `${CARPETA}/estudio-${info.project.name}.png` });
  await page.getByRole("radio", { name: n("Lado a lado") }).click();
  await page.getByRole("button", { name: n("Acercar") }).click();
  await page.getByRole("button", { name: n("Acercar") }).click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${CARPETA}/lado-a-lado-${info.project.name}.png` });
  // El mapa de lo que se pierde, con la pérdida de la calidad de arriba (la de la web).
  await page.getByRole("radio", { name: n("Diferencias") }).click();
  await page.getByTestId("tipo-mapa").selectOption("estructura");
  await page.getByRole("button", { name: n("Ajustar") }).click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${CARPETA}/mapa-${info.project.name}.png` });
});

test("avif y jpeg xl", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.getByTestId("nivel-avanzado").locator("summary").click();
  for (const f of ["avif", "jxl"]) {
    await page.getByTestId("formato").selectOption(f);
    await page.waitForTimeout(1500);
    await page.screenshot({ path: `${CARPETA}/${f}-${info.project.name}.png` });
  }
});

test("medidas y diferencias", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.getByTestId("formato").selectOption("jpeg");
  await page.getByTestId("control-notaObjetivo").getByRole("switch").check();
  await page.getByTestId("buscando").waitFor();
  await page.screenshot({ path: `${CARPETA}/buscando-${info.project.name}.png` });
  await page.getByTestId("nota").waitFor({ timeout: 60_000 });
  await page.screenshot({ path: `${CARPETA}/medidas-${info.project.name}.png` });
  // La nota más alta: «Ni con calidad 100…», el texto más largo, junto a Exportar.
  await page.getByTestId("nota-objetivo").fill("95");
  await page.getByTestId("hallada").filter({ hasText: n("No llega ni con calidad 100") }).waitFor({ timeout: 60_000 });
  await page.screenshot({ path: `${CARPETA}/no-alcanzada-${info.project.name}.png` });
  await page.getByRole("radio", { name: n("Diferencias") }).click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${CARPETA}/diferencias-${info.project.name}.png` });
  await page.getByTestId("tipo-mapa").selectOption("estructura");
  // Alejada, con el aviso de vista reducida: no tiene que pisar el selector ni el rótulo.
  for (let i = 0; i < 8; i++) await page.getByRole("button", { name: n("Alejar") }).click();
  await page.getByTestId("nota-reducida").waitFor();
  await page.screenshot({ path: `${CARPETA}/diferencias-reducida-${info.project.name}.png` });
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${CARPETA}/estructura-${info.project.name}.png` });
});

test("vacío", async ({ page }, info) => {
  await page.goto(raiz());
  await page.getByRole("button", { name: n("Abrir una imagen…") }).waitFor();
  await page.screenshot({ path: `${CARPETA}/vacio-${info.project.name}.png` });
});

test("ajustes", async ({ page }, info) => {
  await page.goto(raiz());
  await page.getByRole("button", { name: n("Ajustes") }).click();
  await page.getByRole("heading", { name: n("Acerca de") }).waitFor();
  await page.screenshot({ path: `${CARPETA}/ajustes-${info.project.name}.png` });
});

test("presets", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByTestId("peso-resultado").waitFor();
  await page.locator("#control-calidad").fill("70");
  await page.getByTestId("nombre-preset").fill("Fotos web");
  await page.getByRole("button", { name: n("Guardar"), exact: true }).click();
  await page.getByRole("button", { name: n("Presets") }).click();
  await page.getByTestId("preset").waitFor();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${CARPETA}/presets-${info.project.name}.png` });
});

test("novedad", async ({ page }, info) => {
  await page.goto(raiz("novedad=0.3.4"));
  await page.getByRole("button", { name: n("Ajustes") }).click();
  await page.getByRole("button", { name: n("Buscar ahora") }).click();
  await page.getByTestId("novedad").waitFor();
  await page.screenshot({ path: `${CARPETA}/novedad-${info.project.name}.png` });
  await page.getByRole("button", { name: n("Descargar") }).click();
  await page.getByRole("button", { name: n("Instalar y reiniciar") }).waitFor();
  await page.screenshot({ path: `${CARPETA}/novedad-lista-${info.project.name}.png` });
});

test("lotes", async ({ page }, info) => {
  await page.goto(raiz());
  await page.getByRole("button", { name: n("Lotes") }).click();
  await page.waitForTimeout(200);
  await page.screenshot({ path: `${CARPETA}/lotes-vacio-${info.project.name}.png` });
  await page.getByTestId("ruta-dev").fill(LOTE);
  await page.getByRole("button", { name: n("Añadir"), exact: true }).click();
  await page.getByTestId("recogida").waitFor();
  await page.getByTestId("salida").fill(join(mkdtempSync(join(tmpdir(), "apolo-capturas-")), "fotos-webp"));
  await page.screenshot({ path: `${CARPETA}/lotes-preparado-${info.project.name}.png` });
  await page.getByRole("button", { name: empieza("Convertir") }).click();
  await page.getByTestId("resumen").waitFor();
  await page.screenshot({ path: `${CARPETA}/lotes-resumen-${info.project.name}.png`, fullPage: true });
});

test("formatos", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByTestId("peso-resultado").waitFor();
  await page.getByRole("radio", { name: n("Lado a lado") }).click();
  await page.getByTestId("lados").getByRole("radio", { name: new RegExp(n("Izquierda")) }).click();
  await page.getByTestId("comparar-formato").click();
  await page.locator("#control-calidadJpeg").fill("30");
  await page.getByTestId("rotulo-izquierda").filter({ hasText: "JPEG ·" }).waitFor();
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${CARPETA}/formatos-${info.project.name}.png` });
});

test("lotes-formatos", async ({ page }, info) => {
  await page.goto(raiz());
  await page.getByRole("button", { name: n("Lotes") }).click();
  await page.getByTestId("ruta-dev").fill(LOTE);
  await page.getByRole("button", { name: n("Añadir"), exact: true }).click();
  await page.getByTestId("anadir-formato").click();
  await page.getByTestId("formato-lote-1").selectOption("png");
  await page.getByTestId("mas-ligero").waitFor();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${CARPETA}/lotes-formatos-${info.project.name}.png`, fullPage: true });
});

test("agentes", async ({ page }, info) => {
  test.setTimeout(90_000);
  await page.goto(raiz());
  await page.getByRole("button", { name: n("MCP") }).click();
  await page.getByTestId("orden-claude-code").waitFor();
  // En la web, sin crearla: enseñaría la carpeta temporal donde la guarda.
  if (!process.env.CAPTURAS_FOTO) {
    await page.getByRole("button", { name: n("Crear la extensión") }).click();
    await page.getByTestId("mcpb-creada").waitFor({ timeout: 60_000 });
  }
  await page.screenshot({ path: `${CARPETA}/agentes-${info.project.name}.png` });
  await page.getByText(n("VS Code"), { exact: true }).click();
  await page.getByTestId("agentes").evaluate((e) => e.closest(".contenido")!.scrollTo(0, 10_000));
  await page.screenshot({ path: `${CARPETA}/agentes-abajo-${info.project.name}.png` });
});

test("recorte", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.getByRole("switch", { name: n("Recortar"), exact: true }).check();
  await page.getByTestId("proporcion").selectOption("4:3");
  await page.getByTestId("recortar-en-imagen").click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${CARPETA}/recorte-${info.project.name}.png` });
  await page.getByTestId("recorte-listo").click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${CARPETA}/recorte-hecho-${info.project.name}.png` });
});

// Los interruptores y el foco con el teclado (ADR 0027): el panel con uno encendido y los demás
// apagados, y el contorno del foco al recorrer el panel con Tab.
test("interruptores y foco", async ({ page }, info) => {
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.getByRole("switch", { name: n("Recortar"), exact: true }).check();
  await page.waitForTimeout(500);
  await page.locator(".panel").screenshot({ path: `${CARPETA}/interruptores-${info.project.name}.png` });
  await page.locator("#control-calidad").focus();
  await page.keyboard.press("Tab");
  await page.waitForTimeout(200);
  await page.screenshot({ path: `${CARPETA}/foco-${info.project.name}.png` });
  await page.keyboard.press("Tab");
  await page.waitForTimeout(200);
  await page.screenshot({ path: `${CARPETA}/foco-2-${info.project.name}.png` });
});

// Solo en Windows: el panel de navegación desplegado con ☰, y abierto encima del
// contenido con la ventana estrecha (ADR 0027).
test("panel de navegación", async ({ page }, info) => {
  test.skip(SISTEMA !== "windows", "solo en Windows");
  await page.goto(raiz());
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: n("Abrir una imagen…") }).click();
  await (await elegir).setFiles(FOTO);
  await page.getByText(NOMBRE_FOTO, { exact: true }).waitFor();
  await page.waitForTimeout(800);
  // Abre plegado; ☰ lo despliega.
  await page.getByTestId("boton-menu").click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${CARPETA}/panel-desplegado-${info.project.name}.png` });
  await page.getByTestId("boton-menu").click();
  await page.setViewportSize({ width: 960, height: 700 });
  await page.getByTestId("boton-menu").click();
  await page.getByRole("button", { name: n("Lotes") }).hover();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${CARPETA}/panel-encima-${info.project.name}.png` });
});
