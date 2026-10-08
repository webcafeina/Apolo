# Estado

Última actualización: **2026-10-07**

## Dónde se paró, y por dónde se sigue

> **Entregas 0, 1, 2 y 3 hechas el 2026-10-06.** Publicadas la v0.2.0, la v0.3.0 y la **v0.3.1**,
> todas como pre-release. La v0.3.1 añade:
> - la ventana rehecha con las medidas de Esfinge;
> - el icono del volumen (una unidad de aluminio con el sol);
> - el instalador de Windows en español y con imágenes.
>
> Lo pidió el cliente tras ver la v0.3.0 en su Mac, donde ya vio bien la ventana del `.dmg`, el
> vidrio y el icono del Dock.
>
> - **La 1** cumple la promesa central: `apolo webp` da **el mismo fichero, byte a byte, que el
>   cwebp 1.6.0 oficial**. Son 1015 de 1015 en Linux, y **25 de 25 en el Mac del cliente con Apple
>   Silicon**, que lo comprobó él a mano.
> - **La 2** es el Estudio ([ADR 0013](adr/0013-el-estudio-por-dentro.md)).
> - **La 3** es la identidad ([ADR 0015](adr/0015-identidad-nativa-con-marca.md) y
>   [0016](adr/0016-ventana-translucida.md)), que pidió el cliente al probar la v0.2.0:
>   - el **sol con laurel** como icono, elegido entre cuatro propuestas;
>   - el oro del sol como acento;
>   - la barra translúcida en macOS;
>   - el sello, la firma y la bienvenida;
>   - el `.dmg` con fondo propio y sin licencia.
>
> **La prueba guiada con el cliente quedó a medias**, a propósito. Los pasos 1 a 3 los hizo con la
> v0.2.0:
> - la CLI;
> - la equivalencia en su Mac;
> - instalar el `.dmg`.
>
> Al ver que la ventana «apenas tiene diseño», pidió hacer antes la identidad, y se sigue desde el
> **paso 4** con la v0.3.0.

### La siguiente acción, al retomar

1. **La entrega 5b está publicada como v0.6.0** (2026-10-08,
   https://github.com/webcafeina/Apolo/releases/tag/v0.6.0) ([ADR 0021](adr/0021-avif-y-jpeg-xl.md)).
   Lleva también los tres retoques de la v0.5.1. Antes de publicar se revisó el borrador:
   - 22 ficheros, y las sumas cuadran;
   - `latest.json` con 14 plataformas, todas con firma válida (`verificar.py`, minisign con la
     clave pública de `tauri.conf.json`);
   - la CLI de Linux de la Release da los mismos bytes que avifenc y cjxl oficiales: 23 de 23.
   - **AVIF y JPEG XL** dan el mismo fichero que **avifenc 1.4.2 y cjxl 0.12.0**: 376 de 376 y 280
     de 280 con `make equivalencia`, también en CI.
   - Van compilados dentro (`crates/avifjxl`): sus `main`, con los argumentos de la orden. libjxl,
     con clang (en el VPS, `~/.local/llvm18`; el Makefile lo pone).
   - Se abren .avif y .jxl; Estudio, Lotes, pegar órdenes, `apolo avif` y `apolo jxl`.
   - CI: comprobar, e2e y equivalencia en verde. Los instaladores salen en macOS, Windows x64 y los
     dos Linux, y en Windows ARM64 tras apagar el NEON de libyuv: **los seis objetivos compilan**
     (ejecución 37687128943).
   - **v0.6.1 publicada el mismo día** con dos arreglos de la prueba: AVIF «Sin pérdida» con la
     calidad tocada antes daba «avifenc terminó con el código 1» (la orden llevaba lo que el panel
     apaga), y la barra de la descarga se veía llena antes de empezar. Revisada igual: 22
     ficheros, sumas, 14 firmas, y la CLI igual que avifenc y cjxl en 8 de 8.
2. ~~**Prueba guiada en el Mac del cliente**~~ **completa el 2026-10-08 con la v0.6.1**: los seis
   pasos bien, y `apolo avif` igual que el avifenc oficial de Mac en 16 de 16. Los pasos fueron: elegir AVIF y JPEG XL en el Estudio
   y moverse por sus controles; exportar y volver a abrir los dos; comparar AVIF con WebP; un JPEG a
   JPEG XL (recomprimido sin pérdida, y con la calidad al apagarlo); un lote con AVIF y «solo el más
   ligero»; `apolo avif` y `apolo jxl` frente a los oficiales de Mac, si los tiene.
3. **La clave del actualizador se queda en el VPS** (`~/.config/apolo/claves/`), por decisión del
   cliente (ADR 0018 y deuda). **No borrarla.** Tampoco se escribe nunca en el chat.
4. La carpeta de prueba de Lotes se regenera con `pruebas/lotes/generar.sh <destino>`.
5. **La entrega 6, métricas, está publicada como v0.7.0** (2026-10-08) ([ADR 0022](adr/0022-medir-la-perdida.md)).
   Borrador revisado como siempre (22 ficheros, sumas, 14 firmas; la CLI busca y mide, y su nota
   coincide con la de `ssimulacra2`). **Falta la prueba guiada en el Mac del cliente.**
   El cliente eligió SSIMULACRA 2 con PSNR y SSIM, el modo «Diferencias», medir opcional en Lotes y
   **la calidad por nota en esta entrega**.
   - La nota es la de la herramienta `ssimulacra2` de libjxl: 56 de 56 con `make equivalencia`.
   - Al publicar: etiqueta `v0.7.0`, revisar el borrador como siempre, y prueba guiada en el Mac
     (la nota, la calidad por nota, Diferencias, un lote midiendo).

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, cimientos (2026-10-06).
- Entrega 1, núcleo WebP y CLI, con la equivalencia byte a byte (2026-10-06). ADR 0011.
- Entrega 2, el Estudio, con enderezar (2026-10-06). ADR 0012 y 0013.
- v0.2.0 publicada como pre-release (2026-10-06). ADR 0014.
- Entrega 3, identidad (2026-10-06). ADR 0015 y 0016.
- v0.3.0 publicada como pre-release (2026-10-06).
- v0.3.1 publicada: ventana de Esfinge, disco y Windows (2026-10-06).
- Prueba guiada de las entregas 1 y 2 en el Mac del cliente, completa (2026-10-06).
- v0.3.2: arreglos de la prueba y HEIC (2026-10-06). ADR 0017. Publicada el 2026-10-07.
- v0.3.3: el actualizador (2026-10-07). ADR 0018. Publicada el mismo día, ya no como pre-release.
  El cliente la instaló a mano, y «Buscar ahora» funciona.
- Entrega 4, Lotes (2026-10-07). ADR 0019. v0.4.0 publicada el mismo día.
- Prueba de Lotes en el Mac del cliente, completa (2026-10-07). v0.4.1 publicada con sus arreglos.
- Entrega 5a: JPEG, PNG, QOI, el proceso, un formato por lado y Lotes con varios formatos
  (2026-10-07). ADR 0020. v0.5.0 publicada el mismo día.
- Prueba guiada de la v0.5.0 en el Mac del cliente, completa (2026-10-07), con tres retoques.
- Entrega 5b: AVIF y JPEG XL con avifenc y cjxl compilados dentro (2026-10-07). ADR 0021. v0.6.0
  publicada el 2026-10-08, y la v0.6.1 con los arreglos de la prueba.
- Prueba guiada de la v0.6.1 en el Mac del cliente, completa (2026-10-08), con AVIF igual que el
  avifenc oficial de Mac en 16 de 16.
- Entrega 6, métricas (2026-10-08). ADR 0022. v0.7.0 publicada el mismo día.

## En curso

- Nada a medias en el código.
