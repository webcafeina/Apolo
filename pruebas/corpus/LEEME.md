# Corpus de pruebas

- `foto.webp`: la imagen de ejemplo de libwebp 1.6.0 (`examples/test.webp`), con licencia BSD,
  como el resto de libwebp (© Google). A partir de ella, `crates/nucleo/tests/equivalencia_cwebp.rs`
  genera el resto del corpus en cada ejecución: PNG, JPEG, TIFF, PNM, WebP y YUV con todas las
  variantes que importan para comparar con cwebp. Así no hay más binarios en el repositorio.
