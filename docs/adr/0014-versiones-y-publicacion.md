# ADR 0014 — Versiones 0.x como pre-release, con la CLI en la Release

**Fecha:** 2026-10-06 · **Estado:** aceptada · matizada por la [0018](0018-actualizarse-sola.md): desde la v0.3.3, versiones normales y no pre-release, para que el actualizador las vea

## Contexto

Tras la entrega 2, el cliente preguntó si se podían hacer ya las Releases de GitHub o si iban en una
entrega posterior. El flujo de publicar existía desde la entrega 0, pero lo que la entrega 6 añade
—actualización automática, icono definitivo, tabla de descargas— no estaba. Pidió sacar la
**v0.2.0 con la CLI incluida**.

## Decisión

- **Las versiones 0.x se publican como pre-release**, en borrador, y el borrador se publica a mano
  después de mirarlo. La 1.0 llega con la entrega 6.
- La Release lleva, además de los instaladores, **la CLI `apolo` para los seis objetivos** (universal
  en macOS, `.zip` en Windows, `.tar.gz` en Linux, con la licencia dentro) y **`SHA256SUMS.txt`**
  con la suma de cada fichero.
- Las notas de cada versión viven en el repositorio, en `.github/notas/vX.Y.Z.md`, y la publicación
  las pone en la Release.
- Las versiones siguen a las entregas: 0.2 es la entrega 2.

## Alternativas descartadas

- **Esperar a la 1.0.** Mientras tanto, probar exige iniciar sesión en GitHub y bajar artefactos de
  Actions, que caducan a los 90 días.
- **Publicar directamente, sin borrador.** Una Release en un repositorio público la ve cualquiera;
  mejor mirarla antes.

## Consecuencias

- Quien instale una 0.x tiene que bajarse a mano las siguientes hasta que llegue la actualización
  automática.
- Cada versión necesita su fichero de notas; si falta, la Release sale con las notas vacías.

## Verificación

2026-10-06, con la **v0.2.0** (https://github.com/webcafeina/Apolo/releases/tag/v0.2.0):

- `publicar.yml` lanzado antes a mano para ver la CLI en las seis máquinas, y después con la
  etiqueta: los once trabajos y `adjuntar` en verde.
- Borrador revisado antes de publicar: 7 instaladores, 5 paquetes de la CLI y `SHA256SUMS.txt`.
- La CLI de Linux amd64 **descargada de la Release**: la suma cuadra con `SHA256SUMS.txt` y
  `apolo webp -q 80` da el mismo fichero que el cwebp 1.6.0 oficial.
- Tauri cuelga además `Apolo_universal.app.tar.gz`, que es lo que usará la actualización automática;
  hoy sobra, pero no molesta.
- **No verificado**: ningún instalador de la Release se ha abierto.
