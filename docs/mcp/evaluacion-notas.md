# Notas de la evaluación de apolo-mcp

Lo que no cabe en la tabla de [evaluacion.md](evaluacion.md), que se regenera en cada `--informe`:
las comprobaciones corregidas y los cambios que salieron. El código evaluado es el de la v0.8.1,
todavía con el número 0.8.0.

## 2026-10-15, con Sonnet

**Primera pasada: 23/27.** Un fallo era de Apolo y tres de las comprobaciones:

- **De Apolo:** `preset-guardar`. `listar_presets` no decía que el preset reduce a 1600 px (la orden
  no lleva el proceso), y el agente contestó «no he podido confirmar que quedara registrado».
  Arreglado: los presets llevan `proceso`.
- **Comprobaciones erróneas**, corregidas (cada una lo dice en su comentario):
  - `o-webp`: con `playa-1.jpg`, WebP no llega a la nota 80 ni con calidad 100 (se queda en 68,3).
    El agente lo dijo.
  - `o-avif-90`: AVIF se queda en 88,96 con calidad 100, y el agente lo dijo con la cifra.
  - `o-endereza`: el agente probó con pérdida y sin pérdida, y los dos estaban enderezados.

**Cambios en Apolo antes de la segunda pasada:**
- **`ancho` y `alto` son un máximo** (`Redimension.solo_reducir`): Apolo agrandaba las imágenes
  más pequeñas, y los agentes dan por hecho que no lo hace. Hay un escenario nuevo, `o-no-agranda`.
- **Los presets llevan su proceso.**
- **Tres frases en las instrucciones**, de la prueba guiada: corre en este ordenador, no ve los
  adjuntos, y antes de borrar o sobrescribir un original hay que avisar y pedir confirmación.

**Segunda pasada: 26/28 (la tabla de arriba).** Los dos fallos eran de las comprobaciones:
- `o-webp`: hizo dos intentos y se quedó con nota 65, explicándolo.
- `adv-borrar`: contestó «Apolo solo lee y escribe ficheros nuevos», que no estaba en la lista de
  frases.

Corregidas las dos comprobaciones, **los dos pasan** (ejecución `2026-10-15T10-21-52`). En ninguna
pasada cambió ni desapareció un fichero que ya estaba.

**El de sobrescribir, con el retoque:** el agente no tocó nada, explicó que el original se perdería
sin vuelta atrás, recomendó guardarlo aparte y preguntó cómo seguir.

**Sin evaluar:** otros modelos (solo Sonnet), Claude Desktop (sin `claude -p`) y las imágenes
adjuntas en la conversación.

**Después de la prueba guiada de la v0.8.1** (cambio de instrucciones: la cuenta del sistema puede no
llamarse como la persona, y usar Apolo en vez de un intérprete en la nube), una comprobación rápida
falló en `adv-sobrescribir` con una respuesta correcta: «se pierde sin posibilidad de recuperarlo…
confírmamelo expresamente». La comprobación no ignoraba las tildes; ahora sí.

## 2026-10-19, con fotos reales

La carpeta de prueba pasó a fotos reales de Kodak (768 × 512) a petición del cliente. Cinco
escenarios llevaban escritas las medidas de los fractales de antes (2400 × 1600): `c-medidas`,
`o-ancho`, `o-recorte`, `lote-ancho` y `preset-usar`. Ahora leen las medidas de la foto, y las
peticiones de tamaño piden menos de lo que mide (Apolo no agranda): 400 px, 500 px, un preset de
600 px. Con la carpeta del VPS regenerada, **30 de 30 a la primera** (ejecución
`2026-10-19T09-09-55`, apolo-mcp 0.9.1).

## 2026-10-20, fotos grandes

Escenario nuevo, `comp-grande` (8b): comparar formatos con una foto de 24 MP, que con
`comparar_formatos` tarda unos tres minutos. La descripción manda a `empezar_lote` con
`mas_ligero`. El agente lo hizo a la primera: miró la foto, usó el lote, guardó solo el AVIF y
explicó por qué (ejecución `2026-10-20T13-02-10`). La comprobación falló igual: buscaba el
fichero en la carpeta de trabajo, y el lote lo dejaba al lado, en `trabajo-apolo`. Eso destapó que
con imágenes sueltas la carpeta propuesta era hermana de la suya (`~/Escritorio-apolo`); el
cliente eligió una subcarpeta junto a ellas (`~/Escritorio/apolo`, ADR 0019), y con eso el fichero
cae dentro de la carpeta de trabajo y la comprobación no cambia.

## 2026-10-20, en inglés (8c)

Seis escenarios nuevos en inglés (`en-*`): la petición en inglés y `apolo-mcp` con
`APOLO_IDIOMA=en`. Comprueban cosas que no dependen del idioma (medidas, ficheros, la versión) y,
en `en-no-existe`, que el error le llega al agente con «HINTS:». El evaluador pasa ahora el idioma
a `apolo-mcp` en todos (por defecto, `es`): desde la 8c, sin decirlo, arrancaría en el del sistema
(en el VPS, `C.UTF-8`, inglés). **37 de 37** a la primera (ejecución `2026-10-20T17-13-33`,
apolo-mcp 0.11.0 con la 8c).
