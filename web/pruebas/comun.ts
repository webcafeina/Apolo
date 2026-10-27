// Lo común de las pruebas de la web: navegadores de verdad, sin red a Google,
// y la elección de cookies ya hecha cuando la prueba no va de eso.
import type { BrowserContext, Page } from "@playwright/test";

export const UA = {
  mac: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15",
  windows: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0",
  linux: "Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0",
  linuxArm: "Mozilla/5.0 (X11; Linux aarch64; rv:144.0) Gecko/20100101 Firefox/144.0",
  iphone: "Mozilla/5.0 (iPhone; CPU iPhone OS 27_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Mobile/15E148 Safari/604.1",
};

/** Sin Chromium de verdad detrás: que la página no lea userAgentData (como Safari y Firefox). */
export async function sinUserAgentData(ctx: BrowserContext | Page) {
  await ctx.addInitScript(() => Object.defineProperty(Navigator.prototype, "userAgentData", { get: () => undefined }));
}

/** Las peticiones a Google, interceptadas: devuelven vacío y se cuentan. */
export async function sinGoogle(page: Page) {
  const pedidas: string[] = [];
  await page.route(/googletagmanager\.com|google-analytics\.com/, (r) => {
    pedidas.push(r.request().url());
    return r.fulfill({ status: 200, contentType: "text/javascript", body: "" });
  });
  return pedidas;
}

/** La elección de cookies ya hecha (rechazada), para las pruebas que no van del aviso. */
export async function cookiesElegidas(ctx: BrowserContext | Page) {
  await ctx.addInitScript(() => localStorage.setItem("apolo-consentimiento", JSON.stringify({ v: 1, analitica: false, fecha: Date.now() })));
}
