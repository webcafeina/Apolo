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
