# Deuda y cabos sueltos

Última actualización: **2026-10-06**

Lo que sabemos que está a medias, mal o sin comprobar. Los bloqueantes primero. Lo saldado se tacha
y se queda.

## Sin comprobar

Lo más caro de esta lista no es lo que está mal, es lo que no sabemos si lo está.

| Elemento | Severidad | Impacto | Estado |
|---|---|---|---|
| **La ventana no se compila en el VPS** | Media | Falta `libwebkit2gtk-4.1-dev` (y `pkg-config`), que exige sudo. Mientras tanto, `src-tauri` solo se compila en CI y aquí se prueban el núcleo, la CLI y la interfaz en el navegador | Abierto · 2026-10-06. Lo arregla `sudo apt install build-essential cmake nasm pkg-config libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev webp` |
| **Ninguna compilación se ha abierto en un Mac, un Windows ni un Debian** | Media | Los instaladores salen de CI; que arranquen solo se sabe abriéndolos | Abierto · 2026-10-06 |
| **La equivalencia con cwebp solo se ha comprobado en Linux x86-64** | Media | En ARM, libwebp usa NEON en vez de SSE2, y mozjpeg también cambia de código. Si algún camino SIMD no es exacto, en un Mac M1 el fichero podría no ser el de cwebp. Google publica cwebp para macOS, Windows y Linux arm64: se puede montar la prueba en las máquinas de CI de cada uno | Abierto · 2026-10-06 |
| **PNG entrelazado, sin probar** | Baja | El codificador del crate png no escribe Adam7, así que el corpus no tiene ninguno. El lector debería dar lo mismo (libpng y el crate desentrelazan igual), pero no se ha visto | Abierto · 2026-10-06 |
| **PNG de 16 bits con `gAMA`, sin probar** | Baja | libpng corrige la gamma antes o después de bajar a 8 bits según su configuración interna; Apolo la aplica después. Un PNG así podría salir distinto | Abierto · 2026-10-06 |
| **Trozos de texto PNG repetidos de clases distintas** | Baja | pngdec se queda con el primero en orden de fichero; el crate png separa tEXt, zTXt e iTXt y se pierde el orden entre clases. Solo importa con dos EXIF o dos XMP en trozos de clase distinta | Abierto · 2026-10-06 |
| **TIFF raros** | Baja | Orientación distinta de arriba-izquierda (libtiff la endereza al leer en RGBA), paleta, CMYK, YCbCr o más de una página. Apolo cubre 8 y 16 bits en gris, RGB y RGBA | Abierto · 2026-10-06 |
| **PNM con valor máximo distinto de 255** | Baja | cwebp escala con su propio lector; Apolo, con el crate `image`. No se ha comparado | Abierto · 2026-10-06 |

## Técnica

| Elemento | Severidad | Impacto | Estado |
|---|---|---|---|
| **El icono es provisional** | Baja | Un sol trazado en SVG para que haya algo en el paquete. El definitivo es parte de la entrega 6 | Abierto · 2026-10-06 |
| **libwebp sin SSE4.1 ni AVX2** | Baja | `libwebp-sys` solo enciende SSE2 en x86-64 si no se le piden sus características `sse41`/`avx2`, y estas, encendidas, exigen la instrucción en la CPU de quien instala (no hay detección en tiempo de ejecución). La salida es la misma; la velocidad no se ha medido | Abierto · 2026-10-06 |
| **`-noasm` se ignora** | Baja | Apagar el SIMD de libwebp es cambiar un puntero global, que afecta a toda la aplicación y a los hilos que estén codificando. Se acepta la opción y se avisa | Abierto · 2026-10-06 |

## De producto

*(Nada todavía.)*

## Saldada

*(Nada todavía.)*
