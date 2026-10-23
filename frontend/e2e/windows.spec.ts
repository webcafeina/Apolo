// El aspecto de Windows 11 (ADR 0027), en el navegador con ?sistema=windows: lo que tiene que
// cumplirse para que se vea como Fluent sin que cambie nada de lo que hace la ventana. Mica de
// verdad solo se ve en la ventana; aquí, el vidrio se simula con ?vidrio=si.

import { expect, test } from "@playwright/test";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/corpus");

test.use({ testIdAttribute: "data-prueba" });
// El panel se anima al plegarse y desplegarse (200 ms): las medidas, sin animaciones.
test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
});

test("en Windows: la escala de Fluent, sus iconos y el fondo opaco sin vidrio", async ({ page }) => {
  await page.goto("/?sistema=windows");
  await expect(page.locator("html")).toHaveAttribute("data-sistema", "windows");
  await expect(page.locator("html")).toHaveAttribute("data-vidrio", "no");
  const estilo = await page.evaluate(() => ({
    fondo: getComputedStyle(document.body).backgroundColor,
    texto: getComputedStyle(document.body).fontSize,
    boton: getComputedStyle(document.querySelector(".zona-soltar button")!).height,
  }));
  expect(estilo.fondo).not.toBe("rgba(0, 0, 0, 0)");
  expect(estilo.texto).toBe("14px");
  expect(estilo.boton).toBe("32px");
  // Los iconos de Fluent (rellenos, en su caja de 16 o de 20), con los mismos nombres.
  await expect(page.locator(".zona-soltar .icono").first()).toHaveAttribute("viewBox", "0 0 16 16");
  await expect(page.locator(".zona-soltar .icono").first()).toHaveAttribute("fill", "currentColor");
});

test("en Windows: los interruptores siguen siendo interruptores, y la barra de progreso no se anima", async ({ page }) => {
  await page.goto("/?sistema=windows");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "foto.webp"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  const paleta = page.getByRole("switch", { name: "Reducir paleta", exact: true });
  await expect(paleta).not.toBeChecked();
  await paleta.check();
  await expect(paleta).toBeChecked();
  // La barra de la versión nueva: su ancho es el del porcentaje (actualizar.spec.ts), así que sin
  // transición también en Windows.
  const duracion = await page.evaluate(() => {
    const barra = document.createElement("div");
    barra.className = "barra-progreso";
    barra.appendChild(document.createElement("i"));
    document.body.appendChild(barra);
    const d = getComputedStyle(barra.firstElementChild!).transitionDuration;
    barra.remove();
    return d;
  });
  expect(duracion).toBe("0s");
});

test("en Windows con vidrio: la barra lateral deja ver el material, la zona de trabajo no", async ({ page }) => {
  await page.goto("/?sistema=windows&vidrio=si");
  await expect(page.locator("html")).toHaveAttribute("data-vidrio", "si");
  const fondos = await page.evaluate(() => ({
    lateral: getComputedStyle(document.querySelector(".lateral")!).backgroundColor,
    zona: getComputedStyle(document.querySelector(".zona")!).backgroundColor,
  }));
  expect(fondos.lateral).toBe("rgba(0, 0, 0, 0)");
  expect(fondos.zona).not.toBe("rgba(0, 0, 0, 0)");
});

test("en Windows: la barra de título propia, y el panel solo navega", async ({ page }) => {
  await page.goto("/?sistema=windows");
  const barra = page.locator(".barra-titulo");
  await expect(barra).toBeVisible();
  expect(await barra.evaluate((b) => b.getBoundingClientRect().height)).toBe(48);
  await expect(barra).toContainText("Apolo");
  for (const nombre of ["Minimizar", "Maximizar", "Cerrar"]) {
    await expect(barra.getByRole("button", { name: nombre, exact: true })).toHaveAttribute("tabindex", "-1");
  }
  // El nombre y la versión los dicen la barra de título y Ajustes › Acerca de.
  await expect(page.locator(".lateral .sello")).toHaveCount(0);
  await expect(page.getByTestId("firma")).toHaveCount(0);
  // Las filas del panel, de 36, con el icono de 16.
  const fila = page.getByRole("button", { name: "Lotes", exact: true });
  expect(await fila.evaluate((b) => b.getBoundingClientRect().height)).toBe(36);
  await expect(fila.locator(".icono")).toHaveAttribute("viewBox", "0 0 16 16");
});

test("en Windows: el panel abre plegado, ☰ lo despliega, y se recuerda", async ({ page }) => {
  await page.goto("/?sistema=windows");
  const lateral = page.locator(".lateral");
  const menu = page.getByTestId("boton-menu");
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().width)).toBe(48);
  await expect(menu).toHaveAttribute("aria-expanded", "false");
  // Plegado, el nombre de cada sección sigue siendo su nombre, y sale al dejar el puntero.
  const lotes = page.getByRole("button", { name: "Lotes", exact: true });
  await expect(lotes.locator(".etiqueta")).toBeHidden();
  await expect(lotes).toHaveAttribute("title", "Lotes");
  await lotes.click();
  await expect(page.getByRole("heading", { name: "Lotes" })).toBeVisible();
  await menu.click();
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().width)).toBe(320);
  await expect(menu).toHaveAttribute("aria-expanded", "true");
  await page.reload();
  expect(await page.locator(".lateral").evaluate((l) => l.getBoundingClientRect().width)).toBe(320);
  await page.getByTestId("boton-menu").click();
  await page.reload();
  expect(await page.locator(".lateral").evaluate((l) => l.getBoundingClientRect().width)).toBe(48);
});

test("en Windows con la ventana estrecha: el panel va plegado y ☰ lo abre encima", async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 700 });
  await page.goto("/?sistema=windows");
  const lateral = page.locator(".lateral");
  const zona = page.locator(".zona");
  const antes = await zona.evaluate((z) => z.getBoundingClientRect().left);
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().width)).toBe(48);
  await page.getByTestId("boton-menu").click();
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().width)).toBe(320);
  // Encima: el contenido no se mueve, y el panel empieza debajo de la barra de título.
  expect(await zona.evaluate((z) => z.getBoundingClientRect().left)).toBe(antes);
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().top)).toBe(48);
  await page.getByRole("button", { name: "Presets", exact: true }).click();
  expect(await lateral.evaluate((l) => l.getBoundingClientRect().width)).toBe(48);
  await expect(page.getByRole("heading", { name: "Presets" })).toBeVisible();
});

test("en Windows: Ajustes con tarjetas, y los interruptores siguen guardando", async ({ page }) => {
  await page.goto("/?sistema=windows");
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Ajustes" })).toHaveCSS("font-size", "28px");
  const idioma = page.getByTestId("idioma");
  await expect(idioma).toHaveClass(/tarjeta-ajuste/);
  await expect(idioma.getByRole("combobox", { name: "Idioma" })).toBeVisible();
  const avisar = page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" });
  await expect(avisar).toBeChecked();
  await avisar.uncheck();
  await page.reload();
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  await expect(page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" })).not.toBeChecked();
  await page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" }).check();
});

test("en Windows: el panel se pliega y se despliega con su animación", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.goto("/?sistema=windows");
  const ventana = page.locator(".ventana");
  await expect(ventana).toHaveCSS("transition-duration", "0.2s");
  await expect(ventana).toHaveCSS("transition-property", "grid-template-columns");
  await page.getByTestId("boton-menu").click();
  await expect.poll(() => page.locator(".lateral").evaluate((l) => l.getBoundingClientRect().width)).toBe(320);
});

// ADR 0032: Recortar y Redimensionar, en una barra con el aspecto del sistema.
test("en Windows: la barra de encima de la imagen es una CommandBar, y la ventanita un Flyout", async ({ page }) => {
  await page.goto("/?sistema=windows");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  const barra = page.getByTestId("barra-imagen");
  await expect(barra).toBeVisible();
  expect(await barra.evaluate((b) => b.getBoundingClientRect().height)).toBe(44);
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
  expect(await ventana.evaluate((v) => getComputedStyle(v).borderRadius)).toBe("8px");
});
