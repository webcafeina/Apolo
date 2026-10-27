import { expect, test } from "@playwright/test";
import { sinGoogle } from "./comun";

// Consent Mode v2 en modo básico: nada de Google hasta que se acepta.
test("la primera vez: el aviso, todo denegado y GTM sin cargar", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await expect(page.locator("[data-cookies]")).toBeVisible();
  const capa = await page.evaluate(() => JSON.parse(JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer)));
  expect(capa[0]).toMatchObject({ 0: "consent", 1: "default", 2: { analytics_storage: "denied", ad_storage: "denied" } });
  expect(pedidas).toEqual([]);
});

test("aceptar: concede la analítica, carga GTM y no vuelve a preguntar", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Aceptar" }).click();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  await expect.poll(() => pedidas.some((u) => u.includes("gtm.js?id=GTM-PRUEBA"))).toBe(true);
  await page.reload();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  const capa = await page.evaluate(() => JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer));
  expect(capa).toContain('"analytics_storage":"granted"');
});

test("rechazar: no carga nada, y se puede cambiar desde el pie", async ({ page }) => {
  const pedidas = await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Rechazar" }).click();
  await page.reload();
  await expect(page.locator("[data-cookies]")).toBeHidden();
  expect(pedidas).toEqual([]);
  await page.getByRole("button", { name: "Preferencias de cookies" }).click();
  await expect(page.locator("[data-cookies]")).toBeVisible();
  await page.getByRole("button", { name: "Configurar" }).click();
  await page.getByRole("checkbox", { name: /Analítica/ }).check();
  await page.getByRole("button", { name: "Guardar la elección" }).click();
  await expect.poll(() => pedidas.some((u) => u.includes("gtm.js"))).toBe(true);
});

test("la CSP deja trabajar a GTM y GA4 y bloquea lo demás", async ({ page }) => {
  // Lo que hacen de verdad al aceptar: gtm.js trae gtag.js, y gtag.js manda la visita por
  // imagen y por fetch. Un script de otro dominio, en cambio, no debe cargar (ADR 0034).
  const violaciones: string[] = [];
  await page.exposeFunction("violacion", (v: string) => violaciones.push(v));
  await page.addInitScript(() =>
    document.addEventListener("securitypolicyviolation", (e) =>
      (window as unknown as { violacion: (v: string) => void }).violacion(`${e.effectiveDirective} ${e.blockedURI}`),
    ),
  );
  await page.route(/googletagmanager\.com\/gtm\.js/, (r) =>
    r.fulfill({
      contentType: "text/javascript",
      body: 'var s=document.createElement("script");s.src="https://www.googletagmanager.com/gtag/js?id=G-PRUEBA";document.head.appendChild(s);',
    }),
  );
  await page.route(/googletagmanager\.com\/gtag/, (r) =>
    r.fulfill({
      contentType: "text/javascript",
      body: 'new Image().src="https://region1.google-analytics.com/g/collect?v=2";fetch("https://region1.analytics.google.com/g/collect?v=2",{method:"POST",mode:"no-cors"});window.gtagCargado=true;',
    }),
  );
  await page.route(/google-analytics\.com|analytics\.google\.com|ajeno\.example/, (r) => r.fulfill({ status: 204 }));
  await page.goto("/");
  await page.getByRole("button", { name: "Aceptar" }).click();
  await expect.poll(() => page.evaluate(() => (window as unknown as { gtagCargado?: boolean }).gtagCargado)).toBe(true);
  await page.evaluate(() => {
    const s = document.createElement("script");
    s.src = "https://ajeno.example/x.js";
    document.head.appendChild(s);
  });
  await expect.poll(() => violaciones.length).toBeGreaterThan(0);
  expect(violaciones).toEqual(["script-src-elem https://ajeno.example/x.js"]);
});

test("las descargas cuentan en la capa de datos", async ({ page }) => {
  await sinGoogle(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Rechazar" }).click();
  // Que el clic no se vaya de la página: el enlace de la tarjeta de macOS.
  await page.route(/github\.com/, (r) => r.fulfill({ status: 204 }));
  await page.locator('.tarjeta[data-para="macos"] a.descargar').click();
  const capa = await page.evaluate(() => JSON.stringify((window as unknown as { dataLayer: unknown[] }).dataLayer));
  expect(capa).toContain('"event":"descarga_apolo"');
  expect(capa).toContain('"sistema":"macos"');
});
