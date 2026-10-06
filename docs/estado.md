# Estado

Última actualización: **2026-10-06** (noche)

## Dónde se paró, y por dónde se sigue

> **Entregas 0, 1 y 2 hechas el 2026-10-06.**
>
> - **La 1** cumple la promesa central: `apolo webp` da **el mismo fichero, byte a byte, que el
>   cwebp 1.6.0 oficial** (1015 de 1015 comparaciones, `make equivalencia`, también en CI).
> - **La 2 es el Estudio**. Tiene:
>   - un comparador original/resultado con deslizador o lado a lado, zoom y desplazamiento;
>   - el panel en tres niveles plegables, con cada opción explicada;
>   - la vista previa en vivo;
>   - la orden cwebp, para copiar y para pegar;
>   - presets con nombre que comparte la CLI;
>   - **enderezar según EXIF** como opción (ADR 0012, lo pidió el cliente);
>   - exportar.
>
>   Por dentro: un servicio con dos transportes, Tauri y HTTP ([ADR 0013](adr/0013-el-estudio-por-dentro.md)).
>   Probado con **14 pruebas de Playwright** en claro y oscuro contra `apolo-dev`.
>
> **Publicada la v0.2.0 como pre-release** (https://github.com/webcafeina/Apolo/releases/tag/v0.2.0),
> con los instaladores, la CLI de los seis objetivos y `SHA256SUMS.txt`. Lo pidió el cliente
> ([ADR 0014](adr/0014-versiones-y-publicacion.md)).
>
> **Lo que más falta ver: la ventana de verdad.** Es la primera entrega con interfaz, y lo propio de
> Tauri no se ha visto funcionar: órdenes, protocolo `apolo://`, diálogos y arrastrar y soltar. Solo
> se compila en CI. Está en [deuda.md](deuda.md) con severidad alta.

### La siguiente acción, al retomar

1. CI quedó **en verde a la primera** al cerrar la entrega 2: `comprobar`, que ya compila
   `src-tauri` con las órdenes y el protocolo, `e2e` y `equivalencia` en la 37494968142, y los seis
   instaladores en la **37494979883**. Mirar `gh run list` por si acaso.
2. **Pedir al cliente que abra el instalador de la v0.2.0** (de la Release) en su Mac
   (y en un Windows si tiene). Que pruebe:
   - abrir una foto;
   - mover la calidad;
   - arrastrar el deslizador y el zoom;
   - exportar;
   - guardar un preset.
3. Con eso visto, la **entrega 3, Lotes** ([siguiente.md](siguiente.md)).

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, cimientos (2026-10-06).
- Entrega 1, núcleo WebP y CLI, con la equivalencia byte a byte (2026-10-06). ADR 0011.
- Entrega 2, el Estudio, con enderezar (2026-10-06). ADR 0012 y 0013.
- **v0.2.0 publicada** como pre-release, con la CLI en la Release (2026-10-06). ADR 0014.

## En curso

- Nada a medias en el código.
