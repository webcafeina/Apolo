// El Estudio de punta a punta: abrir, ver la vista previa, cambiar ajustes,
// pegar una orden, guardar un preset, enderezar y exportar.

import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const corpus = join(import.meta.dirname, "../../pruebas/corpus");

async function abrir(page: Page, fichero: string) {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, fichero));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
}

test.use({ testIdAttribute: "data-prueba" });

test("abrir una imagen enseña el resultado y la orden cwebp", async ({ page }) => {
  await abrir(page, "foto.webp");
  await expect(page.getByText("foto.webp", { exact: true })).toBeVisible();
  await expect(page.getByTestId("orden")).toHaveText("cwebp foto.webp -o foto-apolo.webp");
  await expect(page.getByTestId("ahorro")).toBeVisible();
  // El lienzo pinta algo: no es todo del mismo color.
  const distintos = await page.getByTestId("lienzo").evaluate((c: HTMLCanvasElement) => {
    const d = c.getContext("2d")!.getImageData(0, 0, c.width, c.height).data;
    const vistos = new Set<number>();
    for (let i = 0; i < d.length; i += 4 * 97) vistos.add((d[i] << 16) | (d[i + 1] << 8) | d[i + 2]);
    return vistos.size;
  });
  expect(distintos).toBeGreaterThan(50);
});

test("cambiar la calidad cambia el resultado y la orden", async ({ page }) => {
  await abrir(page, "foto.webp");
  const antes = await page.getByTestId("peso-resultado").textContent();
  await page.locator("#control-calidad").fill("20");
  await expect(page.getByTestId("orden")).toHaveText("cwebp -q 20 foto.webp -o foto-apolo.webp");
  await expect(page.getByTestId("peso-resultado")).not.toHaveText(antes!);
});

test("pegar una orden cwebp carga sus ajustes", async ({ page }) => {
  await abrir(page, "foto.webp");
  await page.getByRole("button", { name: "Pegar orden…" }).click();
  await page.getByTestId("orden-entrada").fill("cwebp -preset drawing -lossless -z 9 a.png -o b.webp");
  await page.getByRole("button", { name: "Cargar" }).click();
  // Sin pérdida con -z 9 tarda 4 s en la compilación de depuración de
  // apolo-dev, y más con la máquina cargada: los 5 s de siempre no llegan.
  await expect(page.getByTestId("orden")).toContainText("-preset drawing", { timeout: 30_000 });
  await expect(page.getByTestId("orden")).toContainText("-lossless");
  await expect(page.getByRole("switch", { name: "Sin pérdida" })).toBeChecked();
});

test("un preset guardado aparece en su sección, se aplica al Estudio, se renombra y se borra", async ({ page }) => {
  await abrir(page, "foto.webp");
  await page.locator("#control-calidad").fill("63");
  await page.getByTestId("nombre-preset").fill("Para la web");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByText("Preset «Para la web» guardado.")).toBeVisible();
  await page.locator("#control-calidad").fill("90");
  // Desde el Estudio, «Partir de».
  await page.getByTestId("partida").selectOption("apolo:Para la web");
  await expect(page.getByTestId("orden")).toContainText("-q 63");
  // La sección Presets.
  await page.locator("#control-calidad").fill("90");
  await page.getByRole("button", { name: "Presets" }).click();
  const ficha = page.getByTestId("preset");
  await expect(ficha).toContainText("Para la web");
  await expect(ficha).toContainText("Calidad 63");
  await expect(ficha).toContainText('apolo webp -apolo_preset "Para la web"');
  await expect(ficha).toContainText("-q 63");
  await ficha.getByRole("button", { name: "Usar en el Estudio" }).click();
  await expect(page.getByTestId("orden")).toContainText("-q 63");
  await page.getByRole("button", { name: "Presets" }).click();
  await ficha.getByRole("button", { name: "Renombrar" }).click();
  await ficha.getByRole("textbox").fill("Web ligera");
  await ficha.getByRole("button", { name: "Aceptar" }).click();
  await expect(ficha.getByRole("heading")).toHaveText("Web ligera");
  // Borrar pide confirmación con un segundo clic.
  await ficha.getByRole("button", { name: "Borrar" }).click();
  await ficha.getByRole("button", { name: "¿Borrarlo?" }).click();
  await expect(page.getByTestId("presets-vacio")).toBeVisible();
});

test("una foto girada avisa, y enderezarla cambia las dimensiones y la orden", async ({ page }) => {
  await abrir(page, "orientacion-6.png");
  await expect(page.getByTestId("aviso-orientacion")).toBeVisible();
  await expect(page.getByTestId("barra-estado")).toContainText("64 × 32");
  await page.getByRole("button", { name: "Enderezar" }).click();
  await expect(page.getByTestId("aviso-orientacion")).toBeHidden();
  await expect(page.getByTestId("barra-estado")).toContainText("32 × 64");
  await expect(page.getByTestId("no-equivalente")).toHaveText("Enderezada: cwebp no la gira");
  await expect(page.getByTestId("motivo")).toContainText("cwebp no gira las fotos");
  await expect(page.getByRole("switch", { name: "Enderezar según EXIF" })).toBeChecked();
});

test("exportar descarga el WebP", async ({ page }) => {
  await abrir(page, "foto.webp");
  const descarga = page.waitForEvent("download");
  await page.getByTestId("exportar").click();
  const d = await descarga;
  // Un .webp no se propone con su nombre: se sobrescribiría el original.
  expect(d.suggestedFilename()).toBe("foto-apolo.webp");
  const datos = readFileSync(await d.path());
  expect(datos.subarray(0, 4).toString()).toBe("RIFF");
  expect(datos.subarray(8, 12).toString()).toBe("WEBP");
});

test("los niveles se pliegan y se recuerda cómo quedaron", async ({ page }) => {
  await abrir(page, "foto.webp");
  const experto = page.getByTestId("nivel-experto");
  await expect(experto).not.toHaveAttribute("open");
  await experto.locator("summary").click();
  await expect(experto).toHaveAttribute("open");
  // El atributo cambia con el clic, pero se guarda después: con el evento
  // «toggle», que llega en otra vuelta, y un efecto de React. Recargar sin
  // esperar a eso falló una vez en CI.
  await expect
    .poll(() => page.evaluate(() => localStorage.getItem("apolo.niveles")))
    .toContain('"experto":true');
  await page.reload();
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(corpus, "foto.webp"));
  await expect(page.getByTestId("nivel-experto")).toHaveAttribute("open");
});

test("la marca: firma con la versión, y sin vidrio el fondo es opaco", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("firma")).toContainText(/Webcafeína.*\d+\.\d+\.\d+/);
  // En el navegador no hay vidrio: si el body fuera transparente, se vería lo
  // que haya detrás (ADR 0016). Es lo mismo que vigila Esfinge.
  await expect(page.locator("html")).toHaveAttribute("data-vidrio", "no");
  const fondo = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  expect(fondo).not.toBe("rgba(0, 0, 0, 0)");
});

test("el zoom del comparador no mueve la ventana, y la vista reducida lo avisa", async ({ page }) => {
  await abrir(page, "foto.webp");
  // La rueda sobre el lienzo la usa el comparador y nadie más.
  const cancelada = await page.getByTestId("lienzo").evaluate((c) => {
    const e = new WheelEvent("wheel", { deltaY: 100, bubbles: true, cancelable: true });
    c.dispatchEvent(e);
    return e.defaultPrevented;
  });
  expect(cancelada).toBe(true);
  // Alejando por debajo del 100 % sale el aviso, que lleva al 100 %.
  for (let i = 0; i < 8; i++) await page.getByRole("button", { name: "Alejar" }).click();
  const nota = page.getByTestId("nota-reducida");
  await expect(nota).toContainText("clica aquí para ver al 100 %");
  await nota.click();
  await expect(page.getByTestId("zoom-100")).toHaveText("100 %");
  await expect(nota).toBeHidden();
});

test("abre una foto HEIC, y la orden cwebp avisa de que cwebp no la lee", async ({ page }) => {
  await page.goto("/");
  const elegir = page.waitForEvent("filechooser");
  await page.getByRole("button", { name: "Abrir una imagen…" }).click();
  await (await elegir).setFiles(join(import.meta.dirname, "../../crates/heic/vendor/libheif/examples/example.heic"));
  await expect(page.getByTestId("peso-resultado")).not.toHaveText("…");
  await expect(page.getByText("HEIC · 1280 × 854")).toBeVisible();
  await expect(page.getByTestId("no-equivalente")).toHaveText("cwebp no abre HEIC");
  await expect(page.getByTestId("motivo")).toContainText("cwebp no lee HEIC");
});

test("cada formato enseña la orden de su herramienta, y QOI no tiene opciones", async ({ page }) => {
  await abrir(page, "foto.webp");
  const formato = page.getByTestId("formato");
  await formato.selectOption("jpeg");
  await expect(page.getByTestId("orden")).toContainText("cjpeg -outfile foto.jpg foto.webp");
  await expect(page.getByTestId("pesos")).toContainText("Resultado · JPEG");
  // cjpeg no lee WebP: lo dice la orden.
  await expect(page.getByTestId("no-equivalente")).toContainText("cjpeg no abre WebP");
  await page.locator("#control-calidadJpeg").fill("40");
  await expect(page.getByTestId("orden")).toContainText("cjpeg -quality 40");
  await formato.selectOption("png");
  await expect(page.getByTestId("orden")).toContainText("oxipng --out foto.png foto.webp");
  await formato.selectOption("qoi");
  await expect(page.getByTestId("orden")).toContainText("qoiconv foto.webp foto.qoi");
  await expect(page.getByText("QOI no tiene opciones")).toBeVisible();
});

test("AVIF y JPEG XL enseñan la orden de avifenc y cjxl, y sus controles la cambian", async ({ page }) => {
  await abrir(page, "foto.webp");
  const formato = page.getByTestId("formato");
  await formato.selectOption("avif");
  await expect(page.getByTestId("orden")).toContainText("avifenc foto.webp foto.avif");
  await expect(page.getByTestId("pesos")).toContainText("Resultado · AVIF", { timeout: 30_000 });
  await expect(page.getByTestId("no-equivalente")).toContainText("avifenc no abre WebP");
  await page.locator("#control-calidadAvif").fill("40");
  await expect(page.getByTestId("orden")).toContainText("avifenc -q 40 foto.webp foto.avif");
  await formato.selectOption("jxl");
  await expect(page.getByTestId("orden")).toContainText("cjxl foto.webp foto.jxl");
  await expect(page.getByTestId("pesos")).toContainText("Resultado · JPEG XL", { timeout: 30_000 });
  await page.locator("#control-esfuerzoJxl").fill("3");
  await expect(page.getByTestId("orden")).toContainText("cjxl foto.webp foto.jxl -e 3");
  // Sin un JPEG de entrada, recomprimir sin pérdida no cuenta.
  await expect(page.getByTestId("control-jpegSinPerdidaJxl")).toHaveClass(/apagado/);
});

test("un AVIF y un JPEG XL exportados se vuelven a abrir", async ({ page }, info) => {
  for (const [f, nombre] of [
    ["avif", "AVIF"],
    ["jxl", "JPEG XL"],
  ]) {
    await abrir(page, "foto.webp");
    await page.getByTestId("formato").selectOption(f);
    await expect(page.getByTestId("pesos")).toContainText(`Resultado · ${nombre}`, { timeout: 30_000 });
    const descarga = page.waitForEvent("download");
    await page.getByTestId("exportar").click();
    const d = await descarga;
    expect(d.suggestedFilename()).toBe(`foto.${f}`);
    const ruta = info.outputPath(`foto.${f}`);
    await d.saveAs(ruta);
    const elegir = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Abrir otra…" }).click();
    await (await elegir).setFiles(ruta);
    await expect(page.getByText(`${nombre} · 128 × 128`)).toBeVisible({ timeout: 30_000 });
  }
});

test("pegar una orden de cjxl pasa a JPEG XL con sus opciones", async ({ page }) => {
  await abrir(page, "foto.webp");
  await page.getByRole("button", { name: "Pegar orden…" }).click();
  await page.getByTestId("orden-entrada").fill("cjxl a.png b.jxl -q 80 -e 4 --resampling=2");
  await page.getByRole("button", { name: "Cargar" }).click();
  await expect(page.getByTestId("formato")).toHaveValue("jxl");
  await expect(page.getByTestId("orden")).toContainText("cjxl foto.webp foto.jxl -q 80 -e 4 --resampling=2", {
    timeout: 30_000,
  });
});

test("pegar una orden de cjpeg pasa a JPEG con sus opciones", async ({ page }) => {
  await abrir(page, "foto.webp");
  await page.getByRole("button", { name: "Pegar orden…" }).click();
  await page.getByTestId("orden-entrada").fill("cjpeg -quality 55 -grayscale -outfile x.jpg a.png");
  await page.getByRole("button", { name: "Cargar" }).click();
  await expect(page.getByTestId("formato")).toHaveValue("jpeg");
  await expect(page.getByTestId("orden")).toContainText("cjpeg -quality 55 -grayscale", { timeout: 30_000 });
});

test("se comparan dos formatos, uno a cada lado", async ({ page }) => {
  await abrir(page, "foto.webp");
  await expect(page.getByTestId("rotulo-izquierda")).toHaveText("Original");
  await page.getByTestId("lados").getByRole("radio", { name: /Izquierda/ }).click();
  await page.getByTestId("comparar-formato").click();
  // El izquierdo empieza en otro formato que el derecho (WebP), con su peso.
  await expect(page.getByTestId("rotulo-izquierda")).toContainText("JPEG ·", { timeout: 30_000 });
  await expect(page.getByTestId("rotulo-derecha")).toContainText("WebP ·");
  // La orden y los pesos son del lado que se edita.
  await expect(page.getByTestId("orden")).toContainText("cjpeg");
  await page.getByTestId("lados").getByRole("radio", { name: /Derecha/ }).click();
  await expect(page.getByTestId("orden")).toContainText("cwebp");
  // Y se puede volver al original.
  await page.getByTestId("lados").getByRole("radio", { name: /Izquierda/ }).click();
  await page.getByRole("button", { name: "Volver al original" }).click();
  await expect(page.getByTestId("rotulo-izquierda")).toHaveText("Original");
});

test("el proceso redimensiona, en cualquier formato, y la orden lo dice", async ({ page }) => {
  await abrir(page, "foto.webp");
  await page.getByTestId("formato").selectOption("png");
  await page.getByRole("switch", { name: "Redimensionar" }).first().check();
  // La foto mide 128: a la mitad por defecto.
  await expect(page.getByTestId("barra-estado")).toContainText("64 × 64", { timeout: 30_000 });
  await expect(page.getByTestId("no-equivalente")).toContainText("Procesada por Apolo");
});
