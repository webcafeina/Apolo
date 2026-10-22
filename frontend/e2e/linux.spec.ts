// El aspecto de Ubuntu 26.10 (ADR 0029), en el navegador con ?sistema=linux: lo
// que tiene que cumplirse para que se vea como una aplicación de libadwaita sin
// que cambie nada de lo que hace la ventana. La parte de arriba se pide con
// &barra=propia (la de Apolo, en GNOME) o &barra=sistema.

import { expect, test } from "@playwright/test";

test.use({ testIdAttribute: "data-prueba" });
// La barra lateral se anima al ocultarse: las medidas, sin animaciones.
test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
});

const alto = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().height);
const ancho = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().width);

test("en Linux: la escala de libadwaita, los iconos de Yaru y la barra lateral sin marca", async ({ page }) => {
  await page.goto("/?sistema=linux&barra=propia");
  await expect(page.locator("html")).toHaveAttribute("data-sistema", "linux");
  // 11 pt, como Ubuntu Sans en Ubuntu.
  expect(await page.evaluate(() => getComputedStyle(document.body).fontSize)).toBe("14.6667px");
  expect(await alto(page.locator(".zona-soltar button").first())).toBe(34);
  expect(await ancho(page.locator(".lateral"))).toBe(220);
  await expect(page.locator(".lateral .sello")).toHaveCount(0);
  await expect(page.getByTestId("firma")).toHaveCount(0);
  const fila = page.getByRole("button", { name: "Lotes", exact: true });
  expect(await alto(fila)).toBe(36);
  // Los simbólicos de Yaru, en su caja de 16.
  await expect(fila.locator(".icono")).toHaveAttribute("viewBox", "0 0 16 16");
  expect(await alto(page.locator(".herramientas").first())).toBe(46);
});

test("en Linux con la barra propia: los botones de la ventana, en el lado que diga el sistema", async ({ page }) => {
  await page.goto("/?sistema=linux&barra=propia");
  await expect(page.locator("html")).toHaveAttribute("data-barra", "propia");
  const derecha = page.locator(".lado-botones.derecha");
  await expect(derecha.getByRole("button")).toHaveCount(3);
  for (const nombre of ["Minimizar", "Maximizar", "Cerrar"]) {
    await expect(derecha.getByRole("button", { name: nombre, exact: true })).toHaveAttribute("tabindex", "-1");
  }
  await expect(page.locator(".barra-sistema-simulada")).toHaveCount(0);
  // A la izquierda, si el sistema los pone ahí.
  await page.goto("/?sistema=linux&barra=propia&botones=close,minimize,maximize:");
  await expect(page.locator(".lado-botones.izquierda").getByRole("button")).toHaveCount(3);
  await expect(page.locator(".lado-botones.derecha")).toHaveCount(0);
});

test("en Linux con la barra del sistema: sin botones propios", async ({ page }) => {
  await page.goto("/?sistema=linux&barra=sistema");
  await expect(page.locator("html")).toHaveAttribute("data-barra", "sistema");
  await expect(page.locator(".lado-botones")).toHaveCount(0);
});

test("en Linux: el botón oculta la barra lateral, y se recuerda", async ({ page }) => {
  await page.goto("/?sistema=linux&barra=propia");
  const boton = page.getByTestId("boton-lateral");
  await expect(boton).toHaveAttribute("aria-expanded", "true");
  await boton.click();
  expect(await ancho(page.locator(".lateral"))).toBe(0);
  await expect(page.locator(".lateral")).toHaveAttribute("inert", "");
  await page.reload();
  expect(await ancho(page.locator(".lateral"))).toBe(0);
  await page.getByTestId("boton-lateral").click();
  expect(await ancho(page.locator(".lateral"))).toBe(220);
});

test("en Linux con la ventana estrecha: la barra lateral va oculta y sale encima", async ({ page }) => {
  await page.setViewportSize({ width: 820, height: 700 });
  await page.goto("/?sistema=linux&barra=propia");
  const lateral = page.locator(".lateral");
  const antes = await page.locator(".zona").evaluate((z) => z.getBoundingClientRect().left);
  expect(await ancho(lateral)).toBe(0);
  await page.getByTestId("boton-lateral").click();
  expect(await ancho(lateral)).toBe(220);
  expect(await page.locator(".zona").evaluate((z) => z.getBoundingClientRect().left)).toBe(antes);
  await page.getByRole("button", { name: "Presets", exact: true }).click();
  expect(await ancho(lateral)).toBe(0);
  await expect(page.getByRole("heading", { name: "Presets" })).toBeVisible();
});

test("en Linux: Ajustes en lista en tarjeta, y el interruptor de libadwaita guarda", async ({ page }) => {
  await page.goto("/?sistema=linux&barra=propia");
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  const idioma = page.getByTestId("idioma");
  await expect(idioma).toHaveClass(/tarjeta-ajuste/);
  expect(await alto(idioma)).toBeGreaterThanOrEqual(50);
  const avisar = page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" });
  expect(await ancho(avisar)).toBe(46);
  expect(await alto(avisar)).toBe(26);
  const pomo = avisar.locator("xpath=following-sibling::span[@class='pomo']");
  expect(await ancho(pomo)).toBe(20);
  await expect(avisar).toBeChecked();
  await avisar.uncheck();
  await page.reload();
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  await expect(page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" })).not.toBeChecked();
  await page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" }).check();
});
