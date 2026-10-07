# ADR 0005 — Redimensionar, reducir paleta, la CLI y el mapa de diferencias

**Fecha:** 2026-10-06 · **Estado:** aceptada · redimensionar, recortar y reducir paleta hechos en la [0020](0020-mas-formatos-y-proceso.md)

## Contexto

Más allá de codificar, Squoosh y `cwebp` hacen cosas alrededor: redimensionar y recortar
(`-resize`, `-crop`), reducir la paleta, y `cwebp` imprime métricas (`-print_psnr`, `-print_ssim`).

## Decisión

Los cuatro extras que eligió el cliente:

- **Redimensionar y recortar**, con varios filtros (Lanczos3, Mitchell, CatmullRom…) mediante
  `fast_image_resize`.
- **Reducir paleta** con libimagequant, el mismo cuantizador que Squoosh
  ([ADR 0006](0006-gplv3-y-repositorio-publico.md) explica por qué se puede usar gratis).
- **Una CLI `apolo`**, como Esfinge tiene la suya: mismos presets, para scripts. `apolo webp` acepta
  las opciones de `cwebp` tal cual.
- **El mapa de diferencias**, con PSNR y SSIM. El SSIM se escribe aquí: `dssim` es AGPL.

## Alternativas descartadas

- **Butteraugli** desde el principio. Se queda en prioridad baja: es caro de calcular y no hay una
  librería suelta con licencia cómoda.

## Consecuencias

- La CLI es una segunda cara que mantener, pero es la misma que la prueba de equivalencia con `cwebp`
  necesita de todos modos.

## Verificación

Elegida con el cliente el 2026-10-06.
