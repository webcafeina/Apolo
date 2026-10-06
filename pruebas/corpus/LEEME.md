# Corpus de pruebas

- `foto.webp`: la imagen de ejemplo de libwebp 1.6.0 (`examples/test.webp`), con licencia BSD,
  como el resto de libwebp (© Google). A partir de ella, `crates/nucleo/tests/equivalencia_cwebp.rs`
  genera el resto del corpus en cada ejecución: PNG, JPEG, TIFF, PNM, WebP y YUV con todas las
  variantes que importan para comparar con cwebp. Así no hay más binarios en el repositorio.
- `orientacion-6.png`: 64×32, mitad roja y mitad azul, con un trozo eXIf que dice orientación 6
  (girar 90° a la derecha). Para las pruebas de enderezar (ADR 0012). Hecha a mano para esto.
