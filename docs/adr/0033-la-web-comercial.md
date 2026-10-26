# ADR 0033 — La web comercial, sin Squoosh

**Fecha:** 2026-10-26 · **Estado:** aceptada, en obra · antes de la 1.0.0 · matiza la [0026](0026-web-y-homebrew.md) y la [0024](0024-la-1-0.md)

## Contexto

Con Apolo vista en todos los sistemas (v0.18.3), el cliente abre la fase comercial antes de la
1.0.0. Quiere:
- una web más visual y dinámica que detecte el sistema y ofrezca la descarga que toca, con sus
  capturas;
- SEO y medición;
- un README comercial;
- **ninguna referencia pública a Squoosh**.

La web de la 0026 es HTML escrito a mano dos veces, una por idioma. Tiene 4 capturas del aspecto
genérico, SEO mínimo y ninguna medición. `web/descargas.js` pide la versión a la API de GitHub y, si
falla, enseña la 0.12.0.

Planteado el 2026-10-26 en modo plan, con cinco rondas de preguntas, una exploración del repositorio
y un agente de diseño. El plan está en `~/.claude/plans/buenos-d-as-vamos-a-vivid-whale.md`.

## Decisión

**Decidido con el cliente** (2026-10-26):

| Tema | Decisión |
|---|---|
| Objetivo | Descargas de Apolo, con Webcafeína como firma visible y un bloque que lleve a webcafeina.com (con UTM) |
| Tecnología | **Astro estático** (MIT), compilado en `web.yml`. Se construye en `sitio/`, sin desplegar, y sustituye a `web/` al final |
| Estilo | **Página de producto de Apple**, elegido sobre un prototipo con tres estilos y cuatro portadas |
| Portada | **Las dos**: el comparador interactivo arriba (original frente a Apolo, con los pesos y la nota reales) y el vídeo en bucle más abajo |
| Idioma | Botones «ESP / ENG», con el activo en negrita. Español en `/` e inglés en `/en/`, sin redirección automática |
| Detección | Windows x64/ARM, macOS y Linux con selector de distribución (Ubuntu/Debian, Fedora/openSUSE, Arch y otra) y de procesador. Lo elegido se recuerda |
| Adaptación al sistema | Se mantiene la identidad de Apolo. Cambian las capturas, las tarjetas de descarga y los detalles (marco de ventana, forma de los botones), y hay un selector «Ver en» |
| Páginas | Portada, descarga por sistema, preguntas frecuentes, novedades, CLI, agentes y cookies, en los dos idiomas |
| Cookies y medición | Banner propio con Consent Mode v2 **básico** (GTM no se carga hasta que se acepta) y una propiedad de GA4 solo para Apolo. Las páginas legales son las de webcafeina.com; la política de cookies es propia |
| Frase | **«Imágenes más ligeras, igual de buenas» / «Lighter images, just as good»**. En la descripción de Linux, que va en ASCII: «Optimiza fotos: pesan menos y se ven igual» |
| Squoosh | **Fuera de todo lo público**: la web, los README, la app, la CLI, el `.mcpb`, los paquetes, el fondo del `.dmg`, las notas que salen en Novedades y el repositorio de GitHub. Se quedan los comentarios del código, las ADR, la bitácora y las Releases ya publicadas |
| Comparador | Fotos CC0 o de dominio público. La primera, la máscara de oro de Micenas, de Jebulon (CC0, Wikimedia Commons) |
| 1.0.0 | Sale a la vez que la web nueva |
| Fuera de esta fase | Firmar los instaladores, newsletter y dominio propio |

**Hecho hasta ahora:**
- **Squoosh fuera de la app y de los paquetes**: `app.lema`, la ayuda de la paleta, el `about` de
  la CLI, las palabras clave del `.mcpb`, las descripciones de `tauri*.conf.json`, el `.desktop`, el
  metainfo, el PKGBUILD, Homebrew, el SVG y los PNG del fondo del `.dmg`, cuatro notas antiguas y
  `CLAUDE.md`.
- **`frontend/src/publico.test.ts`** vigila que esos ficheros no nombren a Squoosh y que la
  descripción de Linux sea ASCII.
- **`?fondo=transparente`** (puente.ts y main.tsx), con `CAPTURAS_FONDO=transparente` y
  `CAPTURAS_ESCALA=2` en `capturas.spec.ts`: las capturas para la web dejan ver lo que hay detrás del
  vidrio, y la web pone su propio escritorio.
- **El andamiaje de la web** (F3) en `sitio/`: Astro 7.3.8 (MIT; la 7.4 tenía menos de dos semanas),
  sin sharp (`ignoredOptionalDependencies`), con @astrojs/check, vitest, Playwright y
  @axe-core/playwright (MPL-2.0, solo en pruebas); el resto de dependencias, MIT, Apache, BSD, ISC
  o CC0. `crates/tema` genera `sitio/src/estilos/tokens.css` con `css_web()`: las paletas de cada
  sistema en `[data-sistema=…]` sobre cualquier elemento (una tarjeta, un marco), no en `:root`.
  `src/datos/release.ts` lee la Release al compilar y falla si falta alguno de los 19 ficheros que
  enlaza la web; `src/scripts/plataforma.ts` detecta el ordenador; `src/i18n/rutas.ts` empareja las
  páginas. `make web` (check, build y vitest) va dentro de `make comprobar`.
- **`frontend/herramientas/rasterizar.mjs` admite un filtro** (`node rasterizar.mjs fondo-dmg`),
  para rehacer un PNG sin tocar los demás.

## Alternativas descartadas

- **Seguir con HTML a mano.** Con siete páginas, dos idiomas y contenido que cambia en cada versión,
  todo habría que escribirlo dos veces.
- **Construir en una rama larga.** `docs/` cambia en cada sesión, y el entorno `github-pages` solo
  despliega desde `main` y `v*`.
- **Consultar la API de GitHub desde el navegador.** Hay un límite de peticiones, depende de CORS y
  la página se queda en una versión vieja si falla. La versión se lee al compilar.
- **Una plataforma de consentimiento de terceros** (Cookiebot, CookieYes): de pago a partir de
  cierto tráfico, con su propio diseño.
- **Consent Mode avanzado**: la AEPD no lo ve claro.
- **Páginas por formato** (convertir a WebP, a AVIF…): el cliente prefirió las de descarga por
  sistema, preguntas, novedades, CLI y agentes.
- **Fotos de Kodak en el comparador**: a 768 × 512 se ven blandas en grande.
- **Los otros estilos** del prototipo (herramienta moderna y editorial) y las otras portadas (solo el
  comparador, solo el vídeo, la captura animada).

## Consecuencias

- La web deja de «no mandar nada a nadie» (0026). **La aplicación sigue sin mandar nada**; la web,
  con permiso, mide visitas.
- Se pierde la búsqueda «alternativa a Squoosh» y la etiqueta `squoosh` del repositorio.
- Los cambios de la app (frase, `.dmg`, descripciones de los paquetes) llegan con la 1.0.0.
- `make comprobar` gana la compilación y las pruebas de la web (fase 3).

## Verificación

- `make comprobar` en verde con `publico.test.ts` (48 casos).
- El PNG nuevo del fondo del `.dmg`, mirado: la frase nueva, con Marcellus y Figtree.
- Una captura de macOS con vidrio, fondo transparente y a 2x (2560 × 1600, RGBA), compuesta sobre
  un color: la barra lateral deja ver lo de detrás.
- **Sin verificar todavía**: la web nueva entera. Va por fases, F3 a F10 del plan.
