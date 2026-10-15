// Ajustes › Agentes (ADR 0023). La fila «MCP» de la barra abre Ajustes por la
// pestaña de los agentes, con la versión del conector, lo que se copia para
// cada agente y el botón del .mcpb. apolo-dev deja el paquete junto a su
// carpeta de configuración temporal, no en Descargas.

import { expect, test } from "@playwright/test";

test.use({ testIdAttribute: "data-prueba" });

test("la fila «MCP» abre los agentes, con la versión y todo lo que se copia", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "MCP" }).click();
  await expect(page.getByRole("radio", { name: "Agentes de IA" })).toHaveAttribute("aria-checked", "true");

  const agentes = page.getByTestId("agentes");
  await expect(agentes.getByTestId("version-conector")).toContainText(/Conector apolo-mcp \d+\.\d+\.\d+/);
  await expect(agentes.getByTestId("version-conector")).toContainText("Se actualiza solo con Apolo");
  await expect(agentes.getByTestId("aviso-agentes")).toHaveCount(0);
  // Lo de reiniciar Claude Desktop, destacado y en negrita.
  await expect(agentes.getByTestId("aviso-reiniciar").locator("strong")).toHaveText(
    "Después de actualizar Apolo, cierra y vuelve a abrir Claude Desktop.",
  );

  // Los tres fragmentos nombran el mismo binario.
  const ruta = (await agentes.getByTestId("ruta-mcp").locator("code").textContent())!;
  expect(ruta).toMatch(/apolo-mcp(\.exe)?$/);
  await expect(agentes.getByTestId("orden-claude-code")).toContainText(`claude mcp add --scope user apolo -- "${ruta}"`);
  await agentes.getByText("VS Code", { exact: true }).click();
  await expect(agentes.getByTestId("json-vscode")).toContainText('"servers"');
  await expect(agentes.getByTestId("json-vscode")).toContainText(JSON.stringify(ruta));

  // Copiar deja la orden en el portapapeles.
  await agentes.getByTestId("orden-claude-code").getByRole("button", { name: "Copiar" }).click();
  await expect(agentes.getByTestId("orden-claude-code").getByRole("button")).toHaveText("Copiado");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(`claude mcp add --scope user apolo -- "${ruta}"`);

  // La privacidad, dicha.
  await expect(agentes).toContainText("lo ve el proveedor del modelo");
});

test("crea la extensión de Claude Desktop", async ({ page }) => {
  test.setTimeout(90_000);
  await page.goto("/");
  await page.getByRole("button", { name: "MCP" }).click();
  await page.getByRole("button", { name: "Crear la extensión" }).click();
  await expect(page.getByTestId("mcpb-creada")).toContainText(/Guardada en .*Apolo-\d+\.\d+\.\d+-\w+(-\d+)?\.mcpb/, { timeout: 60_000 });
});

test("Ajustes abre por General, y se cambia de pestaña", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Ajustes" }).click();
  await expect(page.getByRole("radio", { name: "General" })).toHaveAttribute("aria-checked", "true");
  await expect(page.getByRole("heading", { name: "Acerca de" })).toBeVisible();
  await page.getByRole("button", { name: "MCP" }).click();
  await expect(page.getByRole("heading", { name: "Acerca de" })).toBeHidden();
  await expect(page.getByTestId("agentes")).toBeVisible();
});
