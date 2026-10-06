// Capturas para mirar, no para comprobar: solo corren con CAPTURAS=1 y
// quedan en frontend/capturas/.
import { test } from "@playwright/test";
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

test("vacío", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Abrir una imagen…" }).waitFor();
  await page.screenshot({ path: `capturas/vacio-${info.project.name}.png` });
});

test("ajustes", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Ajustes" }).click();
  await page.getByRole("heading", { name: "Motores" }).waitFor();
  await page.screenshot({ path: `capturas/ajustes-${info.project.name}.png` });
});
