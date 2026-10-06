# ADR 0012 — Enderezar según la orientación EXIF, como opción

**Fecha:** 2026-10-06 · **Estado:** aceptada · **Matiza** la [ADR 0011](0011-leer-como-cwebp.md)

## Contexto

cwebp no aplica la orientación EXIF: una foto de móvil girada sale girada. Apolo, por fidelidad
(ADR 0011), tampoco. Se le planteó al cliente al cerrar la entrega 1 y respondió: *«Sí, quiero que
tenga la opción de enderezar, como opción.»*

## Decisión

**Enderezar es una opción de Apolo**, en el preset (`OpcionesWebp`, y la equivalente en los demás
códecs), **apagada por defecto** para conservar la promesa de la ADR 0002.

- Lee la etiqueta Orientation (1–8) del EXIF de la entrada y gira o voltea los píxeles **antes** de
  recortar y redimensionar, que es lo que espera quien ve la foto derecha.
- Al enderezar, el EXIF que se copie a la salida tiene que llevar Orientation = 1; si no, quien lo
  abra la girará dos veces.
- **No es una opción de cwebp**: con ella encendida, la interfaz marca que la orden cwebp equivalente
  ya no da el mismo fichero, y `apolo webp` la acepta con un nombre propio que no choque con las de
  cwebp (por decidir al hacerla; algo como `-apolo_orientar`).
- El Estudio debería avisar cuando la imagen trae una orientación distinta de 1 y la opción está
  apagada: es el caso en que el resultado «sale girado».

## Alternativas descartadas

- **Enderezar siempre.** Rompe la equivalencia con cwebp en todas las fotos de móvil.
- **No enderezar nunca**, como cwebp. Es lo que pidió el cliente que no fuera.

## Consecuencias

- La orden cwebp equivalente deja de ser siempre exacta: hay que saber decir cuándo no lo es.
- La prueba de equivalencia sigue igual (la opción va apagada); hace falta una prueba propia con las
  ocho orientaciones.

## Verificación

Decidida con el cliente el 2026-10-06. Hecha la misma tarde, en la entrega 2:

- `orientacion.rs`: las ocho orientaciones, comprobadas contra la definición de EXIF (la prueba
  tenía al principio la 6 y la 8 cambiadas; la fórmula estaba bien).
- `-apolo_enderezar` en la CLI. Al principio no hacía nada: la CLI, como cwebp, no leía el EXIF si no
  se iba a copiar. Ahora lo lee también para enderezar (`enderezar_gira_y_deja_el_exif_a_1`).
- En el Estudio, el interruptor en Básico, el aviso cuando la foto viene girada y la marca «Con
  extras de Apolo» en la orden (Playwright).
- El nombre de la opción quedó `-apolo_enderezar`: todas las propias llevan el prefijo `-apolo_`.
