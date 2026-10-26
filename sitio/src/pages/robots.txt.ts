// robots.txt (ADR 0033): todo se puede indexar salvo los repositorios de
// paquetes, que solo gastarían ancho de banda (cada .deb y .rpm pesa ~25 MB).
import type { APIRoute } from "astro";

export const GET: APIRoute = ({ site }) =>
  new Response(
    `User-agent: *
Disallow: /apt/
Disallow: /rpm/
Disallow: /aur/

Sitemap: ${new URL("/sitemap.xml", site).href}
`,
    { headers: { "Content-Type": "text/plain; charset=utf-8" } },
  );
