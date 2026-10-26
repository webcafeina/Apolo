// La web de Apolo (ADR 0033): estática, en español en la raíz y en inglés en
// /en/. Las imágenes ya vienen optimizadas por Apolo (herramientas/), así que
// Astro no las toca.
import { defineConfig, passthroughImageService } from "astro/config";

export default defineConfig({
  site: "https://apolo.webcafeina.com",
  trailingSlash: "always",
  build: { format: "directory" },
  image: { service: passthroughImageService() },
  i18n: { locales: ["es", "en"], defaultLocale: "es", routing: { prefixDefaultLocale: false } },
});
