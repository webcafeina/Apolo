import { expect, test } from "@playwright/test";
import { UA, cookiesElegidas, sinGoogle, sinUserAgentData } from "./comun";

// Lo que se ve primero, según el ordenador de quien visita.
for (const [nombre, ua, boton, tarjeta, vista] of [
  ["Mac", UA.mac, "Descargar para Mac", "macos", "macos"],
  ["Windows", UA.windows, "Descargar para Windows", "windows", "windows"],
  ["Linux", UA.linux, "Descargar para Linux", "linux", "gnome"],
] as const) {
  test(`${nombre}: su botón, su tarjeta primero y sus capturas`, async ({ browser }) => {
    const ctx = await browser.newContext({ userAgent: ua });
    await sinUserAgentData(ctx);
    await cookiesElegidas(ctx);
    const page = await ctx.newPage();
    await sinGoogle(page);
    await page.goto("/");
    await expect(page.locator(".cta:visible")).toHaveText(boton);
    await expect(page.locator("html")).toHaveAttribute("data-vista", vista);
    // La tarjeta de su sistema va la primera (order: -1) y resaltada.
    const orden = await page.locator(".tarjeta").evaluateAll((ts) =>
      ts.map((t) => [t.getAttribute("data-para"), getComputedStyle(t).order] as const),
    );
    expect(orden.find(([p]) => p === tarjeta)?.[1]).toBe("-1");
    await expect(page.locator(`.tarjeta[data-para="${tarjeta}"] .tuya`)).toBeVisible();
    await expect(page.locator(`#sistema .vista[data-vista="${vista}"]`)).toBeVisible();
    await ctx.close();
  });
}

test("el sistema está puesto antes de pintar: sin parpadeo", async ({ browser }) => {
  const ctx = await browser.newContext({ userAgent: UA.windows });
  await sinUserAgentData(ctx);
  await cookiesElegidas(ctx);
  const page = await ctx.newPage();
  await sinGoogle(page);
  // Un script justo al abrir <body>, que corre antes de pintar nada del cuerpo: lo que ve.
  await page.route("**/", async (r) => {
    const res = await r.fetch();
    const html = (await res.text()).replace(
      /<body([^>]*)>/,
      '<body$1><script>window.alPintar = document.documentElement.getAttribute("data-plataforma")</script>',
    );
    await r.fulfill({ response: res, body: html });
  });
  await page.goto("/");
  expect(await page.evaluate(() => (window as unknown as { alPintar: string | null }).alPintar)).toBe("windows");
  await ctx.close();
});

test("Linux ARM, por el UA de Firefox", async ({ browser }) => {
  const ctx = await browser.newContext({ userAgent: UA.linuxArm });
  await sinUserAgentData(ctx);
  await cookiesElegidas(ctx);
  const page = await ctx.newPage();
  await sinGoogle(page);
  await page.goto("/");
  await expect(page.locator("html")).toHaveAttribute("data-arquitectura", "arm64");
  await expect(page.locator('#linux .por-distro[data-distro="ubuntu"] a.por-arq:visible')).toHaveAttribute("href", /_arm64\.deb$/);
  await ctx.close();
});

test("lo que se elige se recuerda al volver", async ({ browser }) => {
  const ctx = await browser.newContext({ userAgent: UA.linux });
  await sinUserAgentData(ctx);
  await cookiesElegidas(ctx);
  const page = await ctx.newPage();
  await sinGoogle(page);
  await page.goto("/");
  await page.locator('#sistema [data-elige="vista"] [data-valor="kde"]').click();
  await page.locator('#linux [data-elige="distro"] [data-valor="fedora"]').click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-vista", "kde");
  await expect(page.locator('#linux .por-distro[data-distro="fedora"]')).toBeVisible();
  await expect(page.locator('#linux [data-elige="distro"] [data-valor="fedora"]')).toHaveAttribute("aria-checked", "true");
  await ctx.close();
});

test("en el móvil, el enlace para el ordenador en vez de descargar", async ({ browser }) => {
  const ctx = await browser.newContext({ userAgent: UA.iphone, viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });
  await cookiesElegidas(ctx);
  const page = await ctx.newPage();
  await sinGoogle(page);
  await page.goto("/");
  await expect(page.locator(".movil")).toBeVisible();
  await expect(page.locator(".cta:visible")).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await ctx.close();
});
