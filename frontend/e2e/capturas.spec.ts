// Capturas para mirar, no para comprobar: solo corren con CAPTURAS=1 y
// quedan en frontend/capturas/.
import { test } from "@playwright/test";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

test.skip(!process.env.CAPTURAS, "solo con CAPTURAS=1");
test.use({ testIdAttribute: "data-prueba" });

test("estudio", async ({ page }, info) => {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  await page.getByText("foto.webp", { exact: true }).waitFor();
  await page.locator("#control-calidad").fill("10");
  await page.waitForTimeout(800);
  await page.getByTestId("nivel-avanzado").locator("summary").click();
  await page.screenshot({ path: `capturas/estudio-${info.project.name}.png` });
  await page.getByRole("radio", { name: "Lado a lado" }).click();
  await page.getByRole("button", { name: "Acercar" }).click();
  await page.getByRole("button", { name: "Acercar" }).click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `capturas/lado-a-lado-${info.project.name}.png` });
});

test("avif y jpeg xl", async ({ page }, info) => {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  await page.getByText("foto.webp", { exact: true }).waitFor();
  await page.getByTestId("nivel-avanzado").locator("summary").click();
  for (const f of ["avif", "jxl"]) {
    await page.getByTestId("formato").selectOption(f);
    await page.waitForTimeout(1500);
    await page.screenshot({ path: `capturas/${f}-${info.project.name}.png` });
  }
});

test("vacío", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Abrir una imagen…" }).waitFor();
  await page.screenshot({ path: `capturas/vacio-${info.project.name}.png` });
});

test("ajustes", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Ajustes" }).click();
  await page.getByRole("heading", { name: "Acerca de" }).waitFor();
  await page.screenshot({ path: `capturas/ajustes-${info.project.name}.png` });
});

test("presets", async ({ page }, info) => {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  await page.getByTestId("peso-resultado").waitFor();
  await page.locator("#control-calidad").fill("70");
  await page.getByTestId("nombre-preset").fill("Fotos web");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await page.getByRole("button", { name: "Presets" }).click();
  await page.getByTestId("preset").waitFor();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `capturas/presets-${info.project.name}.png` });
});

test("novedad", async ({ page }, info) => {
  await page.goto("/?novedad=0.3.4");
  await page.getByRole("button", { name: "Ajustes" }).click();
  await page.getByRole("button", { name: "Buscar ahora" }).click();
  await page.getByTestId("novedad").waitFor();
  await page.screenshot({ path: `capturas/novedad-${info.project.name}.png` });
  await page.getByRole("button", { name: "Descargar" }).click();
  await page.getByRole("button", { name: "Instalar y reiniciar" }).waitFor();
  await page.screenshot({ path: `capturas/novedad-lista-${info.project.name}.png` });
});

test("lotes", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.waitForTimeout(200);
  await page.screenshot({ path: `capturas/lotes-vacio-${info.project.name}.png` });
  await page.getByTestId("ruta-dev").fill(join(import.meta.dirname, "../../pruebas/corpus"));
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await page.getByTestId("recogida").waitFor();
  await page.getByTestId("salida").fill(join(mkdtempSync(join(tmpdir(), "apolo-capturas-")), "fotos-webp"));
  await page.screenshot({ path: `capturas/lotes-preparado-${info.project.name}.png` });
  await page.getByRole("button", { name: /^Convertir/ }).click();
  await page.getByTestId("resumen").waitFor();
  await page.screenshot({ path: `capturas/lotes-resumen-${info.project.name}.png`, fullPage: true });
});

test("formatos", async ({ page }, info) => {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  await page.getByTestId("peso-resultado").waitFor();
  await page.getByRole("radio", { name: "Lado a lado" }).click();
  await page.getByTestId("lados").getByRole("radio", { name: /Izquierda/ }).click();
  await page.getByTestId("comparar-formato").click();
  await page.locator("#control-calidadJpeg").fill("30");
  await page.getByTestId("rotulo-izquierda").filter({ hasText: "JPEG ·" }).waitFor();
  await page.waitForTimeout(500);
  await page.screenshot({ path: `capturas/formatos-${info.project.name}.png` });
});

test("lotes-formatos", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Lotes" }).click();
  await page.getByTestId("ruta-dev").fill(join(import.meta.dirname, "../../pruebas/corpus"));
  await page.getByRole("button", { name: "Añadir", exact: true }).click();
  await page.getByTestId("anadir-formato").click();
  await page.getByTestId("formato-lote-1").selectOption("png");
  await page.getByTestId("mas-ligero").waitFor();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `capturas/lotes-formatos-${info.project.name}.png`, fullPage: true });
});
