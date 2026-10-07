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

1. **La entrega 5a está publicada como v0.5.0** (https://github.com/webcafeina/Apolo/releases/tag/v0.5.0) ([ADR 0020](adr/0020-mas-formatos-y-proceso.md)).
   - **JPEG, PNG y QOI** dan el mismo fichero que **cjpeg, oxipng y qoiconv**: 816, 351 y 13 de
     13, con `make equivalencia`.
   - **El proceso**: enderezar, recortar, redimensionar y reducir paleta, en cualquier formato.
   - **El comparador**, con un formato por lado.
   - **Lotes** con varios formatos y «solo el más ligero».
   - **La CLI** gana `apolo jpeg`, `apolo png` y `apolo qoi`.

   Lo decidió el cliente en cuatro preguntas: dos versiones, la misma promesa que con cwebp,
   comparar formatos y el más ligero.
   Antes de publicar se revisó el borrador:
   - CI en verde;
   - 22 ficheros, y las sumas cuadran;
   - `latest.json` con 14 plataformas, todas con firma válida;
   - la CLI de la Release da lo mismo que cjpeg.
2. **Prueba guiada en el Mac del cliente**, que la recibe por el actualizador:
   - un JPEG y un PNG desde el Estudio;
   - comparar WebP contra JPEG;
   - redimensionar;
   - un lote con dos formatos.
3. **La clave del actualizador se queda en el VPS** (`~/.config/apolo/claves/`), por decisión del
   cliente (ADR 0018 y deuda). **No borrarla.** Tampoco se escribe nunca en el chat.
4. La carpeta de prueba de Lotes está en `~/apolo-pruebas-lotes` del VPS. Se regenera con
   `pruebas/lotes/generar.sh <destino>`.
5. Después, **la entrega 5b: AVIF y JPEG XL** (v0.6). Empieza por decidir el motor de AVIF midiendo.

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

## En curso

- Nada a medias en el código.
