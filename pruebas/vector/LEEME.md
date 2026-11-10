# Imágenes de vectorizar

- `logo.png`: 200×200, un aro rojo, un cuadrado azul girado y una estrella naranja, suavizados y sobre fondo
  transparente. Para las pruebas de la pantalla «Vectorizar» (`frontend/e2e/vectorizar.spec.ts`, ADR 0039).
  Hecha a mano para esto (2,2 KB), sin licencia ajena. Va aparte de `pruebas/corpus/` porque las pruebas de
  Lotes convierten esa carpeta entera y cuentan sus imágenes.
- `apolo-plano.svg` y `apolo-plano.png` (512×512, esquinas transparentes): el sol del icono de Apolo con colores
  planos en vez de degradados, el ejemplo de la web (los degradados que comparten color aún salen en anillos:
  docs/deuda.md). El PNG sale del SVG con `cargo run -p apolo-vector --example rasterizar -- apolo-plano.svg 512
  apolo-plano.png`. Nuestro, sin licencia ajena.
