import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import json from "../src/datos/release.json" with { type: "json" };
import { RUTAS } from "../src/i18n/rutas";
import { cookiesElegidas, sinGoogle } from "./comun";

const PAGINAS = Object.values(RUTAS).flatMap((r) => [r.es, r.en]);
const URLS = new Set(json.assets.map((a) => a.url));

// En reposo: con «reducir el movimiento» no hay animación de aparecer, que deja lo de abajo a
// media opacidad hasta que entra en pantalla (y axe mediría el contraste de ese momento).
test.use({ reducedMotion: "reduce" });

test.beforeEach(async ({ page }) => {
  await cookiesElegidas(page);
  await sinGoogle(page);
});

for (const ruta of PAGINAS) {
  test(`${ruta}: sin errores, SEO completo y accesible`, async ({ page }) => {
    const errores: string[] = [];
    page.on("console", (m) => m.type() === "error" && errores.push(m.text()));
    page.on("pageerror", (e) => errores.push(String(e)));
    await page.goto(ruta);
    expect(errores).toEqual([]);
    // SEO: título, descripción, canonical a sí misma, sus alternas y Open Graph.
    await expect(page).toHaveTitle(/Apolo/);
    expect((await page.locator('meta[name="description"]').getAttribute("content"))?.length).toBeGreaterThan(50);
    expect(await page.locator('link[rel="canonical"]').getAttribute("href")).toBe(`https://apolo.webcafeina.com${ruta}`);
    for (const l of ["es", "en", "x-default"]) await expect(page.locator(`link[rel="alternate"][hreflang="${l}"]`)).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveAttribute("content", /\/social\/apolo-(es|en)\.png$/);
    const bloques = (await page.locator('script[type="application/ld+json"]').allTextContents()).map((s) => JSON.parse(s));
    // El grafo (ADR 0034): Webcafeína, el sitio, la página, Apolo y su código; migas de pan fuera de la portada.
    const grafo: { "@type": string; breadcrumb?: unknown; url?: string }[] = bloques.find((b) => b["@graph"])?.["@graph"] ?? [];
    expect(grafo.map((n) => n["@type"]).sort()).toEqual(["Organization", "SoftwareApplication", "SoftwareSourceCode", "WebPage", "WebSite"]);
    const web = grafo.find((n) => n["@type"] === "WebPage")!;
    expect(web.url).toBe(`https://apolo.webcafeina.com${ruta}`);
    expect(Boolean(web.breadcrumb)).toBe(ruta !== "/" && ruta !== "/en/");
    // Accesibilidad (WCAG 2 AA), con axe.
    const r = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
    expect(r.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target).join(" ")}`)).toEqual([]);
    // Las descargas de la Release enlazan ficheros que existen en ella.
    for (const href of await page.locator('a[href*="/releases/download/"]').evaluateAll((as) => as.map((a) => a.getAttribute("href")!)))
      expect(URLS.has(href), href).toBe(true);
  });
}

test("ESP / ENG: el activo en negrita, y lleva a la misma página en el otro idioma", async ({ page }) => {
  await page.goto("/descargar/mac/");
  const esp = page.locator('.idioma a[hreflang="es"]');
  const eng = page.locator('.idioma a[hreflang="en"]');
  await expect(esp).toHaveAttribute("aria-current", "page");
  expect(Number(await esp.evaluate((a) => getComputedStyle(a).fontWeight))).toBeGreaterThanOrEqual(700);
  expect(Number(await eng.evaluate((a) => getComputedStyle(a).fontWeight))).toBeLessThan(700);
  await eng.click();
  await expect(page).toHaveURL(/\/en\/download\/mac\/$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
});

test("las preguntas enlazan a quien desarrolla Apolo, y los datos estructurados van sin marcas", async ({ page }) => {
  for (const ruta of [RUTAS.preguntas.es, RUTAS.preguntas.en]) {
    await page.goto(ruta);
    const a = page.locator('a[href*="linkedin.com/in/"]');
    await expect(a).toHaveText("Álvaro Cabezas Campos");
    expect(await page.locator('script[type="application/ld+json"]').allTextContents()).toEqual(expect.arrayContaining([expect.stringContaining("Álvaro Cabezas Campos")]));
  }
});

test("la 404, en los dos idiomas y sin indexar", async ({ page }) => {
  const r = await page.goto("/no-existe/");
  expect(r?.status()).toBe(404);
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute("content", "noindex");
  await expect(page.locator('link[rel="canonical"]')).toHaveCount(0);
});

test("sitemap y robots", async ({ request }) => {
  const mapa = await (await request.get("/sitemap.xml")).text();
  for (const ruta of PAGINAS) expect(mapa).toContain(`<loc>https://apolo.webcafeina.com${ruta}</loc>`);
  const robots = await (await request.get("/robots.txt")).text();
  expect(robots).toContain("Disallow: /apt/");
  expect(robots).toContain("Sitemap: https://apolo.webcafeina.com/sitemap.xml");
});

test("las tarjetas de la portada llevan a sus páginas", async ({ page }) => {
  await page.goto("/");
  const seis = page.locator("#mas-de-apolo .tarjeta-enlace");
  await expect(seis).toHaveCount(6);
  for (const ruta of [RUTAS.agentes.es, RUTAS.cli.es, RUTAS.guias.es, RUTAS.comparativas.es, RUTAS.preguntas.es, RUTAS.novedades.es]) {
    await expect(page.locator(`#mas-de-apolo a[href="${ruta}"]`)).toBeVisible();
  }
});

test("los menús de la cabecera abren, se cierran solos y llevan a su página", async ({ page }) => {
  await page.goto("/");
  const [descargas, usar] = [page.locator(".desplegable").nth(0), page.locator(".desplegable").nth(1)];
  await descargas.locator("summary").click();
  await expect(descargas.locator(".panel a").first()).toBeVisible();
  await usar.locator("summary").click();
  await expect(descargas.locator(".panel a").first()).toBeHidden();
  await page.keyboard.press("Escape");
  await expect(usar.locator(".panel a").first()).toBeHidden();
  await usar.locator("summary").click();
  await usar.locator(`.panel a[href="${RUTAS.cli.es}"]`).click();
  await expect(page).toHaveURL(new RegExp(`${RUTAS.cli.es}$`));
});

test("en el móvil hay menú, y lleva a las páginas", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 800 });
  await page.goto("/en/");
  await expect(page.locator(".nav")).toBeHidden();
  await page.locator(".menu-movil summary").click();
  await page.locator(`.panel-movil a[href="${RUTAS.comparativas.en}"]`).click();
  await expect(page).toHaveURL(new RegExp(`${RUTAS.comparativas.en}$`));
});

test("en el móvil caben el logo, el botón de descargar y el menú, sin solaparse ni salirse", async ({ page }) => {
  for (const [ruta, width] of [["/", 320], ["/", 375], ["/", 390], ["/", 820], ["/en/", 320], ["/en/", 390]] as const) {
    await page.setViewportSize({ width, height: 800 });
    await page.goto(ruta);
    const boton = page.locator(".boton-cabecera");
    const menu = page.locator(".menu-movil summary");
    await expect(page.locator(".logo span")).toBeVisible();
    await expect(boton).toBeVisible();
    await expect(menu).toBeVisible();
    await expect(page.locator(".idioma")).toBeHidden();
    const [l, b, m] = await Promise.all([page.locator(".logo").boundingBox(), boton.boundingBox(), menu.boundingBox()]);
    expect(l!.x + l!.width).toBeLessThanOrEqual(b!.x);
    expect(b!.x + b!.width).toBeLessThanOrEqual(m!.x);
    expect(m!.x + m!.width).toBeLessThanOrEqual(width);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
  }
});

test("en el móvil, el idioma está en el menú: el activo marcado, y lleva a la misma página en el otro", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 800 });
  await page.goto("/descargar/mac/");
  await page.locator(".menu-movil summary").click();
  await expect(page.locator('.idioma-movil a[hreflang="es"]')).toHaveAttribute("aria-current", "page");
  for (const colorScheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme });
    const { violations } = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).include(".panel-movil").analyze();
    expect(violations.map((v) => v.id)).toEqual([]);
  }
  await page.locator('.idioma-movil a[hreflang="en"]').click();
  await expect(page).toHaveURL(/\/en\/download\/mac\/$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
});

test("copiar una orden", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/descargar/linux/");
  const b = page.locator('.tarjeta [data-comando="apt"]').first();
  await b.click();
  await expect(b).toHaveText("Copiado");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toContain("sudo apt install apolo");
});

test("el comparador: deslizar con el teclado y cambiar de formato", async ({ page }) => {
  await page.goto("/");
  const rango = page.locator('[data-comparador] input[type="range"]');
  await rango.focus();
  await page.keyboard.press("ArrowLeft");
  expect(await page.locator("[data-comparador]").evaluate((c) => (c as HTMLElement).style.getPropertyValue("--corte"))).toBe("49.5%");
  await page.locator('[data-comparador] .formatos [data-formato="webp"]').click();
  await expect(page.locator("[data-comparador]")).toHaveAttribute("data-formato", "webp");
  await expect(page.locator('[data-comparador] .cifras[data-formato="webp"]')).toBeVisible();
});
