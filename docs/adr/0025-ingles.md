# ADR 0025 — Apolo en inglés: cada texto con sus dos versiones en el mismo sitio

**Fecha:** 2026-10-20 · **Estado:** aceptada · entrega 8c · desarrolla la [0024](0024-la-1-0.md),
matiza la [0007](0007-espanol-con-i18n.md)

## Contexto

La 0024 decidió traducirlo todo: la ventana, la CLI, el MCP, los mensajes del núcleo, las notas y el
README. El plan proponía para Rust «un catálogo con claves, candidata `rust-i18n`, o dos tablas
propias con una prueba de claves iguales». El inventario del 2026-10-20 dio más de lo previsto:

- **Rust**, unos 550 textos y 140 líneas de ayuda: el núcleo 130 (errores de lectura y opciones), la
  CLI 120 (con la ayuda de cwebp, cjpeg y oxipng a mano), el MCP 240 (instrucciones, descripciones
  de herramientas y argumentos, errores con pista, respuestas), el servicio, el `.mcpb` y la
  ventana unos 30.
- **La interfaz**, 573 claves en `es.json`, con i18next.
- Los errores de Rust llegan a la ventana tal cual (`Fallo.mensaje`): sin traducirlos en Rust, la
  ventana en inglés enseñaría errores en español.

## Decisión

**En Rust, cada texto con sus dos versiones en el mismo sitio**, con dos macros de
`apolo_nucleo::idioma`:

- `tr!("No se puede leer «{ruta}»", "Can't read “{ruta}”")`: con formato, da un `String`;
- `trs!("Cancelado", "Cancelled")`: fijo, da un `&'static str`.

El compilador exige las dos y que usen los mismos datos (`format!` falla con un dato que sobra), así
que no hay dos catálogos que puedan desincronizarse ni hace falta una prueba de claves iguales.

**El idioma es uno por proceso** (`idioma::poner`). Sin ponerlo, español: las pruebas no dependen del
`LANG` de la máquina (en CI es `C.UTF-8`). Al arrancar:

- la **CLI**: `--idioma es|en`, y si no, el del sistema (`LC_ALL`, `LC_MESSAGES`, `LANG`, y si no,
  el del sistema operativo con `sys-locale`);
- **`apolo-mcp`**: `APOLO_IDIOMA`, y si no, el del sistema;
- la **ventana**: el de sus ajustes («Idioma: el del sistema, español o inglés», en Ajustes ›
  General), que la interfaz le pasa al núcleo, para que los errores de Rust salgan en el mismo
  idioma que la interfaz.

Un sistema en otro idioma (francés, alemán…) usa el inglés.

**El MCP**: las descripciones de las herramientas y de sus argumentos salen de los comentarios `///`
del código (rmcp y schemars), que se quedan en español. En inglés se cambian al servirlas, con una
tabla español → inglés, y una prueba exige que cada descripción tenga la suya. Los nombres de las
herramientas y de los argumentos no se traducen: son identificadores. Las comprobaciones que miraban
el texto de un error (`contains("cambia el tamaño")`) pasan a mirar un código.

**La interfaz**: `en.json` con las mismas claves que `es.json` (lo vigila `i18n.test.ts`).

**El `.mcpb`**: el que crea la ventana, en su idioma; los de la Release, en inglés.

**No se traducen**: los nombres de herramientas y opciones (`-q`, `--salida`), las palabras de clap
(«Usage», «Options», que ya están en inglés), la ayuda oficial de avifenc y cjxl (ya en inglés), y
la documentación interna del repositorio (`docs/`), que sigue en español.

### Glosario

| Español | Inglés |
|---|---|
| Estudio | Studio |
| Lotes / un lote | Batches / a batch |
| Ajustes | Settings |
| preset | preset |
| calidad | quality |
| nota (SSIMULACRA 2) | score |
| nota objetivo | target score |
| sin pérdida | lossless |
| con pérdida | lossy |
| recortar / recorte | crop |
| redimensionar | resize |
| reducir la paleta | reduce the palette |
| enderezar (según el EXIF) | auto-rotate |
| el más ligero | the lightest |
| comparar formatos | compare formats |
| la orden (equivalente) | the command |
| original | original |
| pesa / peso | size / weighs |
| Diferencias (modo) | Differences |
| «Mostrar en la carpeta» | “Show in folder” |
| agente | agent |

Comillas: «» en español, “” en inglés. Mayúscula inicial en cada frase, como en español.

## Alternativas descartadas

- **`rust-i18n`**: MIT, pero arrastra unas 90 dependencias (YAML, TOML, expresiones regulares,
  recorrido de carpetas) para leer ficheros de traducción en tiempo de compilación.
- **Dos tablas con claves**: separan cada texto de su uso, obligan a inventar 550 claves y a una
  prueba para que no se desincronicen. Con los dos textos juntos, quien lee el código ve los dos.
- **Traducir solo la interfaz**: los errores de Rust saldrían en español en la ventana en inglés.

## Consecuencias

- Una dependencia nueva, `sys-locale` (MIT o Apache-2.0), sin dependencias en macOS y Windows.
- Los textos del núcleo dependen de un estado global: una prueba que comprueba un texto en inglés
  usa `idioma::en_este_hilo`, para no cambiar las demás pruebas que corren a la vez.
- Un texto nuevo en Rust se escribe con `tr!` o `trs!` desde el principio.

## Verificación

2026-10-20, con todo hecho:
- `make comprobar` en verde en una máquina con `LANG=C.UTF-8`, como CI: las pruebas de siempre en
  español, y en inglés las nuevas del núcleo, la CLI (`--idioma`, la ayuda, el resumen de un
  lote), el MCP (`tests/ingles.rs`: instrucciones, herramientas sin nada en español, un error con
  «HINTS:») y el servicio (`tests/idioma.rs`, en su propio proceso);
- `i18n.test.ts`: las mismas claves y los mismos datos en `es.json` y `en.json` (578);
- la batería e2e, 65 de 65, con `ingles.spec.ts` (cambiar el idioma en Ajustes, las pantallas y
  dos errores de Rust en inglés, sin claves que falten);
- `make equivalencia`, igual: ningún fichero cambia;
- la evaluación del MCP, 37 de 37, con seis escenarios en inglés.

**No se ha comprobado**: el inglés en la ventana de verdad (macOS, Windows, Linux), solo en el
navegador contra `apolo-dev`; ni que el idioma del sistema se lea bien en macOS y Windows con
`sys-locale`. Lo verá el cliente.
