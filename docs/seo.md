# SEO y posicionamiento en IAs: qué se mide y cada cuánto

Lo que se hizo está en la ADR 0034. Aquí, cómo se sigue: una revisión al mes, apuntada al final.

## Dónde mirar

| Qué | Dónde |
|---|---|
| Búsquedas, clics, posición y páginas indexadas | Search Console, propiedad `https://apolo.webcafeina.com/` |
| Lo mismo en Bing (lo que usa ChatGPT para buscar) | Bing Webmaster Tools |
| Visitas desde asistentes de IA | GA4 › Adquisición de tráfico, canal **AI Assistant** (viene en el grupo predeterminado de Google; no hace falta uno propio) |
| Descargas desde la web | GA4, evento clave `descarga_apolo`, con las dimensiones `sistema`, `arquitectura`, `formato` y `origen` |
| Descargas totales | La insignia de descargas del README (todas las Releases) |
| El registro de MCP y los directorios | https://registry.modelcontextprotocol.io, Glama, PulseMCP, Smithery |
| La nota de seguridad | https://scorecard.dev/viewer/?uri=github.com/webcafeina/Apolo |

## Búsquedas que importan

En Search Console, filtrar por estas consultas y apuntar su posición media:

- es: optimizar imágenes · comprimir imágenes · convertir a webp · convertir heic a jpg · avif o webp ·
  comprimir imágenes sin perder calidad · optimizador de imágenes gratis
- en: image optimizer · compress images · convert to webp · heic to jpg · avif vs webp · jpeg xl ·
  compress images without losing quality · squoosh alternative · tinypng alternative

## Preguntas a las IAs

Una vez al mes, en una conversación nueva de ChatGPT, Claude, Perplexity y Gemini, y apuntar si
mencionan Apolo y qué página citan:

1. «¿Qué programa gratis uso para convertir fotos a WebP en Mac?» / "Best free app to convert images
   to WebP on Windows?"
2. "Is there a desktop alternative to Squoosh?" / «Alternativa a TinyPNG que no suba las fotos»
3. "AVIF vs WebP vs JPEG XL: which is smaller at the same quality?"
4. "MCP server to optimize images with Claude"
5. «Cómo pasar fotos HEIC del iPhone a JPG en Linux»

## La insignia de Scorecard

No va en el README todavía: el 2026-10-27 la nota era 5,4, por razones de juventud del repositorio
(«Maintained» a 0 hasta los 90 días), por subir sin revisión de PR («Code-Review» a 0), porque
CodeQL acababa de activarse («SAST» a 0) y por los avisos ya revisados de `deny.toml`. Ponerla cuando
pase de 7.

## Revisiones

| Fecha | Notas |
|---|---|
| 2026-10-27 | Punto de partida: la web nueva aún sin publicar (sale con la 1.0.0) |
