// Lotes de punta a punta, contra apolo-dev. El navegador no sabe las rutas de
// lo que se suelta, así que en desarrollo se escriben (el campo `ruta-dev`).

import { expect, test } from "@playwright/test";
import { existsSync, mkdtempSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/corpus");

test.use({ testIdAttribute: "data-prueba" });

test("convierte una carpeta, la resume, y repetirlo no pisa nada", async ({ page }) => {
  const salida = join(mkdtempSync(join(tmpdir(), "apolo-e2e-lote-")), "salida");
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.getByTestId("ruta-dev").fill(corpus);
  await page.getByRole("button", { name: "Añadir", exact: true }).click();

  // El corpus tiene dos imágenes y un LEEME.md, que no cuenta.
  await expect(page.getByTestId("recogida")).toContainText("2 imágenes");
  await expect(page.getByTestId("salida")).toHaveValue(/corpus-webp$/);
  await page.getByTestId("salida").fill(salida);

  await page.getByTestId("preset-lote").selectOption({ label: "Foto" });
  await expect(page.getByTestId("orden-lote")).toContainText("-preset photo");

  await page.getByRole("button", { name: "Convertir 2 imágenes" }).click();
  const resumen = page.getByTestId("resumen");
  await expect(resumen).toContainText("2 imágenes convertidas en", { timeout: 30_000 });
  await expect(resumen).toContainText("% menos");
  // Con cinco o menos no hay «las que menos ahorran»: sería la misma lista.
  await expect(page.getByTestId("peores")).toHaveCount(0);
  await expect(page.getByTestId("filas").locator("li")).toHaveCount(2);
  expect(readdirSync(salida).sort()).toEqual(["foto.webp", "orientacion-6.webp"]);

  // Otra vez, a la misma carpeta: salen con «-2» y lo anterior sigue ahí.
  await page.getByRole("button", { name: "Otro lote" }).click();
  await page.getByTestId("ruta-dev").fill(join(corpus, "foto.webp"));
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await expect(page.getByTestId("recogida")).toContainText("1 imagen");
  await page.getByTestId("salida").fill(salida);
  await page.getByRole("button", { name: "Convertir 1 imagen" }).click();
  await expect(page.getByTestId("resumen")).toContainText("1 imagen convertida", { timeout: 30_000 });
  expect(existsSync(join(salida, "foto-2.webp"))).toBe(true);
});

test("una carpeta sin imágenes lo dice y no deja convertir", async ({ page }) => {
  const vacia = mkdtempSync(join(tmpdir(), "apolo-e2e-vacia-"));
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.getByTestId("ruta-dev").fill(vacia);
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await expect(page.getByTestId("recogida")).toContainText("no hay ninguna imagen");
  await expect(page.getByRole("button", { name: /^Convertir/ })).toBeDisabled();
  await page.getByRole("button", { name: `Quitar ${vacia}` }).click();
  await expect(page.getByTestId("entradas")).toHaveCount(0);
});
