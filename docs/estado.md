# Estado

Última actualización: **2026-10-14**

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

0. **Entrega 7, agentes por MCP** ([ADR 0023](adr/0023-mcp.md)), decidida con el cliente el
   2026-10-14 en modo plan. Pasa delante de la 1.0, que queda como entrega 8.
   - **La 7a está hecha** (sin versión publicada): `crates/mcp` con el binario `apolo-mcp` y las
     diez herramientas, sus pruebas (las herramientas, el protocolo por tuberías con AVIF y JXL a la
     vez, el fd 1), el catálogo generado ([mcp/catalogo.md](mcp/catalogo.md)) y el paseo con Claude
     Code ([mcp/paseo.md](mcp/paseo.md)).
   - **La 7b está hecha en el código** (sin empujar): `apolo-mcp` dentro de cada instalador, el
     `.mcpb` con el relevo a la Apolo instalada, la herramienta `acerca_de_apolo` y la versión en
     las instrucciones, la fila «MCP» encima de Ajustes con la pestaña «Agentes de IA», CI, README y
     las notas de la v0.8.0. Lo pidió el cliente antes de empezarla: que se actualice sola, que diga
     su versión y la fila en la barra.
   - **Al parar (2026-10-14)**: empujado. `Comprobar` en verde (la ventana compila, e2e y
     equivalencia). Publicación de prueba **38558247128** (sin etiqueta): Linux y Windows bien. El
     `.deb` lleva `/usr/bin/apolo-mcp`, y el de la CI responde a `acerca_de_apolo`. El instalador de
     Windows x64 pasa de 9 a 16 MiB, que es `apolo-mcp.exe`. Hay `.mcpb` de Linux y de Windows (11
     herramientas), y la CLI trae `apolo-mcp`. ~~El Mac seguía compilando al parar~~: **acabó bien**
     (2026-10-15). `Apolo.app/Contents/MacOS/apolo-mcp` está junto a `apolo-app`, universal (x86_64 y
     arm64), y el `.mcpb` de macOS lleva `server/apolo-mcp` con el bit de ejecución. Los seis objetivos
     y las cinco CLI, en verde.
   - **Lo siguiente, mañana**:
     1. ~~Mirar cómo acabó el Mac de la publicación de prueba~~: bien (2026-10-15).
     2. ~~La v0.8.0~~: **publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.0),
        con el sí del cliente. Borrador revisado: 27 ficheros (los 22 de siempre y 5 `.mcpb`), 26
        sumas, 7 `.sig` y las 14 plataformas de `latest.json` con firma válida; la CLI y `apolo-mcp`
        de Linux dicen 0.8.0, `apolo-mcp` optimiza por el protocolo y sus instrucciones llevan la
        versión; los cinco `.mcpb` en 0.8.0 con su servidor ejecutable y once herramientas; la CLI da
        el mismo AVIF que avifenc 1.4.2 en 3 de 3.
     3. ~~La prueba guiada en su Mac~~: **bien el 2026-10-15** (pasos 1 a 5; Claude Code, aparte).
        Actualizó a la 0.8.0; la extensión, en Claude Desktop, cede el turno a la aplicación y el
        agente dice «Apolo 0.8.0»; optimizó un PNG con transparencia (AVIF −94 % a nota 80,1)
        mirando el mapa y zonas, sin tocar el original. Dos retoques para la 7c (siguiente.md).
     4. ~~Probar Claude Code con el cliente~~: **bien el 2026-10-15** (Sonnet 5.5, esfuerzo medio):
        rutas relativas, comparar, redimensionar, dos lotes. En la adversaria, Apolo no sobrescribió y
        Claude Code borró el original con su shell **tras pedir permiso** (el cliente lo aceptó): retoque de instrucciones para la 7c.
     5. ~~La 7c~~: **hecha el 2026-10-15**. `scripts/evaluar-mcp.mjs` con 28 escenarios
        (`make evaluar-mcp`): 26/28 con Sonnet en la pasada completa, y los dos que quedaban eran de
        las comprobaciones (corregidas, y pasan). Arreglos en Apolo: `ancho` y `alto` son un máximo
        que no agranda (`Redimension.solo_reducir`); los presets dicen su proceso; los tres retoques
        de las instrucciones.
     6. ~~La v0.8.1~~: **publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.1),
        con el sí del cliente y, a petición suya, el aviso de reinicio (Claude Desktop mantiene el
        servidor viejo hasta reiniciarse). Borrador revisado: 27 ficheros, 26 sumas, 7 `.sig` y 14
        plataformas en `latest.json` con firma válida; CLI y `apolo-mcp` en 0.8.1; cinco `.mcpb` en
        0.8.1 con su servidor ejecutable; el mismo AVIF que avifenc en 3 de 3.
     7. Prueba guiada de la v0.8.1: bien en lo nuevo (no agranda, presets, Claude Code avisa antes de
        sobrescribir); dos retoques de instrucciones y el aviso destacado, en `main` (siguiente.md).
        **Por repetir**: la imagen adjunta en Claude Desktop, en conversación nueva.
     8. **v0.8.2 publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.2)
        con lo que estaba en `main`: el aviso destacado de reiniciar Claude Desktop y los dos retoques
        de instrucciones (la cuenta del sistema; usar Apolo en vez del intérprete en la nube).
        Borrador revisado: 27 ficheros, sumas, 7 `.sig`, `latest.json` con 14 plataformas, CLI y
        `apolo-mcp` en 0.8.2 con las instrucciones nuevas, cinco `.mcpb`, AVIF igual que avifenc 3/3.

### Al parar el 2026-10-15: lo primero mañana

1. **Que el cliente actualice a la 0.8.2 con Claude Desktop abierto**: es la primera vez que se
   puede ver el aviso de «Apolo se ha actualizado» (lo lleva la 0.8.1 que tiene en marcha). Luego
   reiniciar Claude Desktop.
2. **Repetir el paso 4 en Claude Desktop**: conversación nueva, arrastrar una imagen y «Optimízala
   para web». Debe pedir la ruta o buscarla y usar Apolo, no su intérprete en la nube.
3. Después, lo pendiente: recortar con el ratón en el Estudio; recortar y paleta desde el agente;
   el CI a `macos-15` antes del 2 de noviembre (apagones de `macos-14` el 16–17, 19–20, 23–24,
   26–27, 29–30 y 30–31 de octubre, de 16:00 a 02:00 en Madrid: no empujar etiquetas entonces).

     9. Pendiente de antes: en
        la siguiente actualización, ver el aviso en Claude Desktop (deuda). Después: recortar con
        el ratón en el Estudio, recortar y paleta desde el agente, y el CI a `macos-15` antes del 2
        de noviembre. Las notas están en
        `.github/notas/v0.8.1.md`. Después: recortar con el ratón en el Estudio, y recortar y paleta
        desde el agente; pasar el CI a `macos-15` antes del 2 de noviembre (deuda).
   - Probarlo a mano: `make mcp`, y en Claude Code
     `claude mcp add --scope user apolo -- ~/proyectos/apolo/target/release/apolo-mcp`.
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
   coincide con la de `ssimulacra2`).
6. ~~**Prueba guiada de la v0.7.0 en el Mac del cliente**~~ **completa el 2026-10-14**: los cinco
   pasos bien. En el paso 5, la CLI de Mac dio lo mismo que el VPS con `playa-1.jpg` (calidad 90 para
   la nota 80, SSIMULACRA 2 83,2). **v0.7.1 publicada el 2026-10-14** con los tres arreglos; borrador
   revisado (22 ficheros, sumas, 14 firmas, la CLI busca, mide y da el mismo AVIF que avifenc).
   ~~Lo siguiente era la entrega 7, la 1.0~~: el cliente metió antes los agentes por MCP (punto 0), y la 1.0 es la entrega 8. Los pasos 1 (la nota), 2 (calidad por nota) y 3 (Diferencias) salieron bien.
   - El paso 4 es **Lotes midiendo**: `vacaciones` en WebP con «Medir la calidad» (nota por fila,
     nota media, «Las de peor nota»), y opcional otro lote con «Buscar la calidad por nota» a 80.
     El texto del paso está en sesiones.md (2026-10-08, tarde).
   - Después, el paso 5 (opcional): la CLI, con `apolo medir` y `apolo jpeg -apolo_objetivo 80`.
   - **Tres arreglos de la prueba, ya en `main` y sin publicar**, para una **v0.7.1** al terminar:
     - Exportar se cortaba con medidas largas: ahora medidas y Exportar bajan juntos de línea si no
       caben;
     - no se veía que buscaba o medía: ahora hay una rueda y «Buscando la calidad para una nota de
       80…» o «Midiendo la calidad…»;
     - el aviso «Vista reducida» pisaba el selector y el rótulo de Diferencias: ahora va abajo,
       sobre el zoom.
   - Lo que salió y no era fallo: con la nota en 95, JPEG «no llega ni con calidad 100» en
     `captura-1.png` (llega como mucho a 89,8). Con 80 llega con calidad 90.
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
- Prueba guiada de la v0.7.0, completa, y v0.7.1 publicada (2026-10-14).
- Entrega 7a: `apolo-mcp`, el servidor MCP con diez herramientas (2026-10-14). ADR 0023.
- Entrega 7b en el código: empaquetado, relevo, versión y la fila «MCP» (2026-10-14).

## En curso

- Entrega 7 (MCP): hechas la 7a y la 7b en el código; faltan CI, la v0.8.0 con su prueba guiada y la 7c (evaluación).
