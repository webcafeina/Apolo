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
> **Lo que más falta ver: la ventana de verdad.** Es la primera entrega con interfaz, y lo propio de
> Tauri no se ha visto funcionar: órdenes, protocolo `apolo://`, diálogos y arrastrar y soltar. Solo
> se compila en CI. Está en [deuda.md](deuda.md) con severidad alta.

### La siguiente acción, al retomar

1. Mirar Actions (`gh run list -R webcafeina/Apolo`). Lo nuevo en CI: el trabajo `e2e` (Playwright)
   y la compilación de `src-tauri` con las órdenes y el protocolo, **que aquí no se puede compilar**.
   Si `comprobar` está en rojo, lo más probable es un error de tipos en `src-tauri/src/lib.rs`.
2. **Pedir al cliente que abra el instalador** de la última ejecución de `publicar.yml` en su Mac
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

## En curso

- Nada a medias en el código.
