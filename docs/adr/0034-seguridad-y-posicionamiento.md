# ADR 0034 — Seguridad y posicionamiento de la web y del repositorio

**Fecha:** 2026-10-27 · **Estado:** aceptada, en obra · antes de la 1.0.0 · matiza la [0033](0033-la-web-comercial.md)

## Contexto

Con la web nueva terminada en `sitio/` y Analytics, Tag Manager y Search Console configurados, el
cliente para antes del cambio de web y de la 1.0.0 para plantear dos cosas: la seguridad de la web y
del repositorio, y una estrategia para que la web y el repositorio se encuentren en buscadores y en
IAs (ChatGPT, Claude, Perplexity, AI Overviews).

La auditoría del 2026-10-27 encontró:

- **La web.** GitHub Pages la sirve directamente, sin ninguna cabecera de seguridad, y no deja
  ponerlas. La web nueva no tiene CSP. Las notas de Novedades salen de `marked` sin sanear. Las
  descargas solo enseñan algunas sumas.
- **El repositorio.** `main` sin proteger. Una sola acción fijada por commit. Permisos de los
  workflows sin declarar. Sin Dependabot, CodeQL, Scorecard ni `SECURITY.md`. Y lo más serio:
  tauri-action compilaba el frontend (código npm de terceros) con las claves de firma en el
  entorno.
- **El SEO.** Lo básico está bien hecho, pero faltan:
  - páginas que respondan a lo que se busca;
  - datos estructurados de la organización y del código;
  - `llms.txt`;
  - presencia en directorios.

## Decisión

Decidido con el cliente: **todo antes de la 1.0.0**, en cinco partes (S1 a S5).

- **S1, el repositorio y CI:**
  - acciones fijadas por commit, con la versión en un comentario;
  - permisos mínimos por trabajo;
  - `persist-credentials: false`;
  - zizmor (MIT) y `cargo deny check advisories` (MIT/Apache-2.0) en `make seguridad`, dentro de
    `make comprobar`;
  - Dependabot semanal, con siete días de espera;
  - Scorecard de OpenSSF (Apache-2.0);
  - procedencia firmada de cada fichero de la Release (`actions/attest-build-provenance`);
  - `SECURITY.md`.
  - En `publicar.yml`, la interfaz se compila en un paso propio, sin secretos, y
    `tauri.publicar.conf.json` quita el `beforeBuildCommand`. Con su sí, en GitHub: rulesets en
    `main` y en `v*`, permisos de solo lectura por defecto, aviso privado de vulnerabilidades,
    Dependabot y CodeQL.
- **S2, la web:**
  - **Cloudflare delante de Pages** (nube naranja en `apolo`) con las cabeceras de seguridad, sin
    bloquear a quien no es un navegador (`apt`, `dnf`, rastreadores de IA) y sin caché en `/apt`,
    `/rpm` y `/aur`;
  - la CSP del contenido en el HTML, con hashes generados al compilar;
  - las notas saneadas;
  - todas las sumas, la huella GPG y la procedencia en las páginas de descarga;
  - `security.txt`.
- **S3, SEO técnico y para IAs:**
  - `Organization`, `WebSite`, `BreadcrumbList`, `SoftwareSourceCode` y `TechArticle`;
  - `lastmod` en el sitemap;
  - `llms.txt` y `llms-full.txt`;
  - rastreadores de IA permitidos de forma explícita;
  - IndexNow y Bing (ChatGPT busca sobre su índice);
  - títulos con las búsquedas reales.
- **S4, contenido:**
  - **seis guías** (matiza el «ninguna página por formato» de la 0033);
  - **comparativas con Squoosh, TinyPNG, ImageOptim y Caesium**. Matiza el «Squoosh fuera de todo
    lo público»: se nombra **solo** en las comparativas, y `publico.test.ts` lo vigila;
  - preguntas, CLI y agentes, ampliados.
- **S5, GitHub y directorios:**
  - el README con lo que se busca;
  - Discussions;
  - el registro oficial de MCP desde CI;
  - la homepage del `.mcpb` en la web;
  - envíos a directorios, cada uno con su sí;
  - medición de las visitas que llegan desde IAs.

## Alternativas descartadas

- **Solo una CSP en `<meta>`**: sin HSTS, sin `frame-ancestors` y sin el resto de cabeceras. Es
  protección a medias.
- **Mover la web a Cloudflare Pages**: cabeceras con `_headers`, pero es otro despliegue, y los
  repositorios APT y RPM también se mudarían.
- **Solo mejorar las páginas que hay**: menos trabajo, pero sin páginas que respondan a lo que se
  busca no hay de qué posicionar.
- **Exigir PR en `main`**: el proyecto lo lleva una sola persona y se sube directo. El ruleset solo
  impide forzar y borrar.
- **Fijar las imágenes de contenedor del humo** (`fedora:latest`, `archlinux:latest`): el humo
  prueba a propósito en la distribución de hoy. No llevan secretos ni escriben nada.

## Consecuencias

- **Dependabot abre un PR por ecosistema cada semana.** Las acciones fijadas por commit las pone al
  día él.
- **Las cachés de Publicar y de la web** solo las escribe `main`, que es código propio. Lo dice
  `.github/zizmor.yml`, que ignora ahí `cache-poisoning`.
- **Squoosh vuelve a lo público**, solo en las comparativas.
- **Cloudflare, en medio**:
  - lo que bloquea bots se queda apagado en `apolo`;
  - la renovación del certificado de Pages detrás del proxy hay que vigilarla.

## Verificación

- **S1:**
  - zizmor sin avisos: 6 ignorados con su motivo y 46 suprimidos por su baja confianza (2026-10-27);
  - `cargo deny check advisories` en verde tras subir rustls, rustls-webpki, webpki-root-certs y
    tokio; las excepciones, con su motivo en `deny.toml`;
  - Tauri ya no ejecuta `beforeBuildCommand` con `tauri.publicar.conf.json` (comprobado con
    `pnpm tauri build --no-bundle`);
  - los ajustes de GitHub, leídos con `gh api` tras ponerlos el cliente;
  - **sin verificar**: la procedencia de la Release (sale con la 1.0.0).
- **S2:** cabeceras, repositorios (`apt` incluido), GPTBot y ClaudeBot comprobados con `curl` a
  través de Cloudflare; la CSP, con una prueba de Playwright que simula GTM y GA4 y ve bloqueado un
  script ajeno; las notas, con una prueba de vitest. **Sin verificar**: la renovación del
  certificado de GitHub con el proxy (diciembre de 2026).
- **S3:** el grafo JSON-LD, comprobado en las 43 páginas por `paginas.spec.ts`; `llms.txt`, por
  `llms.test.ts`; el JSON de IndexNow, generado a mano con el sitemap. **Sin verificar**: la prueba
  de resultados enriquecidos de Google y el aviso real a IndexNow (cuando la web esté en el aire).
- **S4:** las cifras salen de `medidas.json` (Apolo 0.18.3, 2026-10-27); lo de cada herramienta
  comparada, comprobado en su web y su repositorio ese día. Vistas por el cliente en una galería.
- **S5:** `server.json` validado contra el esquema y con `mcp-publisher validate` (v1.8.1) usando
  los `.mcpb` de la 0.18.3; `mcp.yml` pasa zizmor. Discussions activado por el cliente, con su hilo
  de bienvenida. **Sin verificar**: la publicación real en el registro (sale con la 1.0.0) y los
  envíos a directorios, que esperan al lanzamiento (`docs/lanzamiento.md`).

## Matices de S5 (2026-10-27)

- **awesome-tauri ya no recoge aplicaciones**, solo plugins y plantillas: se descarta. Queda
  awesome-mcp-servers, en «Multimedia Process».
- **GA4 ya trae el canal «AI Assistant»** en su grupo predeterminado: no hace falta un grupo propio.
- **La insignia de Scorecard no va en el README todavía**: la nota era 5,4 por la juventud del
  repositorio, por subir sin revisión de PR y por CodeQL recién activado. Va cuando pase de 7
  (`docs/seo.md`).
- **winget, con komac** (GPL-3.0, funciona en Linux) en vez de wingetcreate, que pide Windows.

## El cambio de web (F10, 2026-10-27)

- `sitio/` pasa a `web/` y la web estática de antes se borra. `web.yml` es el que era el borrador,
  con un cambio decidido con el cliente: **solo despliega al publicar una versión o a mano**, para
  que la web nueva salga a la vez que la 1.0.0. **Tras el lanzamiento hay que volver a añadirle el
  `push` a `main`** con `paths: web/**, .github/notas/**, .github/workflows/web.yml`, o los cambios de
  la web no se publicarían hasta la siguiente versión.
- Los objetivos del Makefile pasan a llamarse `capturas-web` y `video-web`; el `capturas-web` de la
  web vieja desaparece.
