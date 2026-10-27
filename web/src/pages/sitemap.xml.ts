// El mapa del sitio (ADR 0033), con cada página en los dos idiomas y sus
// alternas (hreflang). A mano y no con @astrojs/sitemap: los slugs van
// traducidos y rutas.ts ya sabe emparejarlos. La fecha de cada página
// (lastmod, ADR 0034) es la de la versión publicada: la web se compila con ella.
import type { APIRoute } from "astro";
import { release } from "../datos/version";
import { RUTAS } from "../i18n/rutas";

export const GET: APIRoute = ({ site }) => {
  const url = (r: string) => new URL(r, site).href;
  const entradas = Object.values(RUTAS).flatMap((par) =>
    (["es", "en"] as const).map(
      (i) => `  <url>
    <loc>${url(par[i])}</loc>
    <lastmod>${release.fecha.slice(0, 10)}</lastmod>
    <xhtml:link rel="alternate" hreflang="es" href="${url(par.es)}"/>
    <xhtml:link rel="alternate" hreflang="en" href="${url(par.en)}"/>
    <xhtml:link rel="alternate" hreflang="x-default" href="${url(par.en)}"/>
  </url>`,
    ),
  );
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${entradas.join("\n")}
</urlset>
`;
  return new Response(xml, { headers: { "Content-Type": "application/xml; charset=utf-8" } });
};
