# Sesiones

Bitácora: qué se hizo en cada sesión, la más reciente arriba. La plantilla está al final.

---

## 2026-10-08 (tarde) · v0.7.0 publicada, y la prueba guiada hasta el paso 3

- **v0.7.0 publicada**. CI falló primero en la ventana porque los cambios de `src-tauri` se
  quedaron fuera del commit (trampa); con ellos, los seis objetivos en verde. Borrador revisado: 22
  ficheros, sumas, 14 firmas, y la CLI con la misma nota que `ssimulacra2` (81,19 en las dos).
- **Prueba guiada en el Mac del cliente**: pasos 1 (la nota), 2 (calidad por nota) y 3
  (Diferencias), bien. Tres apuntes, arreglados en `main` para la v0.7.1:
  - Exportar se cortaba por la derecha con medidas largas. Se probó a estrechar las medidas (partían
    «SSIMULACRA / 2») y a que cedieran los pesos (se pisaban). Al final, medidas y Exportar van juntos
    y bajan a otra línea si no caben, partiendo solo entre piezas enteras.
  - Pasaban 4-5 s sin señal mientras buscaba la calidad: ahora hay una rueda y «Buscando la calidad
    para una nota de 80…», con su prueba e2e.
  - El aviso «Vista reducida» pisaba el selector y el rótulo de Diferencias: va abajo, sobre el zoom.
- Con la nota en 95, JPEG no llegaba en `captura-1.png`: es lo esperado (como mucho 89,8).
- **Se sigue el lunes por el paso 4** (Lotes midiendo). El texto que se le dio:
  1. Lotes, carpeta `vacaciones`, una salida WebP por defecto.
  2. En Calidad, «Medir la calidad» encendido; «Buscar la calidad por nota», apagado.
  3. Al terminar: nota en cada fila, «Nota media (SSIMULACRA 2)» en el resumen y «Las de peor nota».
  4. Opcional: otro lote con «Buscar la calidad por nota» a 80. Cada fila con «calidad N», media de
     80 o algo más; alguna foto con ruido de color puede no llegar en WebP y quedarse en 100.

## 2026-10-08 · Entrega 6: medir la pérdida (v0.7.0, sin publicar)

- Al plantearla, salió que libjxl, dentro desde la 5b, trae **SSIMULACRA 2** (y Butteraugli) con
  licencia BSD. El cliente eligió SSIMULACRA 2 con PSNR y SSIM, un modo «Diferencias», medir opcional
  en Lotes y **la calidad por nota en esta misma entrega** ([ADR 0022](adr/0022-medir-la-perdida.md)).
- **SSIMULACRA 2**: `crates/avifjxl/c/medir.cc` compila `tools/ssimulacra2.cc` de libjxl y la llama
  con píxeles en memoria. **Igual a 8 decimales que la herramienta oficial**: 56 de 56 en
  `make equivalencia`.
  - Cuesta 4 s y 1,75 GB con 12 MP en el VPS, como la oficial. Por eso se mide una a la vez en todo
    Apolo, y en el Estudio después de la vista previa, cancelable por generación.
- **PSNR, SSIM y los dos mapas**, escritos en `crates/nucleo/src/medir.rs`. El de estructura saturaba
  con ×4 y pasó a ×2, al verlo en una captura.
- **La calidad por nota**: búsqueda binaria de 0 a 100 en `salida::codificar`; la orden lleva la
  calidad encontrada. Se cambió para no probar la 100 la primera: era lenta, y avifenc avisaba con
  `-q 100`.
- Servicio (`medir`, mapa por píxeles), Tauri y `apolo-dev`; Lotes con nota por fila, nota media y
  las de peor nota; CLI con `apolo medir`, `-apolo_objetivo` y `apolo lote --medir --objetivo`.
- En la interfaz, el selector del mapa apretaba la cabecera y ocultaba el nombre: pasó dentro del
  comparador.
- Verificado: `make comprobar`, `make equivalencia` (la nota, 56 de 56), 53 pruebas e2e y capturas.

## 2026-10-08 · v0.6.0 y v0.6.1 publicadas

- **v0.6.0 publicada** tras revisar el borrador: 22 ficheros con sus sumas, 14 firmas válidas en
  `latest.json` (verificador de minisign rehecho en Python), y la CLI de la Release igual que
  avifenc y cjxl.
- Al revisarla, un JPEG hecho con ffmpeg (empieza con un comentario, `FF D8 FF FE`) no se pasa a
  píxeles en cjxl, ni en el oficial: a la deuda. Y la comparación a mano con `$o` sin partir en zsh
  dio falsos «distintos», trampa ya conocida (`${=o}`).
- **Prueba guiada, paso 1:** «Sin pérdida» en AVIF después de bajar la calidad daba «avifenc
  terminó con el código 1». La orden llevaba `-q 40 -l`, que avifenc rechaza. Arreglado: un control
  apagado no cuenta en la orden (AVIF con `-l`; cjxl con un JPEG que recomprime sin pérdida), con
  una prueba e2e de su recorrido. Y la barra de la descarga ya no se ve llena antes de empezar.
- El cliente preguntó si AVIF y JPEG XL habían tocado la 5a: no. Entre la v0.5.0 y la v0.6.0 no
  cambia ni una línea de los codificadores de WebP, JPEG, PNG y QOI ni del proceso, y la
  equivalencia y las e2e de esos formatos siguen en verde.
- **v0.6.1 publicada** con los dos arreglos, revisada igual.
- **Prueba guiada con la v0.6.1, completa**:
  - AVIF y sus controles («Sin pérdida» ya sin error);
  - JPEG XL con un JPEG, sin pérdida y con calidad;
  - comparar AVIF con WebP;
  - exportar y volver a abrir AVIF y JPEG XL con transparencia;
  - Lotes con WebP y AVIF y «el más ligero», con los mismos números que en el VPS.
  - El cliente notó que el lado izquierdo nuevo copia los ajustes del derecho (los de AVIF del paso
    1): es así a propósito, y le pareció bien.
- **AVIF en Mac, igual que el oficial**: `apolo avif` de la v0.6.1 frente al avifenc 1.4.2 de
  `macOS-artifacts.zip`, 16 de 16, en el Mac del cliente con un script de bash pegado en la
  Terminal (descargado con curl, sin cuarentena).

## 2026-10-07 (noche) · Entrega 5b: AVIF y JPEG XL (v0.6.0, sin publicar)

- El cliente eligió **las bibliotecas enlazadas dentro**, la opción recomendada, frente a llevar los
  binarios oficiales al lado ([ADR 0021](adr/0021-avif-y-jpeg-xl.md)).
- Medido antes: avifenc compilado con gcc da lo mismo que el oficial (40 de 40); **cjxl solo con
  clang 18** (27 de 27), que es como sale el oficial. En el VPS no hay sudo: LLVM 18 descomprimido
  en `~/.local/llvm18`, con una libtinfo 5 de pega (trampas.md).
- **`crates/avifjxl`**:
  - libavif 1.4.2, aom 3.14.1, libyuv y libjxl 0.12.0 con sus dependencias, en diez submódulos;
  - un solo proyecto de CMake, con zlib y libpng una vez, el JPEG de mozjpeg-sys y el sharpyuv de
    libwebp-sys;
  - los `main` de avifenc, avifdec, cjxl y djxl con otro nombre, llamados con los argumentos de la
    orden;
  - los `exit()` de cjxl vuelven con `longjmp`, y la salida de avifenc se calla dentro de Apolo.
- **El núcleo**: `OpcionesAvif` y `OpcionesJxl`, con las opciones del panel y las demás tal cual;
  leer AVIF y JXL con avifdec y djxl; los motores con sus versiones.
- **La interfaz**: los dos formatos en el Estudio y en Lotes, con sus controles; los deslizadores
  enseñan el valor de la herramienta; la calidad de cjxl se apaga si recomprime un JPEG sin pérdida.
- **La CLI**: `apolo avif` y `apolo jxl` son avifenc y cjxl tal cual; Apolo solo interviene con
  sus opciones o con entradas que la herramienta no abre.
- Errores míos que cazaron las pruebas o CI:
  - `--opción=valor` de cjxl se guardaba con el valor repetido (la prueba de ida y vuelta);
  - el primer commit guardó los submódulos en la punta de su rama, no en la etiqueta (trampas);
  - en Linux ARM64 no enlazaba por `+whole-archive`, y en Windows libpng no encontraba zlib.h
    al generar su configuración con el awk de Git (trampas).
- Verificado:
  - `make comprobar`;
  - `make equivalencia`: **376 de 376 con avifenc y 280 de 280 con cjxl**, y las demás iguales,
    también en CI;
  - 49 pruebas e2e (eran 43);
  - capturas del panel de los dos formatos; sin desvío de color (PSNR igual en los tres canales).
- Abierto (deuda): la ventana sin ver; la equivalencia en macOS y Windows; rutas con tildes en
  Windows; los mapas de ganancia de los JPEG.

## 2026-10-07 (tarde) · Entrega 5a: JPEG, PNG, QOI y el proceso (v0.5.0)

- El cliente eligió las cuatro respuestas recomendadas ([ADR 0020](adr/0020-mas-formatos-y-proceso.md)):
  - dos versiones (AVIF y JPEG XL, en la v0.6);
  - la misma promesa que con cwebp para cada formato;
  - un formato por lado en el comparador;
  - «solo el más ligero» en Lotes.
- **JPEG**:
  - el `main` y el `parse_switches` de cjpeg 4.1.5 trasladados sobre `mozjpeg-sys`;
  - la imagen leída como cjpeg (gris, sin componer la transparencia, sin gamma, con sus
    marcadores);
  - `make cjpeg-oficial` compila el cjpeg de referencia con una libpng, todo desde la fuente.
  - **816 de 816 iguales.**
- **PNG**: `oxipng` 10.2.1 con sus opciones convertidas como su propio binario; libdeflater fijada
  a la de su `Cargo.lock`. **351 de 351.**
- **QOI**: `qoi_encode` de qoi.h trasladado (el crate da otros bytes) y la lectura de stb_image.
  **13 de 13.**
- Errores míos que cazaron las pruebas (a trampas.md):
  - el perfil sRGB de cjpeg copiado a mano, con 4 bytes de menos;
  - la codificación aritmética, que el oficial no trae;
  - el rango −3..2 de QOI, que debía ser −2..1.
- **Proceso** para todos los formatos: enderezar (pasa aquí), recortar, redimensionar
  (`fast_image_resize`, Lanczos3, lineal y premultiplicado) y paleta (libimagequant).
- **`salida::Ajuste`** guarda las opciones de todos los formatos, y los presets lo guardan entero
  (los de antes se siguen leyendo).
- **El servicio** va por imagen y lado.
- **La interfaz**:
  - un formato por lado;
  - el panel con los controles de los tres formatos y el grupo Proceso;
  - Lotes con una lista de salidas y «solo el más ligero».
- **La CLI**: `apolo jpeg`, `apolo png`, `apolo qoi`, y `apolo lote --formato --mas-ligero`. Con la
  carpeta de prueba, «el más ligero» guarda 20 WebP, 3 PNG y 1 JPEG: el pixel art y el GIF, que en
  WebP crecían, salen en PNG un 70 % más ligeros.
- `git push` dio «Internal Server Error» tres veces seguidas; esperando un minuto, pasó (trampas).
- Verificado:
  - `make comprobar`;
  - `make equivalencia` (las cuatro herramientas);
  - 43 pruebas e2e;
  - capturas.
- **Prueba guiada en el Mac del cliente**, con la v0.5.0 recibida por el actualizador y la carpeta
  de prueba regenerada. Salieron bien los cinco pasos: JPEG, PNG y QOI, comparar formatos,
  redimensionar, y un lote con dos formatos y «el más ligero».
  - Pidió tres retoques, hechos para la v0.5.1:
    - el «%» del ahorro que bajaba de línea;
    - el hover de los selectores segmentados, que pisaba el fondo de la opción elegida;
    - «Volver al original» como botón.
- **Publicada la v0.5.0** cuando el cliente lo dijo. Antes de publicar:
  - CI y los 11 trabajos de publicar, en verde;
  - las sumas cuadran;
  - las 14 firmas de `latest.json` son válidas;
  - `apolo jpeg` de la Release da el mismo fichero que cjpeg.
- **No verificado**: nada en la ventana (lo verá el cliente), BMP, GIF y Targa en cjpeg, y lotes
  grandes con varios formatos (deuda).

---

## 2026-10-07 · Prueba de Lotes en el Mac, y la v0.4.1

- El cliente probó Lotes en su Mac (v0.4.0) con la carpeta de `pruebas/lotes/generar.sh`, que se
  bajó del VPS. Salieron bien siete pasos: elegir carpetas, soltar sin que el Estudio se lo quede,
  preset, convertir (24 + 1 rota, −76 %, dos que crecen), mostrar en la carpeta, repetir sin pisar
  y cancelar. La CLI no la tenía.
- Pidió tres cosas, hechas:
  - el desplegable «Formato» en Lotes, como en el Estudio;
  - «Cancelar» y el resumen cancelado en rojo.

  Y de la prueba salió una tercera: «Mostrar en la carpeta» abría la subcarpeta de la última
  imagen, y ahora enseña la carpeta de salida.
- Preguntó si la CLI se puede instalar desde un repositorio. Hoy no. Queda anotado para el futuro
  (Homebrew con un tap propio, primero) en siguiente.md.
- **Publicada la v0.4.1** con todo eso y la barra de la descarga. Antes de publicar:
  - CI en verde, también `comprobar` sobre el mismo commit;
  - las sumas y las 14 firmas de `latest.json`, verificadas;
  - el endpoint responde 0.4.1.

---

## 2026-10-07 · La primera actualización de verdad, y su barra

- El cliente **se actualizó de la v0.3.3 a la v0.4.0 desde la banda**, sin reinstalar. Saldada en
  macOS la deuda más alta.
- Avisó de que la barra de la descarga «no funciona bien»: avanzaba a saltos o iba por detrás.
  - Primero se descartó lo evidente: GitHub manda `Content-Length` (11,1 MB) y el plugin pasa bien
    los datos.
  - Se reprodujo en el navegador haciendo la descarga simulada parecida a la real. Con eso, la barra
    iba hasta 41 puntos por detrás del número.
  - La causa: un aviso por trozo, cientos por segundo, y cada uno reiniciaba la transición CSS
    ([trampas](trampas.md)).
  - El arreglo: redibujar solo cuando cambia el porcentaje entero, y quitar la transición. Con una
    transición de 80 ms aún iba 19 puntos por detrás.
  - Una prueba e2e compara la barra pintada con el número.
- Verificado: `make comprobar` y 31 pruebas e2e. Sin publicar: sería la v0.4.1.
- Para que el cliente pruebe Lotes en su Mac, `pruebas/lotes/generar.sh` fabrica con ffmpeg una
  carpeta de prueba: 25 imágenes sintéticas en subcarpetas (JPG, PNG con alfa, HEIC, TIFF, BMP, GIF,
  WebP), una rota, ocultos de macOS que no deben contar, y 48 fotos de 12 megapíxeles para cancelar.
  Pasada por `apolo lote`:
  - 24 convertidas y 1 fallida, 12,9 MB → 3,1 MB;
  - el pixel art y el GIF de patrón pesan unas cuatro veces más en WebP con pérdida.

  De ahí salió un arreglo pequeño de la CLI: con las opciones por defecto escribía «cwebp » con un
  espacio colgando.

---

## 2026-10-07 · Entrega 4: Lotes (v0.4.0)

- El cliente instaló la v0.3.3 a mano, y «Buscar ahora» dice «Ya tienes la última versión.». La
  actualización de verdad se verá con la siguiente versión de las entregas, no con una v0.3.4 aparte.
- Decidió dejar la clave del actualizador en el VPS, sabiendo los riesgos (ADR 0018, deuda).
- Para Lotes eligió las cuatro respuestas recomendadas ([ADR 0019](adr/0019-lotes.md)):
  - una carpeta elegida, con las subcarpetas;
  - si el nombre existe, se añade un número;
  - las que crecen se guardan y se señalan;
  - `apolo lote` en la CLI ya.
- Hecho:
  - `nucleo::lote`: recoger, no pisar con `create_new`, paralelo con `thread::scope`, cancelar y el
    resumen;
  - en el servicio, empezar, estado y cancelar, por Tauri y por HTTP;
  - la sección Lotes;
  - `apolo lote`;
  - `tauri-plugin-opener` para «Mostrar en la carpeta»;
  - el Estudio solo se queda lo que se suelta si es la sección que se ve.
- Verificado:
  - `make comprobar`;
  - 29 pruebas e2e;
  - 7 pruebas del lote en el núcleo, 1 en el servicio y 3 de la CLI;
  - **la CLI da los mismos bytes que el cwebp 1.6.0 oficial**, también con `-metadata all` y en una
    subcarpeta;
  - capturas de la sección.
- **Publicada la v0.4.0** cuando el cliente lo dijo. El borrador se revisó antes:
  - los 11 trabajos en verde;
  - las sumas cuadran;
  - las 14 entradas de `latest.json` tienen firmas válidas;
  - la CLI bajada de la Release dice 0.4.0.

  El endpoint del actualizador responde ya 0.4.0.
- La página de las propuestas de icono se guardó de recuerdo en
  `empaquetado/iconos/propuestas/propuestas-icono.html`, y se borró el artefacto publicado, como pidió
  el cliente.
- **No verificado**: Lotes en la ventana (diálogos, soltar, mostrar en la carpeta) y lotes grandes
  (deuda).

---

## 2026-10-07 · El actualizador (v0.3.3)

- El cliente pidió, antes de Lotes, **actualizar desde la propia aplicación como en Esfinge**, y
  esperar para publicar. Respondió a cuatro preguntas:
  - versiones normales, no pre-release;
  - el `.deb` se instala pidiendo la contraseña;
  - dos pasos, como Esfinge;
  - la clave, en GitHub y una copia en su bóveda.

  Todo queda en la [ADR 0018](adr/0018-actualizarse-sola.md).
- Hecho:
  - `tauri-plugin-updater` y `tauri-plugin-process`;
  - la puerta de 24 h y los ajustes guardados en `ajustes.json`. El servicio pasa a recibir la
    carpeta de configuración, y `apolo-dev` pasa de `--presets` a `--config`;
  - la banda (`Novedad.tsx`) y la sección Actualizaciones en Ajustes;
  - el caso traslocado de macOS;
  - en `publicar.yml`: la firma, `prerelease: false`, y un paso que repasa `latest.json`;
  - la clave minisign (contraseña hexadecimal) en los secretos de GitHub. La copia está en
    `~/.config/apolo/claves/`.
- Verificado aquí:
  - `make comprobar`;
  - 25 pruebas e2e, 3 de ellas nuevas con una novedad simulada (`?novedad=9.9.9`);
  - capturas de la banda;
  - licencias de las dependencias nuevas;
  - la clave, firmando un fichero.
- La prueba «pegar una orden» fallaba por tiempo con la máquina cargada: `-lossless -z 9` tarda 4 s
  en depuración ([trampas](trampas.md)). Ahora tiene su propio plazo.
- CI en verde, también `publicar.yml` lanzado a mano: la ventana compila con el plugin, y los siete
  paquetes del actualizador salen con su `.sig`. **Las siete firmas, verificadas** contra la clave
  pública.
- **Publicada** cuando el cliente dio el visto bueno, ya como versión normal. Al publicar:
  - los 11 trabajos en verde, y el paso de `latest.json` también;
  - las sumas del borrador cuadran;
  - las 14 entradas de `latest.json` apuntan a `releases/download/v0.3.3/…` y sus firmas son
    válidas;
  - `releases/latest/download/latest.json` responde 0.3.3.
- **No verificado**:
  - una actualización real (v0.3.4).

---

## 2026-10-07 · Textos tras probar la v0.3.2

- El cliente probó la v0.3.2 en su Mac: «está todo perfecto», incluida su foto HEIC del iPhone.
- Pidió dos textos:
  - «Distinto de cwebp» pasa a ser una frase corta según el caso: «Enderezada: cwebp no la gira» o
    «cwebp no abre HEIC» (con el formato de la imagen);
  - la nota de vista reducida pasa a decir «Vista reducida al XX %: clica aquí para ver al 100 % y
    apreciar más detalle».
- Saldada la deuda «HEIC sin probar con fotos reales» en macOS.

---

## 2026-10-07 · v0.3.2 publicada, tras un corte

- La sesión se cortó con la v0.3.2 a medio publicar: el trabajo de macOS había caído por un **HTTP
  502** de GitHub al volver a subir el `.dmg`. Con `gh run rerun --failed` pasaron los 11 trabajos.
  `publicar.yml` reintenta ya esa subida.
- Antes, la primera vez, macOS no enlazaba: `___cpu_indicator_init`. Era la detección de AVX2 de
  libde265, que en macOS pide compiler-rt. Se apagó AVX2 en macOS y se compila para macOS 11.
- Verificado: las sumas del `.dmg` y de la CLI cuadran con `SHA256SUMS.txt`, y la CLI de Linux
  bajada de la Release pasa el HEIC de ejemplo a WebP.

---

## 2026-10-06 (noche) · v0.3.2: los arreglos de la prueba y HEIC

- **Los siete arreglos de la prueba**:
  - cabecera con aire;
  - pesos con etiquetas, ahorro y barra;
  - **el rebote al hacer zoom**: el `onWheel` de React es pasivo, y ahora hay un oyente nativo no
    pasivo, más `gesturechange` para el pellizco de WebKit y `overscroll-behavior: none`;
  - aviso de vista reducida;
  - orden con rutas completas;
  - sección Presets;
  - «Distinto de cwebp» con el motivo.
- **Un riesgo que no salió en la prueba**: con un `.webp` de entrada, la orden y el nombre propuesto
  eran `foto.webp -o foto.webp`, que sobrescribían el original. Ahora sale `foto-apolo.webp`.
- **HEIC** (ADR 0017), pedido por el cliente:
  - `crates/heic`, con libde265 v1.1.3 y libheif v1.23.6 como submódulos, compiladas por su
    `build.rs` con CMake (instalado con pip en `~/.local/bin`), y enlaces a mano;
  - el EXIF se deja en orientación 1 porque libheif ya endereza;
  - el modo «embebido» de `libheif-sys` no sirve: no trae HEVC.
- Verificado:
  - la foto de ejemplo de libheif se decodifica y pasa a WebP con sus colores;
  - 20 pruebas de Playwright, con una de HEIC.
- **No verificado**: HEIC en las otras cinco plataformas (lo dirá CI) ni con una foto real de iPhone.

---

## 2026-10-06 (noche) · Fin de la prueba guiada, sobre la v0.3.1

- El cliente vio bien el icono del volumen y la ventana nueva.
- **Paso 4, abrir**: abre bien arrastrando y con «Abrir otra…»; la cabecera, el comparador, los
  pesos y la orden están bien.
- **Paso 5, comparador**: deslizador, zoom, desplazamiento, ajustar, 100 % y lado a lado bien.
  - Con calidad 5 «apenas» veía diferencia en la vista ajustada, y sí al acercarse: la vista
    reducida promedia los defectos.
  - **Fallo**: la ventana rebota al hacer zoom.
- **Paso 6**: copiar y pegar la orden; el preset se guarda; exportar pesa lo prometido. **El fichero
  exportado por la ventana es idéntico al de cwebp** con la misma orden, comprobado por el cliente
  con `cmp`.
- **Paso 7**: una foto de iPhone (HEIC convertida a JPEG con `sips`) sale tumbada con aviso, y al
  enderezarla queda derecha.
- **Siete arreglos** a la lista de la v0.3.2 (siguiente.md):
  - cabecera con más aire;
  - pesos con diseño;
  - el rebote al hacer zoom;
  - el aviso de vista reducida;
  - la orden con rutas completas;
  - una sección de Presets;
  - un texto mejor que «Con extras de Apolo».
- Pendiente: que el cliente diga si sube leer HEIC.

---

## 2026-10-06 (noche) · v0.3.1: la ventana de Esfinge, el disco y el instalador de Windows

- El cliente probó la v0.3.0 en su Mac:
  - la ventana del `.dmg` estaba bien;
  - el vidrio estaba bien, «aunque muy sutil»; se deja como el del sistema;
  - el Dock estaba bien.

  Pidió el icono del volumen como el de Esfinge, identidad en Windows y **la ventana como la de
  Esfinge**: «fíjate en cómo es la ventana de Esfinge para hacerla igual en Apolo».
- **Disco**: `disco.svg`, la carcasa de Esfinge con el sol. CI lo convierte en `.icns` y
  `icono-volumen.sh` lo mete en el `.dmg` de Tauri, que no deja elegirlo.
- **Windows**: NSIS en español, con `lateral.bmp` (164×314) y `cabecera.bmp` (150×57).
  `rasterizar.mjs` ahora escribe BMP de 24 bits.
- **Ventana**: la especificación de Esfinge, sacada de su CSS:
  - barra lateral de 225 px, con el hueco de los semáforos de 52 px;
  - sello de 26 px y título de 20;
  - filas de 38 px con iconos de línea de 18 (caja de 18, trazo de 1,4);
  - Ajustes abajo y la firma al pie;
  - cabecera de 52 px con el título a la izquierda;
  - botones de radio 9 con línea de 0,5 px, segmentado y campos de 30;
  - tarjetas de 14 px;
  - variantes de Windows y Linux.
- **Token nuevo**: `--barra-encima`. La prueba de contraste lo cazó un punto más oscuro de lo que
  aguantan los textos.
- Verificado: 16 pruebas de Playwright, capturas en claro y oscuro, BMP vistos en el navegador y
  `make comprobar`.
- **No verificado**: el disco en un Mac y el instalador en un Windows.

---

## 2026-10-06 (noche) · Prueba guiada, y la entrega 3: identidad

- **Prueba guiada con el cliente** en su Mac con Apple Silicon:
  1. La CLI de la v0.2.0 arranca y es universal.
  2. **Equivalencia con el cwebp 1.6.0 de Google para Mac ARM: 25 de 25**, con WebP, PNG y JPEG.
     Salda en parte la deuda de «solo Linux».
  3. El `.dmg` se instala. Hubo que pulsar «Abrir igualmente» en Privacidad y seguridad, que es el
     camino de macOS 15. La ventana se abre.
- Ahí el cliente pidió parar:
  - el `.dmg` pedía aceptar una licencia;
  - la ventana del `.dmg` debía tener identidad «como Esfinge»;
  - el icono, «más épico, más Apolo»;
  - la aplicación «apenas tiene diseño».

  Eligió identidad nativa con marca, ahora y antes de Lotes, y sin licencia en el `.dmg`
  (ADR 0015).
- **Entrega 3**:
  - **Icono**: cuatro propuestas en la familia de Esfinge (sol con laurel, perfil, lira, arco),
    con una segunda vuelta para el sol y el arco, en una página de comparación. Eligió **el sol
    con laurel**.
  - **El oro del sol** `#ffc83d` en el tema, con piedra encima, y una prueba de que el blanco no
    sirve.
  - **Vidrio en macOS** con `macOSPrivateApi` (ADR 0016) y la orden `plataforma`.
  - **Sello, firma, bienvenida y ficha** en «Acerca de».
  - **Fondo del `.dmg`** a 1x y 2x, unidos en TIFF por CI, y sin licencia.
  - `make iconos` y `make ventana-dmg`, con el Chromium de Playwright, como Esfinge.
- **Verificado**:
  - 16 pruebas de Playwright;
  - el contraste, con una prueba nueva;
  - capturas en claro y oscuro;
  - la ventana del `.dmg`, simulada.
- **Fallos del camino**:
  - la marca a trazo parecía un insecto;
  - el dorado se apagaba al pasar el ratón por la sección activa (especificidad CSS);
  - un `make comprobar | grep` dio por buena una puerta en rojo, que va a trampas.
- **No verificado**: el vidrio, el `.dmg` y el Dock en un Mac de verdad. Es el paso siguiente de la
  prueba guiada.

---

## 2026-10-06 (noche) · La v0.2.0, primera pre-release

- El cliente preguntó si se podían hacer ya las Releases. Se podía; se le explicó lo que faltaba
  para una 1.0 y pidió **la v0.2.0 con la CLI incluida**. ADR 0014.
- `publicar.yml`:
  - compila también la CLI de los seis objetivos (universal con `lipo` en macOS);
  - con la etiqueta, la cuelga en la Release junto a `SHA256SUMS.txt` y a las notas de
    `.github/notas/v0.2.0.md`;
  - las 0.x salen como pre-release en borrador.
- Se probó antes lanzándolo a mano, se etiquetó, se revisó el borrador y se publicó.
- La notas decían «funciona» y se corrigieron antes de publicar: la ventana no la ha abierto nadie.
- Verificado: la CLI bajada de la Release cuadra con su suma y da el mismo fichero que cwebp.
- **No verificado**: ningún instalador abierto.

---

## 2026-10-06 (tarde y noche) · Entrega 2: el Estudio

- El cliente eligió en tres preguntas, las tres como se recomendaban:
  - original contra resultado, no A contra B;
  - niveles en secciones plegables;
  - presets como ficheros JSON compartidos con la CLI.
- **Núcleo**:
  - `orientacion.rs`: enderezar, ADR 0012, que pidió el cliente al cerrar la mañana;
  - `vista.rs`: los píxeles RGBA para enseñar;
  - `presets.rs`: `…/Apolo/presets/*.json`;
  - `cwebp::leer_desde`: opciones encima de un preset;
  - el prefijo `-apolo_` para las opciones propias.
- **`crates/servicio`**: el Estudio sin ventana, con las imágenes abiertas, la vista previa con
  generaciones, exportar y los presets.
- **`crates/dev`** (`apolo-dev`): el servicio por HTTP.
- **`src-tauri`**: las mismas órdenes, el protocolo `apolo://` para los píxeles, el diálogo de
  abrir y guardar, y arrastrar y soltar. ADR 0013.
- **Interfaz**:
  - comparador en canvas, con deslizador, lado a lado, zoom con la rueda y arrastre;
  - panel pintado desde un esquema de 36 controles, con explicación de cada uno;
  - barra con el peso, el ahorro, el PSNR y la orden cwebp, para copiar y para pegar;
  - aviso de foto girada;
  - Ajustes con los presets guardados.
- **CLI**: `-apolo_preset <nombre>` y `apolo presets`.
- **Verificado**:
  - 14 pruebas de Playwright en claro y oscuro;
  - pruebas de servicio, orientación y presets;
  - `-apolo_preset` da el mismo fichero que las opciones escritas a mano;
  - capturas miradas en los dos temas.
- **Fallos que salieron por el camino**:
  - el contador de generación global (al recargar se cancelaba todo);
  - la tabla de la prueba de orientación, con la 6 y la 8 cambiadas;
  - `-apolo_enderezar` en la CLI no leía el EXIF;
  - el damero asomaba por el borde;
  - Playwright esperaba diez minutos por un 405.

  Cuatro trampas nuevas.
- CI en verde a la primera: `src-tauri` compila en Linux con las órdenes nuevas, Playwright pasa
  también en GitHub y los seis instaladores salen (ejecución 37494979883).
- **No verificado**: que la ventana funcione. Compilar no es abrirla: lo tiene que abrir el cliente.

---

## 2026-10-06 (tarde) · Entrega 1: el mismo fichero que cwebp

- Se leyó `examples/cwebp.c` y los lectores de `imageio/` de libwebp 1.6.0 enteros, y se portaron:
  el bucle de opciones (`cwebp::leer`, con `strtol` en base 0 y `-preset` que reescribe lo
  anterior), el `main()` desde la lectura hasta el fichero (`webp::codificar`), los metadatos
  (`metadatos::escribir`) y los lectores (`entrada/`).
- JPEG con mozjpeg (el decodificador de libjpeg-turbo). nasm compilado de fuente en `~/.local`, sin
  sudo.
- **`make equivalencia`**: descarga el cwebp 1.6.0 de Google, comprueba su sha256 y compara 29
  imágenes generadas × 35 combinaciones. Primera pasada: 908 iguales. Fallaban el ICC (libpng
  descarta perfiles inválidos) y el TIFF (libtiff premultiplica el alfa). Corregido: **1015 de
  1015 idénticos**. Corre también en CI (trabajo `equivalencia`).
- Pruebas sin cwebp: sin pérdida conserva los píxeles, redimensionar, cancelar y las opciones en
  JSON. Ida y vuelta preset ⇄ orden cwebp en 12 combinaciones.
- ADR 0011 (leer como cwebp). Cinco trampas nuevas y siete apuntes de deuda, casi todos «sin
  comprobar».
- CI en verde: la equivalencia también en GitHub (1015/1015), y `publicar.yml` compila con mozjpeg
  en las seis máquinas, Windows ARM64 incluido.
- **No verificado**: la equivalencia fuera de Linux x86-64.
- Al cerrar, el cliente pidió **enderezar según el EXIF como opción**: ADR 0012, apuntado en la
  entrega 2. Se retoma con la entrega 2 la tarde del mismo día.

---

## 2026-10-06 · Planteamiento y cimientos

- **Planteamiento con el cliente**, en cuatro rondas de preguntas. De ahí salen las diez primeras
  ADR: Tauri 2 + Rust, libwebp enlazada, WebP más el resto de Squoosh, Estudio y lotes, los extras
  (redimensionar, paleta, CLI, mapa de diferencias), **GPLv3 y repositorio público** (empezó como
  privado y a la venta; cambió a gratuito y público, y eso permitió usar libimagequant), aspecto del
  sistema, español con i18n, sin firmar por ahora y seis objetivos con ARM.
- Por qué no Go + Wails como Esfinge: los códecs son C/C++ y Rust; por cgo serían un suplicio y
  OxiPNG no existe en Go ([ADR 0001](adr/0001-tauri-y-rust.md)).
- **Entrega 0**: documentos vivos con el sistema de Esfinge; workspace de Cargo (`nucleo`, `tema`,
  `cli`, `src-tauri`); interfaz React con i18n y la carcasa (Estudio, Lotes, Ajustes); tokens
  generados desde Rust con la prueba de contraste; icono provisional; `comprobar.yml` y
  `publicar.yml`; repositorio `webcafeina/Apolo`.
- Verificado aquí: `make comprobar` en verde (fmt, clippy, pruebas de núcleo y tema, contraste,
  tokens al día, build y test de la interfaz). `apolo --version` dice **libwebp 1.6.0**. La primera
  paleta no pasaba el contraste en 14 parejas; se ajustó contra la peor superficie de cada modo.
- **No verificado aquí**: la ventana. El VPS no tiene `libwebkit2gtk-4.1-dev` y no hay sudo; se
  compila en CI ([deuda](deuda.md)).
- Rust instalado con rustup en `~/.cargo`, sin tocar el sistema.
- **CI**: `comprobar.yml` en verde a la primera, con la ventana. `publicar.yml` cayó en los cinco
  trabajos por `bundle.category` («Graphics» → «GraphicsAndDesign», a [trampas](trampas.md)), y a la
  segunda salió verde: DMG universal, NSIS x64 y ARM64, `.deb` y AppImage amd64 y arm64.

---

## Plantilla

```
## AAAA-MM-DD · <título>

- Qué se hizo o se decidió.
- Qué se verificó, y con qué.
- Qué queda abierto (→ mover a siguiente.md o deuda.md si procede).
```
