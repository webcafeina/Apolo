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
    for (const s of await page.locator('script[type="application/ld+json"]').allTextContents()) JSON.parse(s);
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
