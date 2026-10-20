// Apolo en inglés (ADR 0025): se elige en Ajustes › General, la interfaz
// cambia en el acto, se queda guardado al recargar, y los errores de Rust
// llegan también en inglés. Ningún texto se queda sin clave: i18next avisa en
// la consola de desarrollo de cada clave que falta, y aquí se vigila.
//
// La batería entera comparte un apolo-dev y su carpeta de ajustes: al acabar
// se vuelve a «el del sistema», que en las pruebas es el español
// (playwright.config.ts), para no dejar en inglés las que vienen después.

import { expect, test, type Page } from "@playwright/test";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/corpus");

test.use({ testIdAttribute: "data-prueba" });

test.afterEach(async ({ request }) => {
  const r = await request.post("/api/poner_idioma", { data: { idioma: "sistema" } });
  expect(await r.json()).toBe("es");
});

/** Las claves que faltan, según los avisos de i18next en la consola. */
function clavesQueFaltan(page: Page): string[] {
  const faltan: string[] = [];
  page.on("console", (m) => {
    if (m.text().startsWith("i18n:")) faltan.push(m.text());
  });
  return faltan;
}

test("se cambia a inglés en Ajustes, y toda la interfaz y los errores de Rust salen en inglés", async ({ page }) => {
  const faltan = clavesQueFaltan(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Te damos la bienvenida a Apolo" })).toBeVisible();

  // Ajustes › General › Idioma.
  await page.getByRole("button", { name: "Ajustes" }).click();
  const idioma = page.getByTestId("idioma").getByRole("combobox");
  await expect(idioma).toHaveValue("sistema");
  await expect(idioma.locator("option")).toHaveText(["Igual que el sistema", "Español", "English"]);
  await idioma.selectOption("en");

  // Cambia en el acto, sin recargar.
  await expect(page.getByRole("heading", { name: "Language" })).toBeVisible();
  await expect(idioma.locator("option")).toHaveText(["Same as the system", "Español", "English"]);
  await expect(page.getByRole("radio", { name: "General" })).toHaveAttribute("aria-checked", "true");
  await expect(page.getByRole("heading", { name: "About" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Updates" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Check now" })).toBeVisible();
  await expect(page.locator("html")).toHaveAttribute("lang", "en");

  // Y se queda guardado: al recargar, Rust dice inglés antes de pintar.
  await page.reload();
  await expect(page.getByRole("heading", { name: "Welcome to Apolo" })).toBeVisible();
  for (const s of ["Studio", "Batches", "Presets", "MCP", "Settings"]) {
    await expect(page.getByRole("button", { name: s, exact: true })).toBeVisible();
  }

  // Lotes.
  await page.getByRole("button", { name: "Batches", exact: true }).click();
  await expect(page.getByText("Drop folders or images here")).toBeVisible();
  await page.getByTestId("ruta-dev").fill(corpus);
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await expect(page.getByTestId("recogida")).toContainText("2 images");
  await expect(page.getByRole("button", { name: "Convert 2 images" })).toBeVisible();

  // Ajustes › Agentes, por la fila MCP.
  await page.getByRole("button", { name: "MCP", exact: true }).click();
  await expect(page.getByRole("radio", { name: "AI agents" })).toHaveAttribute("aria-checked", "true");
  const agentes = page.getByTestId("agentes");
  await expect(agentes.getByTestId("version-conector")).toContainText(/apolo-mcp connector \d+\.\d+\.\d+/);
  await expect(agentes.getByRole("button", { name: "Create the extension" })).toBeVisible();
  await expect(agentes).toContainText("the model provider sees that");

  // El Estudio, con una imagen: los textos y los números en inglés.
  await page.getByRole("button", { name: "Studio", exact: true }).click();
  let elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Open an image…" }).click();
  await (await elegir).setFiles(join(corpus, "foto.webp"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  await expect(page.getByRole("button", { name: "Export…" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Paste command…" })).toBeVisible();
  await expect(page.getByText("Format", { exact: true }).first()).toBeVisible();
  await expect(page.getByRole("switch", { name: "Lossless" })).toBeVisible();

  // Un error de Rust: lo que no es una imagen.
  elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Open another…" }).click();
  await (await elegir).setFiles(join(corpus, "LEEME.md"));
  await expect(page.getByRole("alert")).toContainText("Unrecognized image format");

  // Otro, del servicio: una orden de qoiconv con un fichero de más.
  await page.getByRole("button", { name: "Paste command…" }).click();
  await page.getByTestId("orden-entrada").fill("qoiconv a.png b.qoi c.qoi");
  await page.getByRole("button", { name: "Load" }).click();
  await expect(page.getByText("qoiconv only takes the input and the output")).toBeVisible();

  expect(faltan).toEqual([]);
});

test("en español, ninguna clave falta tampoco", async ({ page }) => {
  const faltan = clavesQueFaltan(page);
  await page.goto("/");
  for (const s of ["Lotes", "Presets", "MCP", "Ajustes", "Estudio"]) {
    await page.getByRole("button", { name: s, exact: true }).click();
  }
  await expect(page.getByRole("heading", { name: "Te damos la bienvenida a Apolo" })).toBeVisible();
  expect(faltan).toEqual([]);
});
