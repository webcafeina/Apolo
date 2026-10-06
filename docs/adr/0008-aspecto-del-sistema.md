# ADR 0008 — Apariencia del sistema

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

Se ofrecieron dos caminos: seguir al sistema operativo, como Esfinge (su ADR 0007), o tomar la
estructura de uno de los sistemas del catálogo de `~/sistemas-diseno-empresas`.

## Decisión

**Apolo sigue al sistema**: tipografía del sistema, modo claro u oscuro según el escritorio,
superficies grises neutras y controles con la forma de los nativos. **La marca solo está en el icono.**
El color de acción es el azul del sistema.

Los tokens se generan como en Esfinge (su ADR 0008), pero desde Rust: `crates/tema` define la paleta
de cada modo, calcula el contraste y escribe `frontend/src/tokens.css` con `make tokens`. **Una
pareja de colores que no llegue a AA hace fallar las pruebas.**

## Alternativas descartadas

- **Un sistema del catálogo.** Para una herramienta que se abre, se usa y se cierra, que se sienta
  del sistema pesa más que una identidad propia.

## Consecuencias

- Una herramienta de imágenes enseña mucha imagen: el cromo de alrededor tiene que callarse. Gris
  neutro también evita que el color de la interfaz engañe al juzgar el de la foto.

## Verificación

Elegida con el cliente el 2026-10-06. El contraste lo mide `cargo test -p apolo-tema`.
