# Estado

Última actualización: **2026-10-06** (noche)

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

1. **La v0.3.2**: lo que salió de la prueba guiada y **leer HEIC** (ADR 0017), con libheif y libde265
   compiladas dentro como submódulos. Si no está publicada:
   - `publicar.yml` a mano (lo nuevo es compilar libheif y libde265 en las seis máquinas, sobre todo
     en Windows ARM64);
   - etiqueta `v0.3.2`;
   - revisar el borrador y publicarlo.
2. Que el cliente la pruebe en su Mac:
   - **una foto HEIC de su iPhone**, que no se ha probado con ninguna real;
   - el zoom sin rebote;
   - los pesos nuevos;
   - la sección Presets.
3. Después, **la entrega 4, Lotes**.

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
- v0.3.2: arreglos de la prueba y HEIC (2026-10-06). ADR 0017.

## En curso

- Publicar la v0.3.2 (ver arriba).
