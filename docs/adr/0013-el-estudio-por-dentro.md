# ADR 0013 — El Estudio por dentro: un servicio, dos transportes y píxeles crudos

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

La entrega 2 es la primera con interfaz de verdad. Hacía falta decidir cómo habla la interfaz con
Rust, cómo llegan los píxeles a la pantalla y cómo se prueba todo en una máquina sin entorno
gráfico (el VPS no tiene webkit ni sudo). Además, el cliente eligió el 2026-10-06, en tres
preguntas: comparar **original contra resultado** (no A contra B como Squoosh), niveles en
**secciones plegables**, y presets como **ficheros JSON** que comparten la ventana y la CLI.

## Decisión

- **Un crate `servicio`** con todo lo que hace el Estudio: imágenes abiertas, vista previa,
  exportar, presets. Sin Tauri. Lo usan **dos transportes**: la ventana (`src-tauri`, órdenes de
  Tauri) y **`apolo-dev`** (`crates/dev`, las mismas órdenes por HTTP). Es el puente de dos caminos
  de Esfinge (su ADR 0009), aquí con Rust en los dos lados. La interfaz habla con `puente.ts` y no
  sabe cuál de los dos tiene detrás.
- **Los píxeles van crudos**: RGBA con ancho y alto delante, por el protocolo `apolo://` en la
  aplicación y por `GET /pixeles/…` en desarrollo. Se pintan en un `<canvas>`. Ni base64 por IPC ni
  `<img>`: un `<img>` pasaría por la gestión de color del navegador, y lo que hay que comparar es lo
  que hay en cada fichero.
- **Vista previa con generaciones**: cada cambio de un control lleva una generación mayor, tras un
  retardo de 120 ms. El servicio cancela, con el aviso de progreso de libwebp, lo que esté
  codificando **esa misma imagen** con una generación anterior, y la interfaz descarta cualquier
  respuesta que no sea la última. La generación va **por imagen**, no global (ver Verificación).
- **El panel se pinta desde un esquema** (`frontend/src/estudio/opciones.ts`): cada control dice
  su campo, tipo, rango, nivel y cuándo tiene sentido. Las etiquetas y explicaciones salen de i18n
  por convención de clave, y una prueba vigila que no falte ninguna.
- **Presets con nombre** en `…/Apolo/presets/<nombre>.json`, en la carpeta de configuración del
  sistema (`dirs::config_dir`), la misma para la ventana y para la CLI
  (`apolo webp -apolo_preset <nombre>`, `apolo presets`).
- Las opciones propias de Apolo llevan el prefijo **`-apolo_`** en la CLI, para no chocar nunca con
  una de cwebp. La interfaz marca cuándo la orden cwebp ya no es equivalente.

## Alternativas descartadas

- **A contra B, como Squoosh.** El doble de controles; el cliente prefirió original contra
  resultado.
- **Mandar los píxeles en base64 por IPC.** Para una foto de 24 MP son ~130 MB de texto por vista
  previa.
- **Un servidor de desarrollo con datos falsos.** Probaría la interfaz contra algo que no es Apolo.

## Consecuencias

- Toda la lógica nueva va al servicio, y los dos transportes son finos. Si una orden aparece en uno
  y no en el otro, falla una de las dos caras.
- Una foto grande viaja entera en RGBA en cada vista previa: 96 MB para 24 MP. Va por memoria
  local y es rápido, pero no se ha medido (deuda).
- El modo «diferencias» del comparador queda para la entrega 5, con las métricas.

## Verificación

2026-10-06:

- **Playwright, 14 pruebas** (7 en claro y 7 en oscuro) contra `apolo-dev` en Chromium: abrir,
  cambiar la calidad, pegar una orden, guardar y usar un preset, enderezar una foto girada y
  exportar. Corren en CI (trabajo `e2e`).
- Capturas mirada a mano en claro y oscuro (`make capturas`).
- **El fallo que encontraron**: con un contador de generación global, al recargar la página todas
  las peticiones parecían viejas y se cancelaban. Pasó a ser por imagen.
- **No verificado**: la ventana de verdad. `src-tauri` (órdenes, protocolo `apolo://`, diálogos y
  arrastrar y soltar) solo se compila en CI y no se ha abierto en ningún sistema. En particular, la
  URL del protocolo en Windows (`http://apolo.localhost/…`) está escrita según la documentación de
  Tauri, sin probar.
