# Cobertura de cjpeg, oxipng y qoiconv

Última actualización: **2026-10-07** (entrega 5, v0.5)

La misma regla que en [cobertura-cwebp.md](cobertura-cwebp.md): una opción no está hecha hasta que
su fila está completa (ADR 0020).

- **CLI**: ✓ si `apolo jpeg` o `apolo png` la aceptan con el mismo efecto.
- **Interfaz**: ✓ y la sección del panel del Estudio: **B**, **A** o **E**. El panel se pinta desde
  `frontend/src/estudio/opciones.ts`.
- **Prueba**:
  - **eq**: está en la lista de su prueba de equivalencia (`equivalencia_cjpeg.rs` o
    `equivalencia_png_qoi.rs`), que compara con la herramienta oficial **byte a byte**;
  - **ida**: la cubre la prueba de ida y vuelta opciones ⇄ orden.

## cjpeg (MozJPEG 4.1.5)

Referencia: `cjpeg.c` y `rdswitch.c` de MozJPEG 4.1.5. El núcleo las traslada en
`crates/nucleo/src/jpeg/cjpeg.rs`; las opciones viven en `jpeg/opciones.rs` (`OpcionesJpeg`).

| Opción | Qué hace | Núcleo | CLI | Interfaz | Prueba |
|---|---|---|---|---|---|
| `-quality N[,…]` | Calidad, una por tabla; desde 80 y 90 quita submuestreo | `calidad` | ✓ | ✓ B | eq, ida |
| `-grayscale` | En gris | `color: Gris` | ✓ | ✓ B | eq, ida |
| `-rgb` | RGB sin convertir a YCbCr | `color: Rgb` | ✓ | ✓ B | eq, ida |
| `-progressive` | Progresivo | `escaneo: Progresivo` | ✓ | ✓ B | eq, ida |
| `-baseline` | Secuencial y tablas de 8 bits | `escaneo: Secuencial` | ✓ | ✓ B | eq, ida |
| `-revert` | Los valores de libjpeg; deshace lo anterior de cinfo | `revertir` | ✓ | ✓ A | eq, ida |
| `-tune-psnr`, `-tune-hvs-psnr`, `-tune-ssim`, `-tune-ms-ssim` | Afinado del trellis | `afinado` | ✓ | ✓ A | eq, ida |
| `-quant-table N` | Tabla base 0–8 | `tabla` | ✓ | ✓ A | eq, ida |
| `-sample HxV[,…]` | Submuestreo | `muestreo` | ✓ | ✓ A (1x1, 2x1, 2x2) | eq, ida |
| `-smooth N` | Suavizado 0–100 | `suavizado` | ✓ | ✓ A | eq |
| `-notrellis` | Sin trellis | `sin_trellis` | ✓ | ✓ A | eq |
| `-fastcrush` | Sin optimizar escaneos | `rapido` | ✓ | ✓ E | eq |
| `-dc-scan-opt N` | Escaneo DC 0–2 | `dc_scan_opt` | ✓ | ✓ E | eq |
| `-dct int\|fast\|float` | Método DCT | `dct` | ✓ | ✓ E | eq, ida |
| `-quant-baseline` | Tablas de 8 bits | `tablas_baseline` | ✓ | ✓ E | eq, ida |
| `-noovershoot` | Sin desbordamiento contra halos | `sin_overshoot` | ✓ | ✓ E | eq |
| `-nojfif` | Sin cabecera JFIF | `sin_jfif` | ✓ | ✓ E | eq |
| `-optimize` | Huffman a medida (ya lo hace MozJPEG) | `optimizar` | ✓ | ✓ E | eq, ida |
| `-notrellis-dc`, `-trellis-dc` | Trellis de la DC | `trellis_dc` | ✓ | — | eq, ida |
| `-trellis-dc-ver-weight F` | Peso vertical de la DC | `peso_dc` | ✓ | — | eq |
| `-lambda1 F`, `-lambda2 F` | Lambdas del trellis | `lambda1`, `lambda2` | ✓ | — | eq |
| `-qslots N[,…]` | Tabla por componente | `ranuras` | ✓ | — | eq |
| `-restart N[B]` | Intervalo de reinicio | `reinicio` | ✓ | — | eq, ida |
| `-icc FICHERO` | Incrustar un perfil | `cjpeg::Extra::icc` | ✓ | — | — |
| `-strict` | Avisos como errores | `cjpeg::Extra::estricto` | ✓ | — | — |
| `-outfile` | Salida | — | ✓ | (exportar) | — |
| `-verbose`, `-debug`, `-report`, `-memdst`, `-maxmemory`, `-targa` | No cambian el fichero | — | se ignoran con aviso | — | — |
| `-version` | Versión | — | ✓ | — | — |
| `-arithmetic` | Codificación aritmética | — | error, como el oficial | — | eq (los dos la rechazan) |
| `-qtables FICHERO`, `-scans FICHERO` | Tablas y escaneos de un fichero | — | **no está** (error) | — | — |

**Entradas:** PNG, JPEG y PNM, comprobadas. BMP, GIF y Targa las lee cjpeg, pero no está comprobado
que Apolo dé los mismos píxeles (deuda). Lo demás no lo lee cjpeg; Apolo lo convierte igual y lo
avisa.

## oxipng 10.2.1

Referencia: `src/cli.rs` y `parse_opts_into_struct` de `src/main.rs`. Núcleo:
`crates/nucleo/src/formatos/png.rs` (`OpcionesPng`). El orden no importa.

| Opción | Qué hace | Núcleo | CLI | Interfaz | Prueba |
|---|---|---|---|---|---|
| `-o 0–6\|max` | Nivel | `nivel` | ✓ | ✓ B | eq, ida |
| `-a` | Optimizar la transparencia | `alfa` | ✓ | ✓ B | eq, ida |
| `-s`, `--strip safe\|all` | Quitar metadatos | `quitar` | ✓ | ✓ B | eq, ida |
| `-i on\|off\|keep` | Entrelazado (keep por defecto con `--nx`) | `entrelazado` | ✓ | ✓ A | eq, ida |
| `-z`, `--zi N`, `--ziwi N` | Zopfli e iteraciones | `zopfli`, `iteraciones`, `sin_mejora` | ✓ | ✓ A (sin `--ziwi`) | eq, ida |
| `--zc N` | Nivel de libdeflate 0–12 | `compresion` | ✓ | ✓ A | eq |
| `--fast` | Evaluación rápida | `rapido` | ✓ | ✓ A | eq |
| `--nx` | Sin reducciones | `sin_reducciones` | ✓ | ✓ E | eq |
| `--nb`, `--nc`, `--np`, `--ng` | Sin reducir bits, color, paleta o gris | `sin_bits`… | ✓ | ✓ E | eq |
| `--nz` | Sin recomprimir | `sin_recodificar` | ✓ | ✓ E | eq |
| `--scale16` | De 16 a 8 bits escalando | `escala16` | ✓ | ✓ E | eq |
| `--force` | Escribir aunque no gane | `forzar` | ✓ | ✓ E | eq |
| `--fix` | Arreglar PNG dañados | `arreglar` | ✓ | — | eq |
| `-f LISTA` | Filtros (número, rango o lista) | `filtros` | ✓ | — | eq, ida |
| `--brute-level N`, `--brute-lines N` | Fuerza bruta | `bruta_nivel`, `bruta_lineas` | ✓ | — | eq |
| `--out`, `--stdout` | Salida (sin ellas, reescribe la entrada) | — | ✓ | (exportar) | — |
| `-v`, `-q`, `-p`, `-r`, `-t`, `-j`, `--sequential` | No cambian el fichero | — | se ignoran con aviso | — | — |
| `--keep`, `--strip` con lista | Trozos concretos | — | **no está** (error) | — | — |
| `--timeout`, `--max-raw-size`, `--dir`, `-d` | — | — | **no está** (error) | — | — |

## qoiconv

No tiene opciones: `qoiconv entrada.png salida.qoi`. Núcleo: `crates/nucleo/src/formatos/qoi.rs`.
CLI: `apolo qoi`. Prueba: eq, 13 de 13. Solo lee PNG; de otro formato, Apolo convierte igual y lo
avisa.
