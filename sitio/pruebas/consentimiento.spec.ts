import { expect, test } from "@playwright/test";
import { sinGoogle } from "./comun";

// Consent Mode v2 en modo básico: nada de Google hasta que se acepta.
test("la primera vez: el aviso, todo denegado y GTM sin cargar", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await expect(page.locator("[data-cookies]")).toBeVisible();
  const capa = await page.evaluate(() => JSON.parse(JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer)));
  expect(capa[0]).toMatchObject({ 0: "consent", 1: "default", 2: { analytics_storage: "denied", ad_storage: "denied" } });
  expect(pedidas).toEqual([]);
});

test("aceptar: concede la analítica, carga GTM y no vuelve a preguntar", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Aceptar" }).click();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  await expect.poll(() => pedidas.some((u) => u.includes("gtm.js?id=GTM-PRUEBA"))).toBe(true);
  await page.reload();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  const capa = await page.evaluate(() => JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer));
  expect(capa).toContain('"analytics_storage":"granted"');
});

test("rechazar: no carga nada, y se puede cambiar desde el pie", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Rechazar" }).click();
  await page.reload();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  expect(pedidas).toEqual([]);
  await page.getByRole("button", { name: "Preferencias de cookies" }).click();
  await expect(page.locator("[data-cookies]")).toBeVisible();
  await page.getByRole("button", { name: "Configurar" }).click();
  await page.getByRole("checkbox", { name: /Analítica/ }).check();
  await page.getByRole("button", { name: "Guardar la elección" }).click();
  await expect.poll(() => pedidas.some((u) => u.includes("gtm.js"))).toBe(true);
});

test("las descargas cuentan en la capa de datos", async ({ page }) => {
  await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Rechazar" }).click();
  // Que el clic no se vaya de la página: el enlace de la tarjeta de macOS.
  await page.route(/github\.com/, (r) => r.fulfill({ status: 204 }));
  await page.locator('.tarjeta[data-para="macos"] a.descargar').click();
  const capa = await page.evaluate(() => JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer));
  expect(capa).toContain('"event":"descarga_apolo"');
  expect(capa).toContain('"sistema":"macos"');
});
