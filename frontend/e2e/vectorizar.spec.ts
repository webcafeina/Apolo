// La pantalla «Vectorizar» de punta a punta (ADR 0039): abrir un logotipo, ver el dibujo y su nota, cambiar
// opciones, mirar los contornos y las diferencias, y exportar.

import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/vector");

test.use({ testIdAttribute: "data-prueba" });

async function abrir(page: Page, fichero = "logo.png", url = "/") {
  await page.goto(url);
  await page.getByRole("button", { name: "Vectorizar" }).click();
  const elegir = page.waitForEvent("filechooser");
  await page.getByTestId("zona-vector").getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, fichero));
  await expect(page.getByTestId("peso-vector")).not.toHaveText("…", { timeout: 30_000 });
}

/** Cuántos colores distintos pinta el lienzo visible (muestreado). */
async function colores(page: Page): Promise<number> {
  return page.getByTestId("lienzo").locator("visible=true").first().evaluate((c: HTMLCanvasElement) => {
    const d = c.getContext("2d")!.getImageData(0, 0, c.width, c.height).data;
    const vistos = new Set<number>();
    for (let i = 0; i < d.length; i += 4 * 31) vistos.add((d[i] << 16) | (d[i + 1] << 8) | d[i + 2]);
    return vistos.size;
  });
}

test("vectorizar un logotipo enseña el dibujo, su nota y lo que tiene", async ({ page }) => {
  await abrir(page);
  await expect(page.getByText("logo.png", { exact: true })).toBeVisible();
  const nota = Number((await page.getByTestId("nota-vector").textContent())!.replace(",", "."));
  expect(nota).toBeGreaterThan(70);
  await expect(page.getByTestId("estadisticas-vector")).toContainText("formas");
  await expect(page.getByTestId("estadisticas-vector")).toContainText("nodos");
  // Con transparencia: lo dice la cabecera.
  await expect(page.getByText(/con transparencia/)).toBeVisible();
  expect(await colores(page)).toBeGreaterThan(5);
});

test("lo transparente se ve sobre cuadros a los dos lados del deslizador", async ({ page }) => {
  await abrir(page);
  // Cuántos píxeles de cada mitad del lienzo (el original a la izquierda, el dibujo a la derecha) son del
  // color oscuro del damero: si el dibujo se pintase sobre blanco, la mitad derecha no tendría ninguno.
  const [izquierda, derecha] = await page.getByTestId("lienzo").locator("visible=true").first().evaluate((c: HTMLCanvasElement) => {
    const muestra = document.createElement("canvas").getContext("2d")!;
    muestra.fillStyle = getComputedStyle(c).getPropertyValue("--damero-b");
    muestra.fillRect(0, 0, 1, 1);
    const [r, g, b] = muestra.getImageData(0, 0, 1, 1).data;
    const d = c.getContext("2d")!.getImageData(0, 0, c.width, c.height).data;
    const cuenta = [0, 0];
    for (let i = 0; i < d.length; i += 4) {
      if (d[i] === r && d[i + 1] === g && d[i + 2] === b) cuenta[(i / 4) % c.width < c.width / 2 ? 0 : 1]++;
    }
    return cuenta;
  });
  expect(izquierda).toBeGreaterThan(1000);
  expect(derecha).toBeGreaterThan(1000);
});

test("limitar los colores vuelve a vectorizar y apaga los degradados", async ({ page }) => {
  await abrir(page);
  const antes = await page.getByTestId("estadisticas-vector").textContent();
  await page.getByTestId("vector-control-colores").getByRole("switch").check();
  // Un deslizador no se escribe: Inicio lo lleva al mínimo, 2 colores.
  await page.locator("#vector-colores").press("Home");
  await expect(page.getByTestId("estadisticas-vector")).not.toHaveText(antes!, { timeout: 30_000 });
  await expect(page.getByTestId("estadisticas-vector")).toContainText("2 colores");
  await expect(page.getByTestId("vector-control-degradados").getByRole("switch")).toBeDisabled();
});

test("buscar una nota dice qué detalle encontró", async ({ page }) => {
  await abrir(page);
  await page.getByTestId("vector-control-objetivo").getByRole("switch").check();
  // Al mínimo, 50: este logotipo llega.
  await page.locator("#vector-objetivo").press("Home");
  await expect(page.getByTestId("hallado")).toContainText("Detalle", { timeout: 60_000 });
  await expect(page.getByTestId("hallado")).toContainText("el más ligero que llega");
});

test("contornos y diferencias pintan algo", async ({ page }) => {
  await abrir(page);
  await page.getByRole("radio", { name: "Contornos" }).click();
  await expect.poll(() => colores(page), { timeout: 15_000 }).toBeGreaterThan(5);
  await page.getByRole("radio", { name: "Diferencias" }).click();
  await expect.poll(() => colores(page), { timeout: 15_000 }).toBeGreaterThan(1);
});

test("exportar a SVG, a PDF y a DXF de bordes", async ({ page }) => {
  await abrir(page);
  for (const [formato, cabecera] of [
    ["svg", "<svg"],
    ["pdf", "%PDF-1.4"],
    ["dxf", "  0\nSECTION"],
  ] as const) {
    await page.getByTestId("vector-formato").selectOption(formato);
    if (formato === "dxf") {
      await page.getByTestId("vector-estilo").selectOption("trazar_bordes");
      await page.getByTestId("vector-dxf").selectOption("rectas");
    }
    const descarga = page.waitForEvent("download");
    await page.getByTestId("exportar-vector").click();
    const d = await descarga;
    expect(d.suggestedFilename()).toBe(`logo.${formato}`);
    const texto = readFileSync((await d.path())!, "latin1");
    expect(texto.startsWith(cabecera)).toBe(true);
    if (formato === "svg") expect(texto).not.toContain("<rect");
    if (formato === "dxf") expect(texto).toContain("LWPOLYLINE");
  }
});

// Para mirarlas a ojo: CAPTURAS_VECTOR=carpeta, con el aspecto de la web y de cada sistema.
for (const sistema of ["", "windows", "macos", "linux"]) {
  test(`capturas de la pantalla${sistema ? ` (${sistema})` : ""}`, async ({ page }, info) => {
    test.skip(!process.env.CAPTURAS_VECTOR, "solo con CAPTURAS_VECTOR=carpeta");
    await abrir(page, "logo.png", sistema ? `/?sistema=${sistema}` : "/");
    const dir = process.env.CAPTURAS_VECTOR!;
    const nombre = `vectorizar${sistema ? `-${sistema}` : ""}-${info.project.name}`;
    await page.waitForTimeout(500);
    await page.screenshot({ path: join(dir, `${nombre}.png`) });
    await page.getByRole("radio", { name: "Contornos" }).click();
    await page.waitForTimeout(1500);
    await page.screenshot({ path: join(dir, `${nombre}-contornos.png`) });
  });
}

// En macOS la barra de herramientas va de borde a borde: «Abrir otra…» no puede salirse ni montarse en los modos,
// a ningún ancho y tampoco con la barra lateral oculta (lo pidió el cliente, 2026-11-10).
for (const ancho of [1280, 1024, 900]) {
  for (const lateral of [true, false]) {
    test(`macOS a ${ancho} px${lateral ? "" : " sin barra lateral"}: los botones de la cabecera caben`, async ({ page }, info) => {
      await page.setViewportSize({ width: ancho, height: 760 });
      await abrir(page, "logo.png", "/?sistema=macos");
      if (!lateral) await page.getByRole("button", { name: "Ocultar la barra lateral" }).click();
      const otra = page.getByRole("button", { name: "Abrir otra…" });
      await expect(otra).toBeVisible();
      const caja = (await otra.boundingBox())!;
      // Dentro de la ventana, con el margen de la barra de herramientas de macOS (8 px, el mismo que en el
      // Estudio: lo comprueba la prueba de abajo).
      expect(caja.x + caja.width).toBeLessThanOrEqual(ancho - 8);
      // Ningún botón de la cabecera se monta sobre otro.
      const cajas = await page.locator(".vectorizar .herramientas button").evaluateAll((bs) =>
        bs.filter((b) => (b as HTMLElement).offsetParent).map((b) => b.getBoundingClientRect().toJSON()),
      );
      for (let i = 1; i < cajas.length; i++) expect(cajas[i].left).toBeGreaterThanOrEqual(cajas[i - 1].right - 1);
      // Con poco sitio (menos de 1.100 px de sección), los modos se quedan en icono y el nombre se lee entero.
      if (ancho - (lateral ? 240 : 0) < 1100) {
        await expect(page.locator(".vectorizar .segmentado.con-iconos .etiqueta").first()).toBeHidden();
        await expect(page.getByRole("heading", { name: "logo.png" })).toHaveJSProperty("scrollWidth", await page.getByRole("heading", { name: "logo.png" }).evaluate((h) => h.clientWidth));
      }
      // El botón de exportar se ve entero (a 900 px se salía de la barra de estado).
      const exportar = (await page.getByTestId("exportar-vector").boundingBox())!;
      expect(exportar.x + exportar.width).toBeLessThanOrEqual(ancho - 320 + 1);
      expect(exportar.x).toBeGreaterThanOrEqual(0);
      if (process.env.CAPTURAS_VECTOR) {
        await page.screenshot({ path: join(process.env.CAPTURAS_VECTOR, `macos-${ancho}${lateral ? "" : "-sin-lateral"}-${info.project.name}.png`) });
      }
    });
  }
}

test("macOS: el margen derecho de la cabecera, el mismo que en el Estudio", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 760 });
  await abrir(page, "logo.png", "/?sistema=macos");
  const vector = (await page.getByRole("button", { name: "Abrir otra…" }).boundingBox())!;
  await page.getByRole("button", { name: "Estudio" }).click();
  const elegir = page.waitForEvent("filechooser");
  await page.getByTestId("zona").getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "logo.png"));
  const estudio = (await page.getByRole("button", { name: "Abrir otra…" }).boundingBox())!;
  expect(Math.abs(vector.x + vector.width - (estudio.x + estudio.width))).toBeLessThanOrEqual(1);
});

test("capturas del Estudio en macOS a 900 px, para comparar", async ({ page }) => {
  test.skip(!process.env.CAPTURAS_VECTOR, "solo con CAPTURAS_VECTOR=carpeta");
  await page.setViewportSize({ width: 900, height: 760 });
  await page.goto("/?sistema=macos");
  const elegir = page.waitForEvent("filechooser");
  await page.getByTestId("zona").getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "logo.png"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  await page.screenshot({ path: join(process.env.CAPTURAS_VECTOR!, "estudio-macos-900.png") });
});
