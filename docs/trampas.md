# Trampas

Lo que costó encontrar y volvería a costar. Cada una con el síntoma tal como se ve —que es lo que se
busca cuando vuelve a pasar— y no con la causa, que es lo que no se sabe todavía.

Formato: `## Síntoma` · qué lo causa · cómo se evita · fecha.

---

## Una contraseña rompe la URL de conexión sin decir nada

`TypeError: Invalid URL`, o un fallo de autenticación que no cuadra con la contraseña del fichero. La
causa es una contraseña con `/`, `+`, `@`, `:` o `#` dentro de una URL. Se evita generándolas en
hexadecimal: `openssl rand -hex 24`. Es regla de toda la máquina (`~/.claude/CLAUDE.md`); Apolo no
tiene servidor, pero el día que tenga la actualización o una web, aplica. · 2026-10-06

## «failed to build bundler settings: invalid category»

Sale **al final** de `tauri build`, después de compilar todo, así que cuesta los minutos de la
compilación entera en cada objetivo. `bundle.category` en `tauri.conf.json` no admite cualquier
texto: es una lista cerrada con los nombres de Apple sin espacios (`GraphicsAndDesign`, no
`Graphics`). Pasó en la primera publicación, en los seis objetivos a la vez. · 2026-10-06

## cwebp descarta un perfil ICC de un PNG y Apolo lo copia

El fichero de cwebp sale más corto, justo el tamaño del perfil. No lo descarta cwebp: lo descarta
**libpng**, que comprueba la cabecera del perfil (longitud, `acsp`, espacio de color, clase, PCS,
tabla de etiquetas) y, si no le gusta, lo tira con un aviso «benigno» que nadie ve. `png::icc_valido`
repite esas comprobaciones. Salió con un perfil falso del corpus de pruebas. · 2026-10-06

## Un TIFF con transparencia da otro fichero que cwebp

Más pequeño en cwebp, en todas las opciones. libtiff, al leer con `TIFFReadRGBAImage`,
**premultiplica el alfa no asociado**, y cwebp solo deshace la premultiplicación si el alfa ya venía
asociado: codifica el color oscurecido. Se repite con la tabla `UaToAa` de libtiff
(`(v·a + 127) / 255`). Ver la ADR 0011. · 2026-10-06

## El crate mozjpeg avisa de los errores con un pánico

Un JPEG dañado no devuelve `Err`: hace `resume_unwind`. El lector lo envuelve en `catch_unwind`, y
eso **solo funciona con `panic = "unwind"`**. Si algún día se pone `panic = "abort"` en el perfil
de release para ganar tamaño, un JPEG roto cerrará la aplicación entera. · 2026-10-06

## libwebp-sys inicializa `WebPConfig` con la versión del decodificador

Sus ayudas `WebPConfig::new()` y `WebPInitConfig` pasan `WEBP_DECODER_ABI_VERSION` a una función
del **codificador**. Hoy da igual porque las dos valen 0x0210, pero el día que difieran, fallará sin
explicación. Apolo llama a `WebPConfigInitInternal` y a `WebPPictureInitInternal` directamente con
`WEBP_ENCODER_ABI_VERSION`. · 2026-10-06

## clippy pide `as_chunks` en vez de `chunks_exact`

Con Rust 1.99, `clippy::chunks_exact_to_as_chunks` falla la puerta con `-D warnings` en cada
`chunks_exact(4)` de tamaño fijo. `as_chunks::<4>().0` da arrays (`&[u8; 4]`) y es lo que se usa en
todo el núcleo. · 2026-10-06

## La vista previa se queda en «…» después de recargar

Las peticiones llegan y se cancelan todas. El contador de generación del servicio era **global**: al
recargar, la interfaz vuelve a contar desde 1, y para el servicio, que ya iba por la 40, todas las
peticiones eran viejas. Ahora va por imagen abierta. Lo encontró Playwright, que recarga en cada
prueba. · 2026-10-06

## Playwright espera a `apolo-dev` hasta agotar el tiempo, y el servidor sí estaba

`webServer.url` comprueba con **GET**, y las rutas de `/api` son solo POST: un 405 no le vale como
«listo» y espera los diez minutos enteros. Para eso está `GET /salud`. · 2026-10-06

## `pkill -f <patrón>` se mata a sí mismo

Si el patrón aparece en la propia orden de la shell (`pkill -f target/debug/apolo-dev` dentro de un
`bash -c` más largo), mata también la shell que lo lanza: código 144 y nada más. Usar
`pgrep -f "debug/apolo-[d]ev" | xargs -r kill`: los corchetes hacen que el patrón no se encuentre a
sí mismo. · 2026-10-06

## Otros proyectos de esta máquina usan los puertos 5173 y 5273

Cronos y Esfinge levantan Vite y Playwright en esta misma máquina, a veces a la vez. Las pruebas de
Apolo usan puertos propios: 5191 para Vite y 34591 para `apolo-dev`. `make dev-web` usa 5173 y 34500,
que son los de por defecto. · 2026-10-06

## `make comprobar | grep …` dice «salida 0» con la puerta en rojo

El código de salida de una tubería es el del **último** comando, el `grep`, no el de `make`. Un
`cargo fmt --check` fallido quedó escondido así, y solo se vio al repetirlo sin tubería. Para saber
si la puerta pasa: `make comprobar > fichero.log 2>&1; echo $?`, y mirar el log aparte. · 2026-10-06

## Con `libheif-sys` y `embedded-libheif`, ninguna foto de iPhone se abre

Compila y enlaza bien, pero al decodificar dice que no hay decodificador para HEVC. Su modo
«embebido» compila libheif con `WITH_LIBDE265=ON`, pero **no compila libde265**: la busca instalada
en el sistema, y si no está, la deja fuera sin avisar. Por eso `crates/heic` compila las dos con su
propio `build.rs` y le da a libheif la ruta de libde265 a mano (`LIBDE265_INCLUDE_DIR`,
`LIBDE265_LIBRARY`). · 2026-10-06

## Una foto HEIC enderezada sale girada otra vez

libheif aplica los giros del propio HEIF al decodificar, y el EXIF del iPhone dice además
«orientación 6». Si se respeta el EXIF, la foto se gira dos veces. Al leer un HEIC, el EXIF se deja en
1 (`entrada/heic.rs`). · 2026-10-06

## En Windows, enlazar libheif estática pide la DLL de libde265

Las cabeceras de libde265 declaran sus funciones como de DLL salvo que se defina
`LIBDE265_STATIC_BUILD`. Va en `cflag` y `cxxflag` al compilar libheif (`crates/heic/build.rs`).
Apuntado antes de verlo fallar: es lo que dice la propia CMakeLists de libde265. · 2026-10-06

## En macOS x86-64: «Undefined symbols: ___cpu_indicator_init»

Al enlazar Apolo (aplicación y CLI) para Intel, con libde265 dentro. libde265 detecta AVX2 con
`__builtin_cpu_supports`, y en macOS eso vive en la biblioteca de soporte de clang (compiler-rt),
que rustc no enlaza. En Linux lo da libgcc y en Windows no se usa. Se apaga `ENABLE_AVX2` en macOS
(`crates/heic/build.rs`): quedan las versiones SSE. · 2026-10-06

## «object file was built for newer macOS version than being linked»

Cientos de avisos al enlazar en el Mac de CI: CMake compila para la versión del Mac que compila, no
para el mínimo de Apolo. Son avisos, pero dicen que la aplicación podría usar algo que macOS 11 no
tiene. `crates/heic/build.rs` pasa `CMAKE_OSX_DEPLOYMENT_TARGET` (11.0, o `MACOSX_DEPLOYMENT_TARGET`
si está puesto). Los mismos avisos salen de mozjpeg, sin plataforma, y son inofensivos. · 2026-10-06

## «pegar una orden cwebp» falla en Playwright, y a mano funciona

La orden se queda en `cwebp foto.webp -o foto-apolo.webp`, sin los ajustes pegados. No se pierden:
la orden de la pantalla sale del resultado codificado, y `-lossless -z 9` tarda **4 s** con la
compilación de depuración de `apolo-dev` (1,3 s en release). Con la máquina cargada por otros
proyectos, se pasa de los 5 s de espera de Playwright. Esa espera lleva su propio plazo de 30 s.
· 2026-10-07

## «los niveles se pliegan» falla en CI de vez en cuando

Tras recargar, el nivel experto sale cerrado. El `open` de `<details>` cambia con el clic, pero lo
que se guarda en `localStorage` llega después: con el evento `toggle`, que el navegador manda en otra
vuelta del bucle, y un efecto de React. La prueba recargaba antes. Ahora espera a verlo guardado.
· 2026-10-07

## La barra de la descarga va a tirones y por detrás del porcentaje

El actualizador avisa **por cada trozo** que llega: con 11 MB son cientos de avisos por segundo. Cada
uno redibujaba la banda y reiniciaba la transición CSS de la anchura, que con `ease` vuelve a
arrancar despacio cada vez: la barra no llegaba nunca adonde iba y se quedaba hasta 41 puntos por
detrás. Lo vio el cliente al actualizar a la v0.4.0. En el navegador no se notaba porque la
descarga simulada eran diez pasos tranquilos. Ahora se redibuja solo cuando cambia el porcentaje
entero, la barra no tiene transición, y la simulación imita la real (trozos de 16 KB a ráfagas). Una
prueba compara la barra pintada con el número. · 2026-10-07

## «cmake_minimum_required … Compatibility with CMake < 3.5 has been removed»

Al compilar el `cjpeg` oficial de MozJPEG 4.1.5 con CMake 4: su CMakeLists pide una versión tan
vieja que CMake 4 se niega a configurar. Se pasa `-DCMAKE_POLICY_VERSION_MINIMUM=3.5` (lo dice el
propio mensaje). Está en el objetivo `cjpeg-oficial` del Makefile. · 2026-10-07

## «sorry, arithmetic coding not supported» en el cjpeg oficial

MozJPEG 4.1.5 viene con la codificación aritmética **apagada** (`WITH_ARITH_ENC=OFF`, igual que
la decodificación). Apolo la había encendido en `mozjpeg-sys` pensando que el oficial la traía, y
`-arithmetic` daba un fichero donde cjpeg da error. Ahora Apolo la rechaza igual. · 2026-10-07

## El JPEG de un PNG con `sRGB` sale 4 bytes más corto que el de cjpeg

En todas las opciones, y solo con ese PNG. cjpeg incrusta su perfil sRGB mínimo (`tiny_srgb`, 536
bytes) cuando el PNG trae el trozo `sRGB`; copiado a mano se perdieron cuatro ceros. Ahora el array
sale del fuente con un script, como las tablas de cuantización. Lo cazó la prueba de equivalencia.
· 2026-10-07

## Una prueba a mano dice «IGUAL» y no ha comparado nada

En zsh, `$o` con `o="-quality 80"` **no se parte** en palabras: los dos programas reciben un solo
argumento raro, los dos fallan, y `cmp` compara los ficheros de la vuelta anterior. Hay que escribir
`${=o}`, y borrar las salidas antes de cada caso para que un fallo no pase por igualdad.
· 2026-10-07

## El QOI de Apolo es válido pero no es el de `qoiconv`

Mismo tamaño, misma cabecera, y difiere a los pocos cientos de bytes. El crate `qoi` apunta cada
color en la tabla de índices aunque ya estuviera; `qoi.h` solo cuando no lo encuentra, y nunca en
una racha. Los dos ficheros son QOI correctos, pero no los mismos. Apolo traslada `qoi_encode` tal
cual. Al trasladarlo, `vr > -3 && vr < 2` se escribió como `(-3..2)`, que en Rust **incluye** el
−3: el fichero salió más pequeño que el de la referencia, que también es un error. · 2026-10-07

## El PNG de Apolo y el de `oxipng` difieren con las mismas opciones

No ha pasado, pero pasaría: Cargo eligió libdeflater **1.26.1** y el `oxipng` 10.2.1 publicado
trae la **1.26.0** en su `Cargo.lock`. Otra versión de libdeflate puede comprimir distinto. Van
fijadas con `cargo update --precise`, y la referencia se instala con `cargo install --locked`.
· 2026-10-07

## `git push`: «remote: Internal Server Error» varias veces seguidas

Con un commit normal y githubstatus.com diciendo que todo va bien. Tres reintentos seguidos
fallaron; esperando un minuto, pasó a la primera. Si vuelve a pasar, se reintenta cada minuto en vez
de seguido. · 2026-10-07

## «HTTP 502: Server Error» al subir el `.dmg` a la Release

Pasó con la v0.3.2: todo compilado, el icono del volumen puesto, y al reemplazar el `.dmg` en la
Release la API de GitHub dio un 502 y el trabajo de macOS cayó; el de adjuntar ya no corrió, y el
borrador se quedó a medias. Es pasajero: `gh run rerun <id> --failed` lo arregla, y desde entonces
`publicar.yml` reintenta la subida cuatro veces. · 2026-10-07

## cjxl compilado con gcc da otro fichero que el oficial

Mismas fuentes, mismas opciones, y el `.jxl` difiere en unos pocos bytes. libjxl calcula en coma
flotante, y gcc y clang no redondean igual: ni con `-ffp-contract=off` en gcc sale lo mismo. El cjxl
oficial sale de clang 18 (lo dice `cjxl --version`: `{Clang 18.1.3}`), y con clang 18 sale idéntico.
Por eso `crates/avifjxl/build.rs` no compila sin clang en Linux. avifenc da lo mismo con los dos.
· 2026-10-07

## El clang 18 del VPS: «libtinfo.so.5: cannot open shared object file»

No hay sudo, así que LLVM 18 está descomprimido en `~/.local/llvm18` desde el paquete oficial, que
se enlazó contra la libtinfo 5 de Ubuntu antiguo. Ubuntu actual solo trae la 6. Hay una de pega en
`~/.local/llvm18/compat/libtinfo.so.5`, compilada de `tinfo5.c` con el script de versiones
`NCURSES_TINFO_5.0.19991023`, que reexporta la 6. El Makefile pone `APOLO_CLANG` y
`LD_LIBRARY_PATH`; con `cargo` a mano, hay que ponerlos (ver el Makefile). · 2026-10-07

## «FETCHCONTENT_FULLY_DISCONNECTED … the source directory for dependency libaom»

libavif trae sus dependencias con FetchContent, y el nombre con el que las declara no es el de la
biblioteca: aom es **libaom**, así que la carpeta se le da con `FETCHCONTENT_SOURCE_DIR_LIBAOM`, no
`_AOM`. Con el nombre mal, CMake no la encuentra, y con `FULLY_DISCONNECTED` no la descarga: el
error habla de `_deps/libaom-src`. · 2026-10-07

## Un submódulo apuntado a la punta de su rama, sin querer

`git submodule add` deja el submódulo en la punta de la rama por defecto. Si después se hace
`git checkout <etiqueta>` dentro, el repositorio de fuera no se entera hasta que se vuelve a añadir:
`git status` lo enseña como `M` (o `AM`) y el commit guarda la punta, no la etiqueta. Pasó con los
diez de `crates/avifjxl/vendor` en el primer commit de la 5b; el segundo los dejó bien. Antes de
hacer commit, `git submodule status` tiene que enseñar el commit fijado. · 2026-10-07

## Linux ARM64: «undefined reference to `avifQueryCPUCount'» al enlazar

En x86-64 enlazaba y en ARM64 no, con las mismas bibliotecas. `build.rs` pedía la del puente con
`static:+whole-archive`, y rustc saca esas del paquete del crate y las pone **detrás** de las demás;
el enlazador de GNU lee en una pasada, y lo que el puente necesitaba ya había pasado. Sin
`+whole-archive` no hace falta nada: Rust llama al puente, y eso lo enlaza. · 2026-10-07

## Windows: «pnglibconf.c: fatal error: 'zlib.h' file not found»

Al compilar libpng dentro de libjxl. En Windows, CMake encuentra el awk de Git, y con awk libpng
genera su `pnglibconf.h` compilando un fichero que no recibe la carpeta de zlib. Sin awk usa el
`pnglibconf.h.prebuilt`, que es la configuración por defecto. El proyecto de CMake pone
`AWK=OFF` en Windows: `OFF` y no `NOTFOUND`, porque con `NOTFOUND` `find_program` vuelve a
buscar. · 2026-10-07
