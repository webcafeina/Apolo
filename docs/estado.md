# Estado

Última actualización: **2026-10-06** (noche)

## Dónde se paró, y por dónde se sigue

> **Entregas 0, 1, 2 y 3 hechas el 2026-10-06.** La v0.2.0 está publicada y la **v0.3.0** sale con la
> entrega 3.
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

1. Si la v0.3.0 no está publicada todavía:
   - lanzar `publicar.yml` a mano;
   - etiquetar `v0.3.0`;
   - revisar el borrador (lleva las notas de `.github/notas/v0.3.0.md`);
   - publicarlo (ADR 0014).
2. **Seguir la prueba guiada en el paso 4**, con la v0.3.0 en el Mac del cliente:
   - abrir arrastrando `mia.png` y con «Abrir otra…» (diálogo de macOS);
   - el comparador, la calidad y el zoom;
   - pegar una orden;
   - guardar un preset;
   - exportar;
   - una foto girada.

   Y además, lo nuevo:
   - **el vidrio de la barra lateral**, que no se ha visto nunca;
   - **la ventana del `.dmg`**;
   - **el icono en el Dock**.

   Lo que falle, a la deuda y a arreglar.
3. Con la prueba hecha: **la entrega 4, Lotes** ([siguiente.md](siguiente.md)).

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, cimientos (2026-10-06).
- Entrega 1, núcleo WebP y CLI, con la equivalencia byte a byte (2026-10-06). ADR 0011.
- Entrega 2, el Estudio, con enderezar (2026-10-06). ADR 0012 y 0013.
- v0.2.0 publicada como pre-release (2026-10-06). ADR 0014.
- Entrega 3, identidad (2026-10-06). ADR 0015 y 0016.

## En curso

- La prueba guiada con el cliente, desde el paso 4.
