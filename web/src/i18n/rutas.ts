// Cada página en los dos idiomas (ADR 0033): para los botones ESP / ENG, el
// hreflang y el sitemap. Los slugs van traducidos. Una página nueva se añade
// aquí con su pareja; pruebas/rutas.test.ts comprueba que existen las dos.

export type Idioma = "es" | "en";

export const RUTAS = {
  inicio: { es: "/", en: "/en/" },
  descargarMac: { es: "/descargar/mac/", en: "/en/download/mac/" },
  descargarWindows: { es: "/descargar/windows/", en: "/en/download/windows/" },
  descargarLinux: { es: "/descargar/linux/", en: "/en/download/linux/" },
  preguntas: { es: "/preguntas/", en: "/en/faq/" },
  novedades: { es: "/novedades/", en: "/en/whats-new/" },
  cli: { es: "/linea-de-comandos/", en: "/en/cli/" },
  agentes: { es: "/agentes/", en: "/en/agents/" },
  cookies: { es: "/cookies/", en: "/en/cookies/" },
  // Las guías y las comparativas (ADR 0034): un índice y una página por cada una.
  guias: { es: "/guias/", en: "/en/guides/" },
  guiaWebp: { es: "/guias/convertir-a-webp/", en: "/en/guides/convert-to-webp/" },
  guiaFormatos: { es: "/guias/avif-webp-o-jpeg-xl/", en: "/en/guides/avif-webp-or-jpeg-xl/" },
  guiaSinPerder: { es: "/guias/comprimir-imagenes-sin-perder-calidad/", en: "/en/guides/compress-images-without-losing-quality/" },
  guiaWeb: { es: "/guias/optimizar-imagenes-para-la-web/", en: "/en/guides/optimize-images-for-the-web/" },
  guiaAgente: { es: "/guias/optimizar-imagenes-con-un-agente-de-ia/", en: "/en/guides/optimize-images-with-an-ai-agent/" },
  guiaHeic: { es: "/guias/heic-a-jpg/", en: "/en/guides/heic-to-jpg/" },
  comparativas: { es: "/comparativas/", en: "/en/compare/" },
  frenteSquoosh: { es: "/comparativas/apolo-frente-a-squoosh/", en: "/en/compare/apolo-vs-squoosh/" },
  frenteTinypng: { es: "/comparativas/apolo-frente-a-tinypng/", en: "/en/compare/apolo-vs-tinypng/" },
  frenteImageoptim: { es: "/comparativas/apolo-frente-a-imageoptim/", en: "/en/compare/apolo-vs-imageoptim/" },
  frenteCaesium: { es: "/comparativas/apolo-frente-a-caesium/", en: "/en/compare/apolo-vs-caesium/" },
  // Todas las páginas, para personas (el sitemap.xml es para los buscadores).
  mapa: { es: "/mapa-del-sitio/", en: "/en/sitemap/" },
} as const satisfies Record<string, Record<Idioma, string>>;

export type Pagina = keyof typeof RUTAS;

export const ruta = (pagina: Pagina, idioma: Idioma) => RUTAS[pagina][idioma];
export const otro = (idioma: Idioma): Idioma => (idioma === "es" ? "en" : "es");
