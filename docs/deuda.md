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
| ~~**El vidrio y el `.dmg` de la entrega 3, sin ver en un Mac**~~ | Media | La barra translúcida, la ventana del `.dmg` y el icono en el Dock solo se habían visto en simulación | **Saldada el 2026-10-06**: el cliente los vio bien en su Mac con la v0.3.0 |
| **El instalador de Windows, sin ver** | Media | (El icono del volumen ya lo vio bien el cliente en su Mac con la v0.3.1.) | El disco con el sol se pone en el `.dmg` después de que Tauri lo haga (`icono-volumen.sh`: pasarlo a escritura, montarlo, cambiar `.VolumeIcon.icns`, recomprimirlo); el NSIS en español con sus imágenes BMP. Ninguno se ha visto: el primero, en un Mac; el segundo, en un Windows | Abierto · 2026-10-06 (v0.3.1) |
| ~~**La ventana del Estudio no se ha visto nunca**~~ | **Alta** | El Estudio se ha probado entero en Chromium contra `apolo-dev`, pero lo propio de la ventana —las órdenes de Tauri, el protocolo `apolo://` con los píxeles, los diálogos de abrir y guardar, arrastrar y soltar— solo se ha compilado en CI. Lo más dudoso: la URL del protocolo en Windows (`http://apolo.localhost/…`) y la CSP | **Saldada en macOS el 2026-10-06** con la prueba guiada sobre la v0.3.1 en el Mac del cliente: abrir arrastrando y con el diálogo, comparador, zoom, lado a lado, calidad en vivo, pegar orden, presets, exportar (**el fichero exportado es idéntico al de cwebp**) y enderezar una foto de iPhone. Queda Windows |
| **La equivalencia con cwebp, sin probar en Windows ni en Linux arm64** | Baja | Ya está comprobada en Linux x86-64 (1015 de 1015) y **en un Mac con Apple Silicon (25 de 25, con PNG, WebP y JPEG, NEON incluido)**, por el cliente el 2026-10-06 con la CLI de la v0.2.0 y el cwebp 1.6.0 de Google para Mac ARM. Faltan Windows y Linux arm64; se puede montar en las máquinas de CI | Abierto, en parte saldado · 2026-10-06 |
| **PNG entrelazado, sin probar** | Baja | El codificador del crate png no escribe Adam7, así que el corpus no tiene ninguno. El lector debería dar lo mismo (libpng y el crate desentrelazan igual), pero no se ha visto | Abierto · 2026-10-06 |
| **PNG de 16 bits con `gAMA`, sin probar** | Baja | libpng corrige la gamma antes o después de bajar a 8 bits según su configuración interna; Apolo la aplica después. Un PNG así podría salir distinto | Abierto · 2026-10-06 |
| **Trozos de texto PNG repetidos de clases distintas** | Baja | pngdec se queda con el primero en orden de fichero; el crate png separa tEXt, zTXt e iTXt y se pierde el orden entre clases. Solo importa con dos EXIF o dos XMP en trozos de clase distinta | Abierto · 2026-10-06 |
| **TIFF raros** | Baja | Orientación distinta de arriba-izquierda (libtiff la endereza al leer en RGBA), paleta, CMYK, YCbCr o más de una página. Apolo cubre 8 y 16 bits en gris, RGB y RGBA | Abierto · 2026-10-06 |
| **PNM con valor máximo distinto de 255** | Baja | cwebp escala con su propio lector; Apolo, con el crate `image`. No se ha comparado | Abierto · 2026-10-06 |

## Técnica

| Elemento | Severidad | Impacto | Estado |
|---|---|---|---|
| ~~**El icono es provisional**~~ | Baja | Un sol trazado en SVG para que haya algo en el paquete | **Saldada el 2026-10-06** en la entrega 3: el cliente eligió el sol con laurel entre cuatro propuestas (ADR 0015) |
| **El icono ocupa todo el lienzo** | Baja | Como el de Esfinge, la placa llena los 1024 px. Los iconos de macOS desde Big Sur dejan un margen (la placa ocupa unos 824 px) y una sombra, así que en el Dock Apolo se verá algo más grande que los del sistema. Se arregla a la vez en los dos, si se quiere | Abierto · 2026-10-06 |
| **HEIC: sin probar con fotos reales** | Media | Solo se ha decodificado la foto de ejemplo de libheif. Faltan fotos de iPhone de verdad (en mosaico, HDR de 10 bits, retrato con profundidad). **Compilar ya compila en los seis sistemas** (v0.3.2, 2026-10-07); que decodifique bien fuera de Linux x86-64 no se ha visto | Abierto · 2026-10-06 (v0.3.2) |
| **Patentes de HEVC** | Baja | Decodificar HEVC (libde265) está cubierto por patentes en algunos países. Para una herramienta gratuita y libre, el riesgo es el de GIMP o ImageMagick, que también lo llevan. Si Apolo se vende, hay que revisarlo (ADR 0017) | Abierto · 2026-10-06 |
| **Windows sin vidrio** | Baja | Esfinge tiene Mica en Windows 11; Apolo, de momento, una ventana opaca (ADR 0016). Hace falta un Windows para probar que en el 10 no queda un agujero | Abierto · 2026-10-06 |
| **libwebp sin SSE4.1 ni AVX2** | Baja | `libwebp-sys` solo enciende SSE2 en x86-64 si no se le piden sus características `sse41`/`avx2`, y estas, encendidas, exigen la instrucción en la CPU de quien instala (no hay detección en tiempo de ejecución). La salida es la misma; la velocidad no se ha medido | Abierto · 2026-10-06 |
| **Fotos grandes en la vista previa** | Media | Cada vista previa manda el resultado entero en RGBA a la interfaz: 96 MB para una foto de 24 MP, y otros tantos el original. Va por memoria local, pero no se ha medido con una foto así. Si va lento, se manda una versión reducida al tamaño de la pantalla y la entera solo al acercarse | Abierto · 2026-10-06 |
| **Recortar en el comparador** | Baja | Con un recorte, el resultado tiene otra proporción y el comparador pasa a lado a lado con una escala aproximada. Lo suyo sería dibujar el recorte sobre el original, y elegirlo con el ratón | Abierto · 2026-10-06 |
| **`-noasm` se ignora** | Baja | Apagar el SIMD de libwebp es cambiar un puntero global, que afecta a toda la aplicación y a los hilos que estén codificando. Se acepta la opción y se avisa | Abierto · 2026-10-06 |

## De producto

*(Nada todavía.)*

## Saldada

*(Nada todavía.)*
