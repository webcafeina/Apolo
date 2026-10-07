// Actualizarse desde la aplicación (ADR 0018). En el navegador no hay plugin de
// Tauri: con `?novedad=9.9.9` aparece una versión nueva de mentira que se
// «descarga» sola, y así se prueban la banda y la sección de Ajustes. La puerta
// de las 24 horas se prueba en Rust (`crates/servicio/src/ajustes.rs`); aquí se
// usa «Buscar ahora», que se la salta, porque el servidor es el mismo para
// todas las pruebas y la primera ya gasta el turno del día.

import { expect, test, type Page } from "@playwright/test";

test.use({ testIdAttribute: "data-prueba" });

async function buscarAhora(page: Page, url: string) {
  await page.goto(url);
  await page.getByRole("button", { name: "Ajustes" }).click();
  await page.getByRole("button", { name: "Buscar ahora" }).click();
}

test("una versión nueva se anuncia, se descarga y pide instalar y reiniciar", async ({ page }) => {
  await buscarAhora(page, "/?novedad=9.9.9");
  await expect(page.getByTestId("actualizaciones")).toContainText("Hay una versión nueva: Apolo 9.9.9.");
  await expect(page.getByTestId("ultima-vez")).toContainText("Se miró por última vez el");

  const banda = page.getByTestId("novedad");
  await expect(banda).toContainText("Hay una versión nueva: Apolo 9.9.9");
  await banda.getByRole("button", { name: "Descargar" }).click();
  await expect(banda).toContainText("Apolo 9.9.9 está lista. Se instalará y volverá a abrirse.");
  await banda.getByRole("button", { name: "Instalar y reiniciar" }).click();
  await expect(banda).toContainText("Instalando Apolo 9.9.9");
  await expect(banda.getByRole("button")).toHaveCount(0);
});

test("«Ahora no» quita la banda", async ({ page }) => {
  await buscarAhora(page, "/?novedad=9.9.9");
  const banda = page.getByTestId("novedad");
  await banda.getByRole("button", { name: "Ahora no" }).click();
  await expect(banda).toHaveCount(0);
});

test("sin versión nueva lo dice, y la casilla se recuerda", async ({ page }, info) => {
  // La carpeta de ajustes es la misma para los dos temas, que corren a la vez.
  test.skip(info.project.name !== "claro", "la casilla se prueba en un solo tema");
  await buscarAhora(page, "/");
  await expect(page.getByTestId("actualizaciones")).toContainText("Ya tienes la última versión.");
  await expect(page.getByTestId("novedad")).toHaveCount(0);

  const casilla = page.getByRole("checkbox", { name: "Avisarme cuando haya una versión nueva" });
  await expect(casilla).toBeChecked();
  await casilla.uncheck();
  await page.reload();
  await page.getByRole("button", { name: "Ajustes" }).click();
  await expect(casilla).not.toBeChecked();
  await casilla.check();
  await expect(casilla).toBeChecked();
});
