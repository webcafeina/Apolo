# ADR 0006 — Repositorio público con licencia GPLv3

**Fecha:** 2026-10-06 · **Estado:** aceptada · **Revisar si** Apolo se quiere vender con el código cerrado

## Contexto

Primero se habló de un repositorio privado y de un producto a la venta. A mitad de las preguntas
cambió: **de momento se da gratis, con el repositorio público**. Y con eso apareció la pregunta de
libimagequant —el cuantizador de Squoosh—, que es **GPLv3 o licencia comercial de pago** (el precio
no es público: se pide a su autor).

## Decisión

**Repositorio público `webcafeina/Apolo` con licencia GPLv3.** Con Apolo bajo GPLv3, libimagequant
se usa gratis. El resto de dependencias tiene licencias compatibles (BSD, MIT, Apache, IJG).

Regla que sale de aquí: **ninguna dependencia nueva sin mirar su licencia.** AGPL no —por eso no
`dssim`— y propietarias no.

## Alternativas descartadas

- **Propietaria y pública, como Esfinge** («todos los derechos reservados»). Obligaba a cambiar
  libimagequant por un cuantizador MIT con menos calidad de paleta.
- **MIT.** Lo mismo: libimagequant no cabe.

## Consecuencias

- Si algún día se quiere vender cerrado, hay que cambiar el cuantizador o comprar su licencia, y
  pedir permiso a quien haya contribuido código.
- Vender binarios GPL sí es posible; lo que no se puede es cerrar el código.
- Con las Releases públicas, la actualización automática puede leerlas sin token.

## Verificación

Elegida con el cliente el 2026-10-06. LICENSE copiado del texto oficial (API de licencias de GitHub).
