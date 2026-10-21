# ADR 0026 — La web y Homebrew

**Fecha:** 2026-10-21 · **Estado:** aceptada · entrega 8d · desarrolla la [0024](0024-la-1-0.md)

## Contexto

La 0024 dejó para la 1.0 una web en `apolo.webcafeina.com` (GitHub Pages, español e inglés) y
Homebrew para la CLI y la aplicación, en un tap propio. Apolo no está firmada (ADR 0009), y Homebrew
ha cambiado mucho en el último año con las aplicaciones sin firmar.

## Decisión

### La web

- **Estática, en `web/`**, sin marco ni paso de compilación: `index.html` (español),
  `en/index.html` (inglés), `estilos.css` y `descargas.js`. Se publica con GitHub Pages desde
  `.github/workflows/web.yml` (Actions como origen), con el dominio puesto en los ajustes del
  repositorio, no con un fichero `CNAME` (con Actions no se usa).
- **La paleta, la de la aplicación**: `make tokens` escribe `web/tokens.css` desde `crates/tema`,
  igual que el de la ventana, y `tests/contraste.rs` vigila los dos. La web solo usa parejas medidas
  ahí; el oro nunca es color de texto.
- **La estructura, de «raycast»** (`~/sistemas-diseno-empresas`), solo la estructura: una web de
  producto donde la interfaz de la aplicación es la protagonista. Escala tipográfica 64/56/40/20/18/16,
  espaciado 4–32 y 96 entre secciones, radios 6–16, borde de un píxel, 1200 px.
- **Las tipografías de la marca, alojadas en la web**: Marcellus y Figtree (OFL, de fontsource). Sin
  Google Fonts: Google vería la IP de cada visitante, y Apolo presume de no mandar nada a nadie.
- **Las descargas**, por la API de GitHub (`releases/latest`): el botón principal lleva el fichero
  del sistema de quien visita, y la tabla, los de todos. Sin red, todo enlaza a la página de la
  última versión.
- **Las capturas**, con `make capturas-web`: las de siempre, con una foto de Kodak en vez de la del
  corpus (de 128 × 128, que ampliada se veía pixelada), en los dos idiomas, y pasadas a WebP con el
  propio Apolo: casi sin pérdida las que llevan foto (SSIMULACRA 2 de 93; con pérdida, el texto de la
  interfaz no deja pasar de 84 ni a calidad 100) y sin pérdida las que son solo interfaz.

### Homebrew

Investigado el 2026-10-21 (Homebrew 7.0.10):

- **El corte de septiembre de 2026** (casks que no pasan Gatekeeper, desactivados) **es solo del
  repositorio oficial** `homebrew/cask`. En un tap propio, `brew` instala un cask sin firmar sin
  quejarse (`cask/audit.rb` solo lo mira en los oficiales); macOS lo bloquea al abrirlo.
- **`--no-quarantine` ya no existe** (desactivado en 5.1, quitado en 6.0). El cask le quita la
  cuarentena al instalar con `postflight_steps` y un paso `run` de `/usr/bin/xattr` (el `postflight
  do … end` de antes está obsoleto desde Homebrew 7.0 y deja de funcionar el 2027-12-11). Las
  actualizaciones no conservan el permiso de macOS de una versión a otra con una firma ad hoc, así
  que quitar la cuarentena en cada instalación es lo que evita el aviso cada vez.
- **Una fórmula no se pone en cuarentena**: la CLI y `apolo-mcp` van con una fórmula de binarios ya
  compilados (`on_macos`, `on_linux` con `on_intel` y `on_arm`), sin bottles.
- **Desde Homebrew 6, un tap ajeno hay que confiarlo**: instalar con el nombre completo
  (`brew install webcafeina/apolo/apolo`) confía en ese paquete. Así se escribe en todas partes.
- `depends_on macos: :big_sur` (el `">= :big_sur"` de antes está obsoleto).
- **El tap se actualiza desde `publicar.yml`**: un trabajo al final, tras subir los ficheros, que
  rellena la fórmula y el cask con la versión y las sumas y los empuja a
  `webcafeina/homebrew-apolo` con una deploy key de escritura solo en ese repositorio, guardada como
  secreto. Las acciones existentes (`bump-homebrew-formula-action`, `brew bump-*`) no manejan una
  descarga por sistema.

## Alternativas descartadas

- **Un generador de sitios (Astro, Vite)**: dos páginas no lo necesitan, y sería otra cadena de
  dependencias que mantener.
- **Google Fonts**: lo de la IP de arriba.
- **El cask sin quitar la cuarentena**, con un aviso: cada actualización volvería a bloquearla.
- **Un PAT para el tap**: caduca y va unido a una persona; la deploy key, no.

## Consecuencias

- El repositorio del tap, su deploy key y activar Pages con el dominio son publicaciones hacia
  fuera: se hacen con el sí del cliente en ese momento. El registro DNS lo pone el cliente.
- Si Homebrew endurece también los taps ajenos, el cask es lo primero que se cae; la fórmula de la
  CLI, no.

## Verificación

2026-10-21, la web:
- vista en Chromium a 1280 y 390 px, claro y oscuro, español e inglés: sin desplazamiento lateral, y
  el botón detecta el sistema y enlaza al fichero real de la última versión;
- el contraste, por `tests/contraste.rs` con `web/tokens.css`.

**No se ha comprobado**: Safari ni Firefox; un móvil de verdad; la web publicada; Homebrew entero
(ni el tap ni un `brew install`).
