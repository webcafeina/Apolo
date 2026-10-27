// La web de Apolo (ADR 0033): estática, en español en la raíz y en inglés en
// /en/. Las imágenes ya vienen optimizadas por Apolo (herramientas/), así que
// Astro no las toca.
import { defineConfig, passthroughImageService } from "astro/config";

export default defineConfig({
  site: "https://apolo.webcafeina.com",
  trailingSlash: "always",
  build: { format: "directory" },
  image: { service: passthroughImageService() },
  // No hay Markdown con código resaltado en la web (las notas pasan por marked):
  // sin Shiki, que pone estilos en línea y choca con la CSP.
  markdown: { syntaxHighlight: false },
  i18n: { locales: ["es", "en"], defaultLocale: "es", routing: { prefixDefaultLocale: false } },
  // La CSP de cada página (ADR 0034), en un <meta> con los hashes de sus
  // scripts que calcula Astro al compilar. GTM y GA4, sus dominios. Los estilos
  // admiten 'unsafe-inline' (el comparador fija su proporción con style=): lo
  // que importa cerrar son los scripts. frame-ancestors no cabe en un <meta>:
  // va como cabecera en Cloudflare, con el resto de cabeceras.
  security: {
    csp: {
      algorithm: "SHA-256",
      directives: [
        "default-src 'self'",
        "img-src 'self' data: https://*.google-analytics.com https://*.googletagmanager.com",
        "connect-src 'self' https://*.google-analytics.com https://*.analytics.google.com https://*.googletagmanager.com",
        "font-src 'self'",
        "media-src 'self'",
        "object-src 'none'",
        "base-uri 'self'",
        "form-action 'none'",
      ],
      scriptDirective: { resources: ["'self'", "https://www.googletagmanager.com"] },
      styleDirective: { resources: ["'self'", "'unsafe-inline'"] },
    },
  },
});
