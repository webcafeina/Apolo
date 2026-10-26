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
} as const satisfies Record<string, Record<Idioma, string>>;

export type Pagina = keyof typeof RUTAS;

export const ruta = (pagina: Pagina, idioma: Idioma) => RUTAS[pagina][idioma];
export const otro = (idioma: Idioma): Idioma => (idioma === "es" ? "en" : "es");
