// El aspecto de KDE Plasma 6 con Breeze (ADR 0031), en el navegador con
// ?sistema=linux&escritorio=kde: lo que tiene que cumplirse para que se vea como
// una aplicación de KDE. La parte de arriba es la de Apolo, como en GNOME
// (&barra=propia, elegida por el cliente), con el marco de Breeze.

import { expect, test } from "@playwright/test";
import { join } from "node:path";
import { ICONOS_BREEZE } from "../src/iconos-breeze";

test.use({ testIdAttribute: "data-prueba" });
test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
});

const KDE = "/?sistema=linux&escritorio=kde&barra=propia";
const alto = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().height);
const ancho = (l: import("@playwright/test").Locator) => l.evaluate((e) => e.getBoundingClientRect().width);

test("en KDE: la escala de Breeze, sus iconos y la barra lateral", async ({ page }) => {
  await page.goto(KDE);
  await expect(page.locator("html")).toHaveAttribute("data-escritorio", "kde");
  // 10 pt, como Noto Sans en Plasma.
  expect(await page.evaluate(() => getComputedStyle(document.body).fontSize)).toBe("13.3333px");
  expect(await alto(page.locator(".zona-soltar button").first())).toBe(32);
  expect(await ancho(page.locator(".lateral"))).toBe(240);
  const fila = page.getByRole("button", { name: "Lotes", exact: true });
  expect(await alto(fila)).toBe(30);
  // Los iconos de Breeze, no los de Yaru.
  await expect(fila.locator(".icono path").last()).toHaveAttribute("d", ICONOS_BREEZE.lotes.d);
  expect(await alto(page.locator(".herramientas").first())).toBe(46);
  // La fila elegida, rellena del acento.
  const elegida = page.getByRole("button", { name: "Estudio", exact: true });
  const [fondo, relleno] = await elegida.evaluate((e) => {
    const prueba = document.createElement("i");
    prueba.style.background = "var(--relleno)";
    document.body.append(prueba);
    const r = getComputedStyle(prueba).backgroundColor;
    prueba.remove();
    return [getComputedStyle(e).backgroundColor, r];
  });
  expect(fondo).toBe(relleno);
});

test("en KDE: los botones de la ventana son los de Breeze, en el lado que diga el sistema", async ({ page }) => {
  await page.goto(KDE);
  const derecha = page.locator(".lado-botones.derecha");
  await expect(derecha.getByRole("button")).toHaveCount(3);
  expect(await ancho(derecha.getByRole("button", { name: "Cerrar", exact: true }))).toBe(20);
  await expect(derecha.getByRole("button", { name: "Minimizar", exact: true }).locator("path").last()).toHaveAttribute(
    "d",
    ICONOS_BREEZE.minimizar.d,
  );
});

test("en KDE: Ajustes como un formulario, la etiqueta a la izquierda del control", async ({ page }) => {
  await page.goto(KDE);
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  const idioma = page.getByTestId("idioma");
  expect(await idioma.evaluate((e) => getComputedStyle(e).display)).toBe("grid");
  const titulo = await idioma.locator(".tarjeta-titulo").boundingBox();
  const control = await idioma.locator(".tarjeta-control").boundingBox();
  expect(titulo && control).toBeTruthy();
  expect(titulo!.x + titulo!.width).toBeLessThanOrEqual(control!.x);
  expect(Math.abs(titulo!.y + titulo!.height / 2 - (control!.y + control!.height / 2))).toBeLessThan(8);
  // El interruptor de Breeze: 36 de ancho, la pista de 12 y su botón de 18.
  const avisar = page.getByRole("switch", { name: "Avisarme cuando haya una versión nueva" });
  expect(await ancho(avisar)).toBe(36);
  expect(await alto(avisar)).toBe(12);
  expect(await ancho(avisar.locator("xpath=following-sibling::span[@class='pomo']"))).toBe(18);
});

test("en KDE con el marco: las esquinas, con el radio de Breeze", async ({ page }) => {
  await page.goto(`${KDE}&esquinas=si`);
  expect(await page.locator(".ventana").evaluate((v) => getComputedStyle(v).borderRadius)).toBe("5px");
});

test("en GNOME y los demás escritorios, nada de Breeze", async ({ page }) => {
  await page.goto("/?sistema=linux&escritorio=otro&barra=propia");
  expect(await page.evaluate(() => getComputedStyle(document.body).fontSize)).toBe("14.6667px");
  expect(await ancho(page.locator(".lateral"))).toBe(220);
});

test("en KDE: el botón de la barra lateral dice qué hará, plegar o desplegar", async ({ page }) => {
  await page.goto(KDE);
  const boton = page.getByTestId("boton-lateral");
  const icono = boton.locator(".icono path").last();
  await expect(icono).toHaveAttribute("d", ICONOS_BREEZE.lateralOcultar.d);
  await expect(boton.locator(".icono g")).toHaveAttribute("transform", ICONOS_BREEZE.lateralOcultar.t!);
  await boton.click();
  await expect(icono).toHaveAttribute("d", ICONOS_BREEZE.lateral.d);
  await boton.click();
});

test("en KDE: Agentes de IA en tarjetas, no en el formulario de los ajustes", async ({ page }) => {
  await page.goto(KDE);
  await page.getByRole("button", { name: "Ajustes", exact: true }).click();
  await page.getByRole("radio", { name: "Agentes de IA" }).click();
  const desktop = page.getByTestId("desktop");
  await expect(desktop).toHaveClass(/grupo/);
  await expect(page.getByTestId("agentes").locator(".tarjeta-ajuste")).toHaveCount(0);
  // La orden de Claude Code, a todo el ancho de su tarjeta.
  const orden = await page.getByTestId("orden-claude-code").boundingBox();
  const tarjeta = await page.getByTestId("orden-claude-code").locator("xpath=..").boundingBox();
  expect(orden!.width).toBeGreaterThan(tarjeta!.width - 30);
});

// ADR 0032: Recortar y Redimensionar, en una barra con el aspecto del sistema.
test("en KDE: la barra de encima de la imagen es de Kirigami, y la ventanita de Breeze", async ({ page }) => {
  await page.goto("/?sistema=linux&escritorio=kde&barra=propia");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../pruebas/corpus/foto.webp"));
  const barra = page.getByTestId("barra-imagen");
  await expect(barra).toBeVisible();
  expect(await barra.evaluate((b) => b.getBoundingClientRect().height)).toBe(40);
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
  expect(await ventana.evaluate((v) => getComputedStyle(v).borderRadius)).toBe("5px");
});
