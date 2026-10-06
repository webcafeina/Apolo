# ADR 0007 — En español, con i18n desde el primer día

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

Esfinge está escrita solo en español, con los textos dentro del código (su ADR 0005). Apolo es
público y gratuito, y es más fácil que llegue a quien no habla español.

## Decisión

**Arranca en español, pero ningún texto visible se escribe dentro de un componente**: todos van en
`frontend/src/i18n/es.json` y se piden por clave. Mayúscula inicial en cada frase, como en Esfinge.

## Alternativas descartadas

- **Solo español, con los textos en el código**, como Esfinge. Traducir después obliga a recorrer
  todos los componentes.
- **Español e inglés desde el principio.** Doble trabajo con cada texto mientras la interfaz cambia
  cada día.

## Consecuencias

- Añadir el inglés es traducir un fichero.
- La CLI se queda en español; si se traduce, irá aparte.

## Verificación

Elegida con el cliente el 2026-10-06.
