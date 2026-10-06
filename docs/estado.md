# Estado

Última actualización: **2026-10-06**

## Dónde se paró, y por dónde se sigue

> **Entrega 0 (cimientos) hecha el 2026-10-06**, el mismo día del planteamiento. Hay repositorio
> público (`webcafeina/Apolo`, GPLv3), documentos vivos con las diez ADR del planteamiento, workspace
> de Cargo con libwebp 1.6.0 enlazada, la CLI `apolo`, la carcasa de la interfaz en React con i18n, y
> los tokens generados desde Rust con su prueba de contraste. `make comprobar` está en verde aquí.
>
> **CI en verde el mismo día**: `comprobar.yml`, con la ventana incluida, y `publicar.yml`, que
> empaqueta los seis objetivos. La primera publicación cayó entera por la categoría del paquete
> ([trampas.md](trampas.md)). **Lo que falta para dar la entrega 0 por vista es abrir un instalador
> en un Mac, un Windows o un Debian.** Los artefactos están en la ejecución 37454237051 de Actions.

### La siguiente acción, al retomar

1. Que el cliente abra uno de los instaladores de la ejecución 37454237051
   (`gh run download 37454237051 -R webcafeina/Apolo`) y diga si arranca y si Ajustes enseña
   «libwebp 1.6.0».
2. Si el cliente puede, que ejecute en el VPS
   `sudo apt install build-essential cmake nasm pkg-config libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev webp`
   para compilar la ventana aquí y tener `cwebp` de referencia. **Ojo:** el `cwebp` de Ubuntu no será el
   1.6.0; para la prueba de equivalencia se descarga el binario oficial de Google 1.6.0.
3. **Empezar la entrega 1** ([siguiente.md](siguiente.md)): decodificar las entradas y la
   correspondencia de `cwebp` con `WebPConfig`, rellenando [cobertura-cwebp.md](cobertura-cwebp.md)
   fila a fila, con la prueba byte a byte contra `cwebp` 1.6.0.

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0 con CI en verde en los seis objetivos (2026-10-06).

## En curso

- Entrega 0: falta abrir un instalador en una máquina de verdad.
