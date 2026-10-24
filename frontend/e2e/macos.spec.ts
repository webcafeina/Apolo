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

test("en macOS: la escala del kit, la barra lateral de 240 sin marca y filas de 36", async ({ page }) => {
  await page.goto("/?sistema=macos");
  await expect(page.locator("html")).toHaveAttribute("data-sistema", "macos");
  expect(await page.evaluate(() => getComputedStyle(document.body).fontSize)).toBe("13px");
  expect(await alto(page.locator(".zona-soltar button").first())).toBe(24);
  expect(await ancho(page.locator(".lateral"))).toBe(240);
  await expect(page.locator(".lateral .sello")).toHaveCount(0);
  await expect(page.getByTestId("firma")).toHaveCount(0);
  expect(await alto(page.getByRole("button", { name: "Lotes", exact: true }))).toBe(36);
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

test("en macOS: la selección del segmentado se desliza, y el control no se abomba", async ({ page }) => {
  await page.goto("/?sistema=macos");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "foto.webp"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  const grupo = page.locator(".herramientas .segmentado");
  await expect(grupo).toHaveAttribute("data-medido", "si");
  const x = () => grupo.evaluate((g) => g.style.getPropertyValue("--sel-x"));
  const antes = await x();
  const lado = page.getByRole("radio", { name: "Lado a lado" });
  await lado.click();
  await expect.poll(x).not.toBe(antes);
  // El fondo lo pinta el segmentado, no el botón, y el control no cambia de tamaño al pulsarlo.
  expect(await lado.evaluate((b) => getComputedStyle(b).backgroundColor)).toBe("rgba(0, 0, 0, 0)");
  await lado.hover();
  await page.mouse.down();
  expect(await grupo.evaluate((g) => getComputedStyle(g).transform)).toBe("none");
  await page.mouse.up();
});

test("en macOS: el interruptor es una pista de 44 × 20 con su píldora aparte, y la barra lateral va en el tamaño grande", async ({ page }) => {
  await page.goto("/?sistema=macos");
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  const avisar = page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" });
  expect(await ancho(avisar)).toBe(44);
  expect(await alto(avisar)).toBe(20);
  // El botón es un elemento propio (el `.pomo` de `Interruptor`), una píldora de 26 × 16, que
  // se desliza al marcarlo.
  const pomo = avisar.locator("xpath=following-sibling::span[@class='pomo']");
  expect(await ancho(pomo)).toBe(26);
  expect(await alto(pomo)).toBe(16);
  expect(await pomo.evaluate((p) => getComputedStyle(p).transform)).toContain("14");
  const fila = page.getByRole("button", { name: "Lotes", exact: true });
  expect(await alto(fila)).toBe(36);
  expect(await fila.locator(".icono").evaluate((i) => i.getBoundingClientRect().width)).toBe(20);
  // Los iconos, en oro: en claro, el intermedio que eligió el cliente (el de 3:1 se veía oliva);
  // en oscuro, el de la marca.
  const oro = test.info().project.name === "oscuro" ? "rgb(255, 200, 61)" : "rgb(217, 154, 0)";
  await expect(fila.locator(".icono")).toHaveCSS("color", oro);
});

// ADR 0032: Recortar y Redimensionar, en una barra con el aspecto del sistema.
test("en macOS: la barra de encima de la imagen es una barra secundaria, y la ventanita un popover", async ({ page }) => {
  await page.goto("/?sistema=macos");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  const barra = page.getByTestId("barra-imagen");
  await expect(barra).toBeVisible();
  expect(await barra.evaluate((b) => b.getBoundingClientRect().height)).toBe(38);
  // Debajo de la cabecera, encima del lienzo.
  const [cabecera, lienzo, caja] = await Promise.all([
    page.locator(".estudio .herramientas").boundingBox(),
    page.getByTestId("lienzo").boundingBox(),
    barra.boundingBox(),
  ]);
  expect(caja!.y).toBeGreaterThanOrEqual(cabecera!.y + cabecera!.height - 1);
  expect(lienzo!.y).toBeGreaterThanOrEqual(caja!.y + caja!.height - 1);
  await page.getByTestId("redimensionar").click();
  const ventana = page.getByRole("dialog", { name: "Redimensionar" });
  expect(await ventana.evaluate((v) => getComputedStyle(v).borderRadius)).toBe("14px");
});
