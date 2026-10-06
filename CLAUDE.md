# Apolo

Optimizador de imágenes nativo para macOS, Windows y Linux que sustituye a Squoosh. El motor de WebP es
**libwebp enlazada** —el mismo que hay detrás de `cwebp` y de Squoosh— y Apolo tiene que hacer todo lo que
hace `cwebp`, pero viéndolo. Además: MozJPEG, OxiPNG, AVIF, JPEG XL y QOI; redimensionar, recortar,
reducir paleta y medir la pérdida.

## Protocolo de sesión

La documentación viva está en [docs/](docs/). Se mantiene con disciplina, no con un control
automático: un validador de documentación se acaba sorteando, y lo que hay que sostener es el hábito.
Es el mismo sistema que Esfinge (`~/proyectos/Esfinge/docs/`).

**Al empezar**, leer [docs/estado.md](docs/estado.md). Dice dónde está el proyecto y cuál es la
siguiente acción concreta.

**Al tomar una decisión** que costaría volver a discutir, o que deja el código raro sin explicación,
o que descarta lo que parecía la opción evidente: una ficha en [docs/adr/](docs/adr/) y su línea en
[docs/decisiones.md](docs/decisiones.md). Las secciones son fijas —Contexto, Decisión, Alternativas
descartadas, Consecuencias, Verificación— y la última es la que más se agradece: dice qué se
comprobó de verdad y **qué no**.

**Al encontrar algo a medias o mal**, aunque no se arregle: a [docs/deuda.md](docs/deuda.md), con su
severidad y su impacto.

**Al encontrar una trampa** —algo que costó encontrar y que volvería a costar—: a
[docs/trampas.md](docs/trampas.md), no aquí. Este fichero se queda corto a propósito.

**Al cerrar**, actualizar [docs/estado.md](docs/estado.md) y añadir la entrada en
[docs/sesiones.md](docs/sesiones.md), que tiene su plantilla al final.

Lo que se cierra no se borra: se tacha y se queda, con la fecha.

## Cómo está hecho

Tauri 2 + Rust, con la interfaz en React + Vite + TypeScript ([ADR 0001](docs/adr/0001-tauri-y-rust.md)).

| Carpeta | Qué es |
|---|---|
| `crates/nucleo` | Todo el trabajo con imágenes: decodificar, codificar, procesar, medir. Sin interfaz |
| `crates/tema` | La paleta, el cálculo de contraste y el generador de `frontend/src/tokens.css` |
| `crates/cli` | El binario `apolo`, para scripts. `apolo webp` acepta las opciones de `cwebp` |
| `src-tauri` | La aplicación de ventana: órdenes de Tauri sobre el núcleo |
| `frontend` | La interfaz. Los textos, en `src/i18n/es.json`, nunca escritos a mano en un componente |
| `pruebas/corpus` | Imágenes de prueba para la equivalencia con `cwebp` |

Dos caras sobre el mismo núcleo: lo que hace la ventana lo hace la CLI, con los mismos presets.

## Cómo se compila

- Rust vive en `~/.cargo` (rustup, sin sudo). Node 22 y pnpm, por nvm.
- `make comprobar`: formato, clippy, pruebas, contraste y la interfaz. Es la puerta de CI.
- `make cli`: el binario `apolo`.
- `make tokens`: regenera `frontend/src/tokens.css` desde `crates/tema`. **No se edita a mano.**
- `make app`: la aplicación. En Linux necesita `libwebkit2gtk-4.1-dev`; si la máquina no lo tiene, la
  ventana se compila en CI y aquí solo el núcleo y la CLI.
- Publicar: etiqueta `vX.Y.Z` empujada → `.github/workflows/publicar.yml` saca los instaladores de los
  seis objetivos en la Release.

## Convenciones

- **Todo en español**, con mayúscula inicial en cada frase de la interfaz. Los textos visibles pasan
  por i18n ([ADR 0007](docs/adr/0007-espanol-con-i18n.md)).
- Nombres del dominio en español en el código (`calidad`, `preset`, `lote`); los de las librerías,
  como vengan.
- **Cada opción de `cwebp`** está en [docs/cobertura-cwebp.md](docs/cobertura-cwebp.md): dónde vive en el
  núcleo, en la CLI y en la interfaz, y qué prueba la vigila. Una opción nueva no está hecha hasta que
  su fila está completa.
- Licencia GPLv3 ([ADR 0006](docs/adr/0006-gplv3-y-repositorio-publico.md)): **ninguna dependencia
  nueva sin mirar su licencia**. AGPL no (por eso no `dssim`); propietarias no.
- Commits en español y en prosa.
