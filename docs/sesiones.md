# Sesiones

Bitácora: qué se hizo en cada sesión, la más reciente arriba. La plantilla está al final.

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
