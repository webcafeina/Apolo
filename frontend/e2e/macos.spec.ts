// El aspecto de macOS 27 (ADR 0028), en el navegador con ?sistema=macos: lo que
// tiene que cumplirse para que se vea como macOS sin que cambie nada de lo que
// hace la ventana. El vidrio de verdad solo se ve en la ventana; aquí se simula
// con ?vidrio=si.

import { expect, test } from "@playwright/test";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/corpus");

test.use({ testIdAttribute: "data-prueba" });
// La barra lateral se anima al ocultarse: las medidas, sin animaciones.
test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
});

const alto = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().height);
const ancho = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().width);

test("en macOS: la escala del kit, la barra lateral de 240 sin marca y filas de 32", async ({ page }) => {
  await page.goto("/?sistema=macos");
  await expect(page.locator("html")).toHaveAttribute("data-sistema", "macos");
  expect(await page.evaluate(() => getComputedStyle(document.body).fontSize)).toBe("13px");
  expect(await alto(page.locator(".zona-soltar button").first())).toBe(24);
  expect(await ancho(page.locator(".lateral"))).toBe(240);
  await expect(page.locator(".lateral .sello")).toHaveCount(0);
  await expect(page.getByTestId("firma")).toHaveCount(0);
  expect(await alto(page.getByRole("button", { name: "Lotes", exact: true }))).toBe(32);
});

test("en macOS: la cabecera de cada sección es la barra de herramientas de 52, y en el Estudio cubre el inspector", async ({ page }) => {
  await page.goto("/?sistema=macos");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "foto.webp"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  const barra = page.locator(".estudio .herramientas");
  expect(await alto(barra)).toBe(52);
  // De lado a lado: por encima del lienzo y del panel de opciones.
  const caja = await barra.evaluate((b) => b.getBoundingClientRect().right);
  const panel = await page.locator(".estudio > .panel").evaluate((p) => ({ top: p.getBoundingClientRect().top, right: p.getBoundingClientRect().right }));
  expect(caja).toBe(panel.right);
  expect(panel.top).toBeGreaterThanOrEqual(52);
  // Las órdenes, en cápsulas de 36.
  expect(await alto(barra.getByRole("button", { name: "Abrir otra…" }))).toBe(36);
  expect(await alto(barra.getByRole("radiogroup"))).toBe(36);
});

test("en macOS: el botón oculta la barra lateral, y se recuerda", async ({ page }) => {
  await page.goto("/?sistema=macos");
  const boton = page.getByTestId("boton-lateral");
  await expect(boton).toHaveAttribute("aria-expanded", "true");
  await boton.click();
  expect(await ancho(page.locator(".lateral"))).toBe(0);
  await expect(boton).toHaveAttribute("aria-expanded", "false");
  // Oculta, no se llega a ella con el tabulador: `inert`.
  await expect(page.locator(".lateral")).toHaveAttribute("inert", "");
  await page.getByRole("button", { name: "Lotes", exact: true }).focus();
  expect(await page.evaluate(() => document.activeElement?.textContent)).not.toBe("Lotes");
  await page.reload();
  expect(await ancho(page.locator(".lateral"))).toBe(0);
  await page.getByTestId("boton-lateral").click();
  expect(await ancho(page.locator(".lateral"))).toBe(240);
  await page.reload();
  expect(await ancho(page.locator(".lateral"))).toBe(240);
});

test("en macOS: Ajustes en filas agrupadas, y los interruptores siguen guardando", async ({ page }) => {
  await page.goto("/?sistema=macos");
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  const idioma = page.getByTestId("idioma");
  await expect(idioma).toHaveClass(/tarjeta-ajuste/);
  expect(await alto(idioma)).toBeGreaterThanOrEqual(42);
  await expect(idioma.getByRole("combobox", { name: "Idioma" })).toBeVisible();
  const avisar = page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" });
  await expect(avisar).toBeChecked();
  await avisar.uncheck();
  await page.reload();
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  await expect(page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" })).not.toBeChecked();
  await page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" }).check();
});

test("en macOS con vidrio: la barra lateral deja ver el material, el contenido no", async ({ page }) => {
  await page.goto("/?sistema=macos&vidrio=si");
  await expect(page.locator("html")).toHaveAttribute("data-vidrio", "si");
  const fondos = await page.evaluate(() => ({
    lateral: getComputedStyle(document.querySelector(".lateral")!).backgroundColor,
    zona: getComputedStyle(document.querySelector(".zona")!).backgroundColor,
  }));
  expect(fondos.lateral).toBe("rgba(0, 0, 0, 0)");
  expect(fondos.zona).not.toBe("rgba(0, 0, 0, 0)");
});
