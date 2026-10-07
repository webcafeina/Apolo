# ADR 0019 — Lotes: carpeta de salida con subcarpetas, nunca sobrescribir, y la misma orden en la CLI

**Fecha:** 2026-10-07 · **Estado:** aceptada · desarrolla la [0004](0004-estudio-y-lotes.md)

## Contexto

La entrega 4 es Lotes: muchas imágenes con un mismo preset. La ADR 0004 ya decía qué tenía que
llevar: carpetas o ficheros arrastrados, un preset, en paralelo, cancelable y con un resumen. Faltaba
decidir qué pasa con los ficheros que salen.

Hoy solo hay WebP; los demás formatos de salida llegan con la entrega 5.

## Decisión

El cliente eligió las cuatro respuestas recomendadas:

- **Salida en una carpeta elegida, repitiendo las subcarpetas.**
  - Los originales no se tocan nunca y el resultado queda todo junto.
  - Se propone una carpeta junto a la de entrada, con «-webp» detrás (`vacaciones` →
    `vacaciones-webp`), y se puede cambiar.
  - **Con una sola carpeta**, la estructura es la de dentro de ella (`vacaciones/playa/1.jpg` →
    `vacaciones-webp/playa/1.webp`).
  - **Con varias cosas**, cada carpeta conserva su nombre y los ficheros sueltos van arriba. Así dos
    carpetas con un `1.jpg` no chocan.
  - La carpeta de salida se salta al recorrer: repetir el lote no convierte lo ya convertido.
- **Nunca se sobrescribe nada.** Si el nombre existe, sale `foto-2.webp`, `foto-3.webp`… El nombre
  se reserva al crear el fichero (`create_new`), sin mirar antes si existe. Así no hay hueco entre
  mirar y escribir, y dos hilos que llegan al mismo nombre no se lo quitan. Un fichero que falla a
  medio escribir se borra.
- **Si una imagen pesa más que su original, se guarda igual y se señala.** El resumen cuenta
  cuántas son, y la fila lo marca con «+NN %» en el color de aviso.
- **`apolo lote` en la CLI, en esta misma entrega**, con los mismos presets:
  - `apolo lote fotos/ --salida fotos-webp --preset "Fotos web" -- -q 80`;
  - las opciones de cwebp van detrás de `--`, encima del preset (como `-apolo_preset` en
    `apolo webp`);
  - el progreso va a la salida de errores y el resumen a la normal;
  - sale con error si falló alguna imagen.

Y lo que decidí yo, dentro de lo que ya decía la 0004:

- **Cada imagen se codifica exactamente como `apolo webp`** (`cwebp::ejecutar`). La equivalencia
  byte a byte con cwebp se conserva imagen a imagen, y hay una prueba que lo comprueba.
- **En paralelo, un hilo por núcleo, como mucho ocho.** Cada hilo tiene una imagen entera en
  memoria; ocho fotos de 24 megapíxeles son unos 800 MB.
- **Cancelar** aborta la imagen a medias (libwebp pregunta en cada paso) y las pendientes ya no
  empiezan. Lo ya escrito se queda.
- **Se recogen** las extensiones que Apolo sabe leer. Se saltan los ocultos (los `._foto.jpg` que
  deja macOS en los discos externos) y las carpetas enlazadas (para no dar vueltas). Un fichero
  suelto se intenta aunque la extensión no cuadre; si no es una imagen, falla él solo.
- **La interfaz pregunta cómo va** cada 250 ms y recibe solo lo nuevo (`estado_lote(id, desde)`).
  Funciona igual por Tauri que por HTTP, sin eventos que solo tenga uno de los dos (ADR 0013).
- **La sección no se desmonta** al cambiar de sección, como el Estudio: un lote largo sigue contando
  mientras se mira otra cosa. Lo que se suelta en la ventana va a la sección que se ve.
- **El resumen** dice:
  - cuántas se convirtieron y en cuánto tiempo;
  - lo que pesaban y lo que pesan;
  - cuántas crecieron y cuántas fallaron;
  - las cinco que menos ahorran (solo si hay más de cinco; si no, ya están todas a la vista);
  - «Mostrar en la carpeta», con `tauri-plugin-opener`.
- **El navegador de desarrollo no sabe las rutas** de lo que se suelta. Ahí sale un campo para
  escribirlas, que la ventana no enseña.

## Alternativas descartadas

- **Junto a cada original, o en una subcarpeta junto a cada original.** Mezclan resultados y
  originales, y con muchas carpetas se reparten por todas partes.
- **Sobrescribir, o saltarse lo que ya existe.** Sobrescribir pierde trabajo sin avisar. Saltarlo
  hace que repetir un lote tras cambiar el preset no haga nada, también sin avisar.
- **No guardar las que crecen.** Deja huecos si se necesita todo en WebP.
- **Eventos de Tauri para el progreso.** El servidor de desarrollo tendría que imitarlos con
  WebSockets o SSE. Preguntar cada cuarto de segundo cuesta poco y es igual en los dos.
- **`rayon` o `walkdir`.** Con `std::thread::scope` y `read_dir` sobra, y son dependencias menos.

## Consecuencias

- Un lote repetido a la misma carpeta crea `-2`, `-3`… Para empezar de cero, se borra la carpeta de
  salida o se elige otra.
- La carpeta de salida puede quedar dentro de la de entrada: se excluye al recorrer, pero solo esa.
  Una salida de otro lote anterior, con otro nombre, sí se recogería.
- Al cancelar en la CLI (Ctrl+C) el proceso muere sin más. Lo ya escrito queda bien, y como mucho
  queda un fichero a medias, el que se estaba escribiendo en ese instante.
- `Error::Fichero` en el núcleo, para los fallos de leer o escribir con la ruta ya en el texto.

## Verificación

2026-10-07, en el VPS:

- **Núcleo** (`crates/nucleo/src/lote.rs`, 7 pruebas):
  - recoger con una y con varias entradas, sin ocultos y sin la salida;
  - no pisar (`foto-2`, `foto-3`);
  - **el lote da los mismos bytes que `apolo webp`** para cada imagen;
  - una imagen rota falla sola;
  - el resumen ordena las peores;
  - repetirlo da `-2`;
  - cancelar antes de empezar no hace nada, y cancelar a medias deja de empezar.
- **Servicio**: un lote entero, preguntando por el estado hasta que termina.
- **CLI** (`crates/cli/tests/lote.rs`, 3 pruebas):
  - subcarpetas, no pisar y la salida propuesta;
  - una imagen rota da error y las demás salen;
  - un preset que no existe no empieza.
- **A mano**: la CLI contra el cwebp 1.6.0 oficial, con `-q 70 -m 6 -metadata all`, da **los mismos
  bytes** en las dos imágenes (una de ellas en una subcarpeta).
- **e2e** (`frontend/e2e/lotes.spec.ts`):
  - una carpeta con el preset Foto: la orden enseñada, el resumen, los ficheros en disco;
  - repetirlo da `foto-2.webp`;
  - una carpeta vacía no deja convertir y se puede quitar.
- **Capturas** de los momentos vacío, preparado y resumen, en claro y oscuro.
- Licencias de `tauri-plugin-opener` y lo que trae: MIT o Apache-2.0.

**No verificado:**

- en la ventana: los diálogos de elegir carpetas, soltar sobre Lotes (y que el Estudio no lo
  abra), y «Mostrar en la carpeta». Aquí no hay webkit; lo verá el cliente;
- lotes grandes (miles de imágenes, fotos de 24 megapíxeles): memoria y tiempo, sin medir;
- cancelar desde la interfaz (sí en Rust).
