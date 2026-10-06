# Estado

Última actualización: **2026-10-06**

## Dónde se paró, y por dónde se sigue

> **Entrega 0 (cimientos) hecha el 2026-10-06**, el mismo día del planteamiento. Hay repositorio
> público (`webcafeina/Apolo`, GPLv3), documentos vivos con las diez ADR del planteamiento, workspace
> de Cargo con libwebp 1.6.0 enlazada, la CLI `apolo`, la carcasa de la interfaz en React con i18n, y
> los tokens generados desde Rust con su prueba de contraste. `make comprobar` está en verde aquí.
>
> **Lo único de la entrega 0 que no se ha visto aquí es la ventana**: el VPS no tiene webkit2gtk y no
> hay sudo. La compila CI. Si `comprobar.yml` y `publicar.yml` (lanzado a mano) salen en verde en los
> seis objetivos, la entrega 0 está cerrada.

### La siguiente acción, al retomar

1. Mirar las ejecuciones de Actions (`gh run list -R webcafeina/Apolo`). Si algo está en rojo, es lo
   primero.
2. Si el cliente puede, que ejecute en el VPS
   `sudo apt install build-essential cmake nasm pkg-config libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev webp`
   para compilar la ventana aquí y tener `cwebp` de referencia. **Ojo:** el `cwebp` de Ubuntu no será el
   1.6.0; para la prueba de equivalencia se descarga el binario oficial de Google 1.6.0.
3. **Empezar la entrega 1** ([siguiente.md](siguiente.md)): decodificar las entradas y la
   correspondencia de `cwebp` con `WebPConfig`, rellenando [cobertura-cwebp.md](cobertura-cwebp.md)
   fila a fila, con la prueba byte a byte contra `cwebp` 1.6.0.

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, salvo verla en CI (2026-10-06).

## En curso

- Entrega 0: comprobar CI.
