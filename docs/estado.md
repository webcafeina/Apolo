# Estado

Última actualización: **2026-10-06**

## Dónde se paró, y por dónde se sigue

> **Entregas 0 y 1 hechas el 2026-10-06**, el mismo día del planteamiento.
>
> **La 1 cumple la promesa central** ([ADR 0002](adr/0002-libwebp-enlazada.md)): con las mismas
> opciones, `apolo webp` da **el mismo fichero, byte a byte, que el cwebp 1.6.0 oficial de Google**.
> Comprobado en 1015 comparaciones (29 imágenes × 35 combinaciones), con `make equivalencia`, que
> también corre en CI. Para llegar ahí hubo que copiar tres manías de los lectores de cwebp —la
> gamma y los ICC de libpng, el alfa premultiplicado de libtiff— que están en la
> [ADR 0011](adr/0011-leer-como-cwebp.md).
>
> Cada opción de cwebp tiene su fila en [cobertura-cwebp.md](cobertura-cwebp.md); a todas les falta
> solo la columna de la interfaz, que es la entrega 2.
>
> **Lo que no se ha visto**: la equivalencia fuera de Linux x86-64 (en ARM cambia el SIMD) y ningún
> instalador abierto en una máquina de verdad. Las dos cosas, en [deuda.md](deuda.md).

### La siguiente acción, al retomar

1. CI quedó en verde al cerrar (2026-10-06): `comprobar` y `equivalencia` en la ejecución
   37463425487, y los seis objetivos con mozjpeg en la 37463471194. Mirar `gh run list` por si acaso.
2. **Empezar la entrega 2, el Estudio** ([siguiente.md](siguiente.md)): primero el modo de
   desarrollo por HTTP (para probar la interfaz sin ventana, como el puente de Esfinge), luego el
   comparador y los controles, que se pintan a partir de `OpcionesWebp` y de los niveles de
   [cobertura-cwebp.md](cobertura-cwebp.md).
3. Pendiente del cliente: abrir un instalador en su Mac, Windows o Debian.

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, cimientos, con CI en verde en los seis objetivos (2026-10-06).
- Entrega 1, núcleo WebP y CLI, con la equivalencia byte a byte (2026-10-06). ADR 0011.

## En curso

- Nada a medias en el código.
