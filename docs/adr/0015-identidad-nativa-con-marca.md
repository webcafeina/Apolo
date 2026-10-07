# ADR 0015 — Nativa con marca, como Esfinge hoy

**Fecha:** 2026-10-06 · **Estado:** aceptada · hecha en la entrega 3 · **Matiza** la [ADR 0008](0008-aspecto-del-sistema.md)

## Contexto

Al probar la v0.2.0 en su Mac, el cliente dijo de la aplicación que *«apenas tiene diseño visual»*,
de la ventana del `.dmg` que *«debe ser visualmente rica, con la identidad visual, al igual que hace
Esfinge»*, y del icono que *«debe ser más épico, más Apolo; hay que proponer alternativas»*. La ADR
0008 había dejado la marca solo en el icono. Esfinge empezó igual (su ADR 0007), pero luego añadió
un acento de marca (su ADR 0021), la barra translúcida de macOS y su propia ventana de `.dmg`.

## Decisión

Elegida con el cliente en tres preguntas:

- **Nativa con marca, como Esfinge hoy**: la ventana sigue pareciendo del sistema (tipografía y
  controles), pero con **acento propio de Apolo**, **barra lateral translúcida** en macOS, una
  **pantalla vacía con el icono y una ilustración**, y el **`.dmg` con fondo propio**.
- **Ahora, antes de Lotes**: una entrega «Identidad» (pasa a ser la 3; Lotes, la 4). Empieza con
  **propuestas de icono** para que el cliente elija.
- **Sin licencia en el `.dmg`**: era la GPLv3 como contrato que había que aceptar al instalar. La GPL
  no lo pide; la licencia sigue dentro de la aplicación y en el repositorio.

## Alternativas descartadas

- **Identidad propia completa** (paleta, tipografía y formas en toda la ventana). Más marca, menos
  «del sistema»; no es lo que pidió.
- **Solo icono y `.dmg`**. Deja la ventana tan sobria como ahora, que es la queja.
- **Esperar a Lotes o a la 1.0.** Lotes heredaría un diseño que luego habría que rehacer.

## Consecuencias

- El acento de marca pasa por `crates/tema` y la prueba de contraste, como el oro de Esfinge: el
  color de marca rellena, y el texto encima se elige por contraste medido, no a ojo.
- El color de la interfaz deja de ser solo gris y azul del sistema. Hay que vigilar que el cromo no
  tiña el juicio sobre la foto: el acento va en controles, no alrededor de la imagen.

## Verificación

Decidida el 2026-10-06 y hecha el mismo día (entrega 3).

- **Icono**: cuatro propuestas (`empaquetado/iconos/propuestas/`) en la misma familia que Esfinge,
  enseñadas en una página con cada una a 1024, 128, 64, 32 y 16 px, en claro y oscuro y en un Dock
  junto a Esfinge. La página, tal como la vio el cliente, se guarda de recuerdo en
  `empaquetado/iconos/propuestas/propuestas-icono.html` (se abre sin conexión). Hubo una segunda vuelta: en la primera, el sol y el arco se quedaban cortos y el
  laurel parecía confeti. **Eligió el sol con la corona de laurel.**
- **Oro del sol** `#ffc83d` como `--relleno`, con piedra encima (más de 9:1). Una prueba deja
  escrito que el blanco sobre el oro no llega ni a 3:1.
- **Marca a trazo** (`empaquetado/marca.svg`) para el sello de la barra y la bienvenida. La primera
  versión, con las hojas sueltas, parecía un insecto a 22 px.
- **`.dmg`**: fondo propio (simulado sin Mac con `make ventana-dmg`) y sin licencia que aceptar.
- Capturas de la bienvenida, el Estudio y Ajustes, en claro y oscuro.
- Con la v0.3.0 en su Mac, el cliente vio bien la ventana del `.dmg`, el vidrio («correcto, aunque
  muy sutil»; se deja así, como el del sistema) y el icono del Dock. Pidió además:
  - el **icono del volumen** como el de Esfinge, una unidad nativa de Mac con el sol en el centro;
  - **identidad en el instalador de Windows**;
  - la **ventana con las proporciones de la de Esfinge**.

  Va en la **v0.3.1**:
  - `empaquetado/macos/disco.svg`, la carcasa de Esfinge con el sol, que se pone en el `.dmg` con
    `icono-volumen.sh` en CI, porque Tauri no deja elegirlo;
  - NSIS en español con las imágenes `lateral.bmp` y `cabecera.bmp`;
  - la hoja de estilos rehecha con las medidas de Esfinge, iconos de línea y variantes de Windows y
    Linux.
- **No verificado**: el icono del volumen en un Mac y el instalador de Windows en un Windows.
