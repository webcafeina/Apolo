# ADR 0037 — Sin firma de Claude en el historial

**Fecha:** 2026-11-06 · **Estado:** aceptada

## Contexto

GitHub mostraba a @claude como colaborador de `webcafeina/Apolo`, porque todos los commits acababan en
`Co-Authored-By: Claude…` (253 de 253, y 4 de 17 en el tap `homebrew-apolo`). El cliente pidió quitarlo.
La línea está en los mensajes de los commits, así que solo sale reescribiendo el historial.

## Decisión

- **Reescribir el historial** con `git-filter-repo` y un `--message-callback` que quita solo esa línea,
  en un clon aparte, y subir `main` y las 36 etiquetas con `--force`. Autor, fechas y árboles no cambian.
- **Desde ahora**, ni commits ni PR llevan esa firma ni «Generated with Claude Code».
- **`CLAUDE.md` y `docs/` se quedan** en el repositorio (decisión del cliente): lo que le hacía salir
  como colaborador era la firma.
- Los hashes citados en `docs/` se cambian por los nuevos en un commit normal, usando el `commit-map`.

## Alternativas descartadas

- **Dejar el historial y dejar de firmar en adelante**: @claude seguiría en la lista de colaboradores.
- **Un repositorio nuevo con el historial aplastado**: se pierden las Releases, las estrellas, las
  bifurcaciones, los avisos de Security y la historia de cada decisión.
- **Sacar también `CLAUDE.md` y limpiar `docs/`**: más historial reescrito, y sin ese fichero en GitHub
  se perderían las instrucciones si se pierde el VPS.

## Consecuencias

- **Cambian todos los hashes.** Quien tenga un clon tiene que volver a clonar. Los hashes de los
  documentos están al día; los de las ejecuciones viejas de CI apuntan al commit antiguo.
- **Las dos bifurcaciones de terceros** (`overlook940/Apolo` y `blueoceandevops/Apolo`) conservan el
  historial viejo, y los commits viejos pueden seguir abriéndose por su hash, hasta que GitHub los
  limpie o se le pida. Tampoco cambian `refs/pull/*` ni las ramas viejas de Dependabot. No cuentan para
  la lista de colaboradores, que sale de la rama principal.
- **La procedencia firmada de las versiones publicadas** cita el commit antiguo en sus metadatos. Se
  sigue verificando, porque va por la suma de cada fichero.
- Las Releases no se tocan: van por nombre de etiqueta.

## Verificación

- 2026-11-06, antes de subir: en el clon reescrito, 254 commits con el mismo árbol, autor y fechas; los
  mensajes, iguales salvo la firma y los hashes citados; las 36 etiquetas, en su commit reescrito; 0
  firmas. El tap, 17 commits con el mismo árbol.
- Copia previa en `~/.cache/apolo-revision/antes-de-reescribir/` y en R2
  (`antes-de-reescribir-2026-11-06/`), como bundles verificados.
- 2026-11-06, después de subir: `main` y las etiquetas, en GitHub con los hashes nuevos; CI en verde;
  36 Releases, la 1.0.2 con sus 50 ficheros; enlace fijo, `latest.json` y `gh attestation verify`, bien;
  la API de colaboradores, solo `webcafeina`.
- **Sin confirmar**: la lista de colaboradores de la web (`contributors_list`) seguía mostrando a
  @claude al cerrar la sesión. Si sigue en unos días, se pide a GitHub Support que la recalcule.
- **Sin ver**: que el envío al directorio de Anthropic siga bien con el commit nuevo (lo mira el cliente).
