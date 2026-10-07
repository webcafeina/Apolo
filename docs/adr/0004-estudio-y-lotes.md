# ADR 0004 — Estudio y lotes

**Fecha:** 2026-10-06 · **Estado:** aceptada · Lotes, desarrollado en la [0019](0019-lotes.md)

## Contexto

Squoosh es una imagen cada vez. El trabajo real a menudo es una carpeta entera.

## Decisión

Dos flujos sobre el mismo núcleo y los mismos presets:

- **Estudio**: una imagen, afinada viéndola. Comparador antes/después con deslizador, lado a lado y
  mapa de diferencias; zoom y desplazamiento sincronizados; peso, ahorro y métricas en vivo.
- **Lotes**: carpetas o ficheros arrastrados, un preset (con uno o varios formatos de salida), un
  patrón de nombre y de carpeta, en paralelo y cancelable, con un resumen al final.

Lo afinado en el Estudio se guarda como preset y se usa en Lotes o en la CLI.

## Alternativas descartadas

- **Solo una imagen**, como Squoosh. Deja fuera el caso más frecuente.
- **Solo lotes.** Sin dónde afinar, el preset se elige a ciegas.

## Consecuencias

- El preset es el centro del modelo: un JSON con formato y opciones que entienden la ventana y la CLI.

## Verificación

Elegida con el cliente el 2026-10-06.
