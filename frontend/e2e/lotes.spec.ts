// Lotes de punta a punta, contra apolo-dev. El navegador no sabe las rutas de
// lo que se suelta, así que en desarrollo se escriben (el campo `ruta-dev`).

import { expect, test } from "@playwright/test";
import { copyFileSync, existsSync, mkdtempSync, readdirSync } from "node:fs";
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

  // El formato: WebP por defecto; AVIF y JPEG XL, a la vista como «pronto».
  await expect(page.getByTestId("formato-lote")).toHaveValue("webp");
  await expect(page.getByTestId("formato-lote").locator("option:disabled")).toHaveCount(2);

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

test("cancelar a medias para, y el botón y el resumen van en rojo", async ({ page }) => {
  // Muchas copias de la foto del corpus, con un preset lento: da tiempo a cancelar.
  const entrada = mkdtempSync(join(tmpdir(), "apolo-e2e-cancelar-"));
  for (let i = 0; i < 60; i++) copyFileSync(join(corpus, "foto.webp"), join(entrada, `${String(i).padStart(2, "0")}.webp`));
  const salida = join(mkdtempSync(join(tmpdir(), "apolo-e2e-cancelar-salida-")), "salida");
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.getByTestId("ruta-dev").fill(entrada);
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await page.getByTestId("salida").fill(salida);
  await page.getByTestId("preset-lote").selectOption({ label: "Dibujo" });
  await page.getByRole("button", { name: "Convertir 60 imágenes" }).click();

  const cancelar = page.getByRole("button", { name: "Cancelar" });
  const rojo = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--error").trim());
  await expect(cancelar).toHaveClass(/peligro/);
  await cancelar.click();
  const titulo = page.getByTestId("titulo-resumen");
  await expect(titulo).toContainText("Cancelado:", { timeout: 30_000 });
  await expect(titulo).toHaveClass(/cancelado/);
  expect(rojo).not.toBe("");
  const hechas = Number((await titulo.textContent())!.match(/Cancelado: (\d+)/)![1]);
  expect(hechas).toBeLessThan(60);
  expect(existsSync(salida) ? readdirSync(salida).length : 0).toBe(hechas);
});

test("varios formatos: un fichero de cada, o solo el más ligero", async ({ page }) => {
  const base = mkdtempSync(join(tmpdir(), "apolo-e2e-formatos-"));
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.getByTestId("ruta-dev").fill(corpus);
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await expect(page.getByTestId("recogida")).toContainText("2 imágenes");
  await page.getByTestId("anadir-formato").click();
  await page.getByTestId("formato-lote-1").selectOption("png");
  await expect(page.getByTestId("orden-lote-1")).toHaveText("oxipng");
  // Con dos formatos, la carpeta propuesta acaba en -apolo.
  await expect(page.getByTestId("salida")).toHaveValue(/corpus-apolo$/);

  const todos = join(base, "todos");
  await page.getByTestId("salida").fill(todos);
  await page.getByRole("button", { name: "Convertir 2 imágenes" }).click();
  await expect(page.getByTestId("resumen")).toContainText("2 imágenes convertidas", { timeout: 30_000 });
  await expect(page.getByTestId("por-formato").locator("li")).toHaveCount(2);
  expect(readdirSync(todos).sort()).toEqual(["foto.png", "foto.webp", "orientacion-6.png", "orientacion-6.webp"]);

  await page.getByRole("button", { name: "Otro lote" }).click();
  await page.getByTestId("ruta-dev").fill(corpus);
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await page.getByTestId("anadir-formato").click();
  await page.getByTestId("formato-lote-1").selectOption("png");
  await page.getByTestId("mas-ligero").getByRole("checkbox").check();
  const ligero = join(base, "ligero");
  await page.getByTestId("salida").fill(ligero);
  await page.getByRole("button", { name: "Convertir 2 imágenes" }).click();
  await expect(page.getByTestId("resumen")).toContainText("2 imágenes convertidas", { timeout: 30_000 });
  expect(readdirSync(ligero)).toHaveLength(2);
});
