# ADR 0015 — Nativa con marca, como Esfinge hoy

**Fecha:** 2026-10-06 · **Estado:** aceptada · **Matiza** la [ADR 0008](0008-aspecto-del-sistema.md)

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

Decidida el 2026-10-06. Sin hacer.
