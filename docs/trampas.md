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

## «La configuración en paralelo no es correcta» al abrir Apolo en Windows

Sale al abrir `apolo-app.exe`; la CLI y `apolo-mcp.exe`, con el mismo manifiesto, van bien. El
manifiesto que Tauri mete en la ventana estaba roto. `app_manifest` no lo mete como fichero: lo
copia línea a línea en un `.rc`, con un espacio delante de cada una, y `rc.exe` lee el `.rc` en
cp1252. Rompía dos cosas:
- `<?xml …?>` dejaba de estar al principio;
- las tildes y las comillas «» del comentario se convertían en bytes que no son UTF-8.

Windows no crea el proceso. Se evita con un manifiesto sin declaración XML y solo en ASCII, que
`crates/cli/build.rs` comprueba en cada compilación. `humo.py` abre ahora la ventana: antes la
instalaba pero no la abría. Para ver el manifiesto de un `.exe`, `7zz x app.exe '.rsrc/MANIFEST/*'`
(el 7-Zip de github.com/ip7z/7zip, y antes `7zz x` del instalador). · 2026-10-20

## «No se encontró MSVCP140.dll» al abrir Apolo en Windows

Sale en un Windows recién instalado, y **nunca en las máquinas de CI**, que traen instalado
«Visual C++ Redistributable»: el humo pasaba en verde. Lo causan los motores en C++ (libjxl, aom,
libheif, libde265) compilados con /MD, la biblioteca de C++ en DLL, y Rust también enlaza
VCRUNTIME140.dll por defecto. Pasaba en todos los `.exe` de Windows desde la primera versión. Se
evita con todo en /MT:
- `+crt-static` en `.cargo/config.toml`, que también lo pasa al crate cc;
- `CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded` en los CMake de `heic` y `avifjxl`.

`humo.py` mira las DLL que importa cada `.exe` y falla si sale alguna de Visual C++. · 2026-10-19

## El icono de Apolo, borroso en el listado de aplicaciones de Ubuntu

En el Dock de Ubuntu se veía bien y en la cuadrícula de aplicaciones, borroso. El `.deb` solo
llevaba 32 y 128 píxeles en `hicolor`. El de 256 lo copiaba Tauri a `hicolor/256x256@2`, porque
se llamaba `128x128@2x.png`: Tauri lo toma como uno de 256 a doble escala, 512 píxeles reales. La
cuadrícula, a doble escala, quiere unos 192 píxeles y estiraba el de 128. Se evita listando en
`bundle.icon` PNG sin `@2x` en el nombre (`256x256.png` y `icon.png`, de 512). · 2026-10-19

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

## Windows ARM64: «unresolved external symbol CopyRow_NEON»

libyuv, al enlazar. Su CMake solo compila los ficheros NEON `if(NOT MSVC)`, y clang-cl cuenta como
MSVC; sus cabeceras, en cambio, encienden los caminos NEON al ver `__aarch64__`. El proyecto de
CMake define `LIBYUV_DISABLE_NEON`, `_SVE` y `_SME` en Windows ARM64, y libyuv usa su C. · 2026-10-07

## CI: «this method takes 5 arguments but 4 arguments were supplied» en `src-tauri`

El VPS no compila `src-tauri` (no hay webkit), así que un cambio ahí no lo ve ni `make comprobar`.
Pasó al hacer commit nombrando carpetas a mano (`git add crates frontend …`): los cambios de
`src-tauri/src/lib.rs` se quedaron fuera, y CI falló en la ventana mientras la CLI compilaba. Antes de
cada commit, `git status --short` no tiene que enseñar nada sin añadir. · 2026-10-08

## Un MCP por stdio que se rompe «a veces»: «Unexpected token» o el cliente se desconecta

El cliente lee la salida estándar del servidor como mensajes JSON-RPC, uno por línea, y **cualquier
otra cosa** que llegue por ahí lo rompe: un `printf` de avifenc, un `println!` de depuración. Pasa
solo con algunas operaciones, así que parece aleatorio. `apolo-mcp` llama a
`apolo_avifjxl::apartar_salida()` **antes de nada**: copia el fd 1 para el protocolo y apunta el fd 1
a stderr. Tiene que ir en C (`puente.c`): en Windows, las bibliotecas de C escriben por la tabla de
descriptores del CRT, y un `SetStdHandle` desde Rust no la cambia. `crates/avifjxl/tests/salida.rs`
lo vigila (ADR 0023). · 2026-10-14

## El fd 1 se queda en `/dev/null` tras convertir AVIF y JXL a la vez

Callar un `main` de C es apuntar el fd 1 a `/dev/null` y devolverlo al acabar. Si dos llamadas con
candados distintos (AVIF y JXL) lo hacen a la vez, la segunda guarda como «original» el `/dev/null`
de la primera y lo restaura al final: el fd 1 se queda en `/dev/null` para siempre. En la ventana no
se ve; en `apolo lote --formato avif,jxl` la CLI se queda muda, y en un MCP sería el protocolo. Solo
se callan avifenc y avifdec, que comparten candado; cjxl y djxl van con `--quiet`. · 2026-10-14

## Un nombre de herramienta MCP con ñ o tildes

Los nombres de herramienta solo admiten `[a-zA-Z0-9_-]`, y algunos clientes ni avisan: la
herramienta no aparece. Por eso `ver_imagen`, `tamano` y nada de `año`. Tampoco hace falta prefijo:
el cliente ya las llama `mcp__apolo__…`. La prueba `tests/tuberia.rs` lo comprueba. · 2026-10-14

## `make catalogo-mcp` y la prueba del catálogo

`docs/mcp/catalogo.md` sale de las descripciones del código (los comentarios `///` de cada
herramienta y de cada campo). Cambiar una descripción sin regenerarlo hace fallar `make comprobar`
con «docs/mcp/catalogo.md no está al día». Se arregla con `make catalogo-mcp`. · 2026-10-14

## El agente no ve los avisos de una herramienta MCP, aunque están en la respuesta

Una herramienta que devuelve texto y `structuredContent` a la vez: Claude Code le pasa al modelo
**solo el `structuredContent`**, en JSON, y el texto preparado para él (avisos, pistas, la orden
equivalente) no le llega. No da ningún error: el agente simplemente no sabe lo que dice el texto, y
propone hacer lo que ya se hizo. En `apolo-mcp`, el texto va también como `resumen` dentro de
`structuredContent` (`lib.rs::resultado`), y `tests/tuberia.rs` lo comprueba. Cronos responde
igual, con texto y datos, y probablemente le pase lo mismo. Salió en el paseo
([mcp/paseo.md](mcp/paseo.md)). · 2026-10-14

## `src-tauri` deja de compilar si `externalBin` está en `tauri.conf.json` y falta el binario

(Sabido de la documentación de Tauri, no visto aquí: se evitó de entrada.) Con `externalBin` en `tauri.conf.json`, el `build.rs` de Tauri exige el binario en
`src-tauri/binaries/apolo-mcp-<triple>` **en cada compilación**, también en `cargo clippy`, en
`tauri dev` y en CI de comprobar, donde no hace falta. Por eso el sidecar va en
`src-tauri/tauri.mcp.conf.json`, que solo se fusiona al empaquetar (`--config`, en `make app` y en
`publicar.yml`), y `make sidecar` lo pone antes. El nombre lleva el triple de destino; en macOS
universal se ponen el universal y los dos de cada arquitectura (ADR 0023). · 2026-10-14

## Crear el `.mcpb` desde `apolo-dev` tarda medio minuto

En depuración, `apolo-mcp` pesa unos 200 MB, y comprimirlo con zlib-rs sin optimizar tardaba 43 s:
más que una prueba e2e. El perfil `dev` optimiza `zlib-rs`, `flate2`, `crc32fast` y `zip`, y el
paquete se comprime con nivel 1: 0,9 s. Ojo, el motor de deflate de `zip` 4 es **zlib-rs**, no
miniz_oxide: optimizar el que no es no cambia nada. · 2026-10-14

## macOS 15 en CI: «Undefined symbols: _DGifOpen, _GifErrorString…» al enlazar

Al pasar el CI de `macos-14` a `macos-15`, la CLI y `apolo-mcp` dejaron de enlazar. La máquina nueva
trae **giflib con Homebrew**, el CMake de libjxl la encuentra (`find_package(GIF)`) y activa la
lectura de GIF en cjxl, pero nadie la enlaza. En `macos-14` no estaba, en el VPS tampoco. Se apaga en
`crates/avifjxl/cmake/CMakeLists.txt` con `CMAKE_DISABLE_FIND_PACKAGE_GIF`, para que todas las
máquinas compilen lo mismo. Al cambiar de imagen de CI, mirar qué trae Homebrew. · 2026-10-16

## Claude Desktop: «invalid type: string "{\"x\":0,…}", expected struct RecorteMcp»

Un argumento que es un objeto (`recorte`, la `zona` de `ver_imagen`) y cuyo esquema es una
referencia a `$defs` (`anyOf: [{$ref}, {type: null}]`, lo que genera schemars para un
`Option<Struct>`) llega desde Claude Desktop **como texto con el JSON dentro**, y serde lo rechaza.
Claude Code lo manda bien, así que la evaluación no lo vio. Arreglo en `entrada.rs`: el esquema
escrito en el sitio (`esquema_rectangulo`, `type: object`, sin `$ref`) y una lectura que acepta
también el texto (`objeto_o_texto`). `tests/objetos.rs` lo vigila. Para cualquier objeto nuevo en
las entradas de las herramientas, lo mismo. · 2026-10-19

## Windows: «Failed to open file for write» de avifenc, o «Could not write jxl file» de cjxl

Solo cuando la ruta tiene tildes o eñes, por ejemplo una carpeta de usuario `C:\Users\José`, o la
temporal dentro de ella. WebP funciona y AVIF y JPEG XL no: avifenc y cjxl, compilados dentro,
abren sus ficheros con `fopen`, que en Windows interpreta el `char*` en la página de códigos del
proceso (cp1252), y Rust les pasa UTF-8. No sale en ningún sitio más que en Windows con esa ruta: lo
destapó `humo.yml` el 2026-10-19. Arreglo: `activeCodePage` UTF-8 en el manifiesto de los tres
`.exe` (`empaquetado/windows/apolo.manifest`), con `tauri-build` en la aplicación y `embed-resource`
en `apolo` y `apolo-mcp`. Windows 10 1903 o posterior. · 2026-10-19

## `publicar.yml` en macOS: «failed to run bundle_dmg.sh», sin más

Pasó una vez en `macos-15` y a la segunda salió bien: `hdiutil` falla a veces en las máquinas de
GitHub. `gh run rerun <id> --failed` (relanza también lo que depende de él, el humo). Si se repite,
reintentar ese paso. · 2026-10-19

## Un lote de fotos grandes gasta en Linux casi el doble que su cuenta

Con 8 hilos y fotos de 24 MP a AVIF, el pico crecía con el número de fotos: 2,3 GB con 16 y 2,9 GB
con 200, aunque nunca hubiera más de ocho a la vez, y limitar cuántas iban a la vez no lo cambiaba.
Es glibc: cada hilo tiene su reserva (arena) y lo liberado en ella no vuelve al sistema. Con
`MALLOC_ARENA_MAX=2`, 1,5 GB y más rápido. `apolo_nucleo::memoria::preparar()` lo pone con
`mallopt` al arrancar cada programa. macOS y Windows devuelven los bloques grandes al liberarlos. Se
ve con `scripts/banco.sh` o con `/usr/bin/time -f %M`. · 2026-10-20

## Un `pkill -f` que se mata a sí mismo

La orden termina con el código 144 y no hace lo que venía detrás. `pkill -f patrón` busca en la
línea de órdenes completa de cada proceso, y la del propio shell que lo ejecuta contiene el patrón:
se mata a sí mismo. Pasó tres veces (2026-10-20 y 21). Se evita buscando el PID con un patrón
anclado (`pgrep -f "^python3 -m http.server 8765"`) y matando ese PID, o guardando el PID al
lanzar el proceso. · 2026-10-21

## GitHub Pages con dominio propio: el certificado HTTPS no llega nunca

Con el dominio puesto y el CNAME bien en Cloudflare, Pages decía que el certificado estaba en
camino durante horas y «Enforce HTTPS» seguía sin poder marcarse. Se arregló quitando el dominio
personalizado en los ajustes de Pages y volviéndolo a poner: a los pocos minutos lo emitió. Después,
«Enforce HTTPS» y la web responde 200. · 2026-10-21

## Windows sin marco: el menú de Snap no sale aunque se conteste `HTMAXBUTTON`

En una ventana de Tauri sin marco, contestar `HTMAXBUTTON` a `WM_NCHITTEST` en la ventana principal
no sirve: el ratón sobre la barra de título está encima de la ventana hija de WebView2, y Windows le
pregunta a ella. Hace falta otra ventana hija sobre el botón de maximizar que conteste eso
(`src-tauri/src/ventana_windows.rs`, ADR 0027), con `WS_CLIPSIBLINGS` (sin él, WebView2 le gana) y
sin `WS_EX_LAYERED` (con él, pierde el ratón); invisible porque no pinta (pincel nulo). Esa ventana
se queda con el ratón: el `:hover` y el `onclick` del botón HTML de debajo ya no llegan, y por eso
avisa con el evento `ventana:maximizar` y maximiza ella. Si se cambian las medidas de la barra en
el CSS, hay que cambiarlas también en `ALTO_BARRA` y `ANCHO_BOTON`. · 2026-10-21

## «EDQUOT» al escribir en la carpeta temporal de Claude Code

`/tmp` es un tmpfs con cuota: comprobar `ventana_windows.rs` para Windows con `cargo check
--target x86_64-pc-windows-msvc` dejó más de un giga en el scratchpad, y con las descargas de
versiones viejas se llenó. Fallaron escribir ficheros y hasta la salida de las órdenes (`make
comprobar` dio error sin decir por qué). Se arregla borrando lo grande del scratchpad
(`du -sh .../scratchpad/*`); las compilaciones de prueba, mejor fuera de `/tmp`. · 2026-10-21

## El conector de Figma: «you don't have edit access» y el límite de consultas

El conector de Figma exige **permiso de edición** sobre el archivo aunque solo vaya a leer, y la
cuenta del conector es la de quien lo conectó (aquí `nacho@webcafeina.com`), no la que tenga abierta
el cliente. Un archivo duplicado en otra cuenta da «Looks like you don't have edit access» aunque se
vea en el navegador. Con un plan gratuito no se puede dar edición a otro: hay que duplicar el kit en
la misma cuenta del conector (se hizo pasándolo a `info@` y compartiéndolo). Además, con un asiento
de solo lectura («View») Figma limita mucho las consultas: con unas 20 se cortó. Mejor
`get_metadata` de cada página, que trae nombres y tamaños de todo, que muchas consultas pequeñas; y
encargarlo a un agente que escriba las medidas en un fichero. · 2026-10-21

## WebKit no imita la refracción de Liquid Glass

Las demos de «liquid glass» en CSS usan `backdrop-filter: url(#filtro)` con un mapa de
desplazamiento SVG. En WebKit (Safari, y por tanto la ventana de Apolo en macOS) no funciona (bug
245510): se ve un desenfoque sin más. La refracción solo la da el vidrio nativo
(`NSGlassEffectView`, `ventana_macos.rs`); en la interfaz, brillo, sombras y `blur()`. · 2026-10-21

## CI: «RPC failed; HTTP 503» al clonar aom o libyuv

Varios trabajos de `publicar.yml` fallan en el paso de los submódulos, todos en el mismo minuto, con
«The requested URL returned error: 503» de `aomedia.googlesource.com/aom` o
`chromium.googlesource.com/libyuv/libyuv`. No es Apolo: los servidores git de Google se caen a ratos.
Se comprueba con `curl -s -o /dev/null -w "%{http_code}"
'https://aomedia.googlesource.com/aom/info/refs?service=git-upload-pack'` (200 cuando ya responde) y
se arregla con `gh run rerun <id> --failed`. Pasó con la v0.14.0. · 2026-10-21

## `-apple-visual-effect` se descarta si el CSS llega antes que la preferencia

La propiedad privada de WebKit para el vidrio del sistema (ADR 0028) solo la reconoce el analizador
de CSS si la preferencia `_setUseSystemAppearance` ya está encendida. Si la hoja se lee antes, la
declaración se tira sin avisar y `@supports` dice que no. Por eso `vidrio-macos.css` se importa con
`?inline` y main.tsx la inyecta con un `<style>` cuando `plataforma` dice `vidrio_controles`, y no
con las demás hojas. · 2026-10-21

## El botón de un interruptor hecho con un degradado sale como una elipse

En WebKit un `<input>` no pinta `::before`, así que el botón del interruptor va en el fondo. Con
`radial-gradient(ellipse …)` sale una elipse, no una píldora, y en Safari se veía enorme (v0.14.0 en
el Mac del cliente). Un SVG de fondo con un `rect` de radio la mitad del alto da la píldora, con su
sombra (`feDropShadow`). · 2026-10-21

## Iconos de Yaru que salen descolocados al juntar sus trazados

Los SVG simbólicos de Yaru traen a veces varios `<path>`, cada uno empezando por una «m»
minúscula. En un trazado, la primera «m» cuenta como absoluta; al juntarlos en un solo `d`, las de
después pasan a ser relativas y se descolocan (el icono del Estudio salía como un punto). Hay que
cambiar a «M» el comienzo de cada uno al juntarlos. Además, algunos llevan una traslación en un
`<g>` o una caja que no es de 16 (`viewBox="0 0 4.2333 4.2333"`): `iconos-yaru.ts` las guarda. ·
2026-10-22

**Y cambiar la «m» por «M» no basta.** Las parejas de números que siguen a una «m» son líneas
*relativas*; detrás de una «M» pasan a ser *absolutas*. `m318 165.5-3 5.5` se convertía en
`M318 165.5-3 5.5`, una línea al punto (-3, 5.5) fuera del icono. Así, desde la v0.15.0, en Ubuntu
faltaba la montaña del icono del Estudio, el pie del monitor de «Escritorio» y los meridianos del
globo de «Idioma» (visto el 2026-10-23 al hacer los de Breeze, que tienen el mismo caso). La función
`absoluto()` de los generadores pone una «l» tras los dos números de la «M» si siguen más. Los
generadores están en `~/.cache/apolo-revision/{yaru,breeze}.py` y las fuentes, en `~/.cache/kde-ref/`
(Yaru en el commit `7f18973`). · 2026-10-23

## Ubuntu: el Centro de aplicaciones enseña un .deb suelto sin icono, sin editor y con «?» en las tildes

Al abrir el `.deb` con doble clic, el Centro de aplicaciones de Ubuntu (26.10) dice «Editor
desconocido», licencia «unknown», pone un icono genérico y cambia las tildes por «?»
(«im?genes»). No es el paquete: para un `.deb` suelto solo pide a PackageKit `GetDetailsLocal`
(nombre del paquete, resumen, descripción, web, tamaño), deja fijos el editor y el icono, y
PackageKit pone la licencia a «unknown» y lee la descripción sin UTF-8 (apt la pasa a ASCII con
`UTF8ToCodeset`, que cambia lo que no sabe por «?»). Código de `ubuntu/app-center`
(`local_deb_model.dart`, `app_title.dart`) e issues #120 y #1679. Lo único que se arregla desde el
paquete son las tildes: la descripción del `control` en español sin palabras con tilde (ASCII,
que sobrevive; lo eligió el cliente frente al inglés), en `tauri.linux.conf.json`; el lanzador
lleva la frase con sus tildes en `Comment[es]` y la inglesa en `Comment[en]` (plantilla
`empaquetado/linux/apolo.desktop`). El icono, el editor y la licencia solo saldrían desde un
repositorio con catálogo AppStream (PPA o snap). · 2026-10-22

## Linux sin decoraciones: sin sombra ni esquinas redondeadas

Con `decorations: false`, GTK3 no dibuja el marco de la ventana (sombra, radio y el borde exterior de
redimensionado): `get_shadow_width()` sale sin sombra si la ventana no está decorada. La opción
`shadow` de Tauri no hace nada en Linux. En Wayland, tao pone de barra un `EventBox` vacío, así que
basta con `set_decorated(true)` después de crearla para tener el marco sin barra (con `set_titlebar`
GTK desrealiza la ventana y puede romper el webview). El radio no recorta el webview: hacen falta la
ventana transparente y el radio en la página. En X11, no (volvería la barra del servidor). ·
2026-10-22

## Linux en la máquina virtual: huecos en la esquina de arriba a la izquierda

En la máquina virtual del cliente (VMPal, Ubuntu 26.10 ARM64, WebKitGTK 2.52.6) salen huecos al
repintar, siempre anclados en la esquina de arriba a la izquierda y del tamaño de lo repintado (el
botón de la barra lateral al moverse, la fila elegida al cambiar de sección). Con la ventana
transparente se ve el escritorio por ellos; opaca, salen en negro o en gris oscuro. **No son del
marco ni de la transparencia, ni de nuestro código**: salen igual con `APOLO_ESQUINAS_RECTAS=1` y con
la primera compilación de prueba, de antes del marco; la primera vez no se vieron porque en gris
oscuro casi no se distinguen. Tampoco los quitan `WEBKIT_SKIA_ENABLE_CPU_RENDERING=1`,
`WEBKIT_DISABLE_COMPOSITING_MODE=1` ni `WEBKIT_DISABLE_DMABUF_RENDERER=1`. Queda la gráfica virtual
de la máquina con WebKitGTK; en un Linux de verdad, sin ver (deuda.md). En la máquina virtual Fedora ARM del
cliente, en el mismo programa, no salen. Para ver dónde se pierde el
desplazamiento, `WEBKIT_SHOW_DAMAGE=1`. Y el código de la ventana de Linux solo lo revisa clippy en
CI: en el VPS no hay GTK (con la v0.15.0, un `if` anidado que pedía juntarse). · 2026-10-22

## Pages solo despliega desde `main`: con `release` hay que permitir las etiquetas

El entorno `github-pages` tenía la regla de despliegue «solo la rama `main`». `web.yml` se lanza
también al publicar una versión (ADR 0030), y entonces corre sobre la etiqueta (`refs/tags/vX.Y.Z`):
Pages lo rechaza con «Tag "vX.Y.Z" is not allowed to deploy to github-pages due to environment
protection rules». Se añadió la regla de etiquetas `v*` (Ajustes › Entornos › github-pages, o
`gh api -X POST repos/webcafeina/Apolo/environments/github-pages/deployment-branch-policies -f
name='v*' -f type=tag`). · 2026-10-22

## El AUR cerró el registro de cuentas nuevas

El 2026-10-22, aur.archlinux.org/register daba «New account registration is temporarily closed» (HTTP
503) por una ola de altas automáticas, sin cola ni fecha: se anuncia en aur-general y en las
noticias de Arch. No reintentar con un guion. Mientras, `aur.yml` desactivado y el PKGBUILD en la web
(ADR 0030). · 2026-10-22

## Un título con «…» que no se corta: `align-items` le quita el ancho

`white-space: nowrap; overflow: hidden; text-overflow: ellipsis` solo corta si la caja tiene un
ancho limitado. En una columna flex con `align-items: center` o `flex-start` (la cabecera de Linux,
centrada en GNOME y al principio en el Estudio y en KDE), los hijos no se estiran: miden lo que su
texto, y la columna, aunque tenga `min-width: 0`, no los frena. El `h1` con un nombre de archivo
largo se salía por encima de los modos del comparador, sin «…». Lo vio el cliente en Fedora KDE, y
pasaba igual en GNOME desde la v0.15.0. Arreglo: `max-width: 100%` en los hijos de `.titulos`. Lo
vigila `linux.spec.ts` con un nombre largo en GNOME y en KDE. · 2026-10-23

## macOS 11: sin `popover` ni `<dialog>` en el WKWebView

Apolo promete macOS 11 (ADR 0010), y su WKWebView es el Safari de entonces: no tiene el atributo
`popover` (Safari 17) y `<dialog>` llegó en la 15.4. La ventanita de Redimensionar (`Emergente.tsx`,
ADR 0032) es propia: un `div` con `position: absolute` junto a su botón, con `role="dialog"`, y
Escape, el clic fuera y el foco fuera hechos a mano. Tampoco `:has()` en lo que tenga que funcionar
allí. · 2026-10-23

## Una ventanita dentro de algo que se desplaza: el foco la descoloca

Al abrir la ventanita, el foco entraba en su primer control antes de que se diera la vuelta hacia
arriba: el navegador desplazaba el contenido de Lotes para enseñar dónde estaba un momento antes, y
quedaba cortada por arriba. El foco va en el `requestAnimationFrame` siguiente, con
`preventScroll`, y después `scrollIntoView({ block: "nearest" })`. Y para decidir hacia dónde se
abre hay que mirar lo que la recorta (el primer antepasado con `overflow`), no solo la ventana. En
las pruebas, pulsar dentro mientras hace su animación de entrada hace que Playwright desplace el
contenido: se espera a que entre. · 2026-10-24

## El repositorio RPM, ahora en zstd

Desde la v0.18.0, el `createrepo_c` del runner de `web.yml` escribe los metadatos en zstd
(`primary.xml.zst`), no en gzip. dnf (Fedora, RHEL 9) y zypper los leen; para mirarlos a mano hay
que buscar `.zst` en `repomd.xml` y descomprimir con `zstd -dc`, o la comprobación da un 404 que no
es del repositorio. · 2026-10-24

## macOS pone en inglés el menú y los diálogos de una aplicación de Tauri

Dos causas distintas, con el mismo síntoma (lo vio el cliente con la v0.18.2, con el Mac en
español). **El menú**: sin menú propio, Tauri pone `Menu::default`, con los textos escritos en
inglés («About», «Hide», «Quit», «Edit»…). **Los diálogos** (abrir y guardar de Finder) y lo que
macOS añade al menú (Emoji y símbolos, Dictado): AppKit los pone en el idioma en que esté la
aplicación, y un `.app` sin `CFBundleLocalizations` ni carpetas `.lproj` cuenta como solo inglés.
Arreglo: menú propio con `tr!` (`menu_macos.rs`) y los idiomas en `src-tauri/Info.plist`, que Tauri
fusiona con el suyo, más `es.lproj` y `en.lproj` en `bundle.macOS.files`. En Windows y Linux los
diálogos los pone el sistema en su idioma (IFileOpenDialog, GTK o el portal) y no hay menú. · 2026-10-26

## `astro preview` (Astro 7) deja un candado y un servidor vivo

Si un `astro preview` lanzado desde un guion no se cierra bien, el siguiente dice «Preview server
already running» y termina enseguida: Playwright lo toma por un servidor caído («Process from
config.webServer exited early») o por el puerto ocupado. `astro preview stop` lo para, y
`--ignore-lock` (en `sitio/playwright.config.ts` y `herramientas/readme.mjs`) no mira el candado. · 2026-10-26

## Medir «antes de pintar» desde Playwright

Un `addInitScript` corre antes de que exista `document.documentElement`: un `MutationObserver` puesto
ahí no observa nada. Para saber qué atributos tiene `<html>` al pintar, se intercepta el HTML con
`page.route` y se mete un `<script>` justo al abrir `<body>` (pruebas/deteccion.spec.ts). · 2026-10-26

## axe y las animaciones de aparecer

Lo que sube al entrar en pantalla está a media opacidad hasta entonces, y axe mide el contraste de
ese momento: da fallos de color que en reposo no existen. Se pasa con `reducedMotion: "reduce"`,
que deja la página en reposo (y así la ve quien lo pide). · 2026-10-26

## La grabación de vídeo de Playwright sale borrosa y sin puntero

`recordVideo` graba en VP8 con poca calidad y no pinta el cursor. Para el vídeo de la web
(`frontend/e2e/video.spec.ts`): el screencast de Chromium por CDP (`Page.startScreencast`) a doble
densidad, cada fotograma con su hora, montado con ffmpeg (`sitio/herramientas/video.mjs`), y un
puntero dibujado que sigue al ratón. Guardar los PNG ralentiza la página: el montaje lo acelera
1,6×. Y apolo-dev arranca en inglés solo si el idioma va en la variable que mira
`frontend/playwright.config.ts` (`CAPTURAS_IDIOMA` o `VIDEO_IDIOMA`). · 2026-10-26

## La CSP de la web va al final de la cabeza, y bloquea lo que se inyecta en las pruebas

Con `security.csp`, Astro pone el `<meta http-equiv="content-security-policy">` al final de
`<head>`, detrás de `temprano.js` y del arranque de GTM, que van en línea: se ejecutan antes de que
la política exista, y desde ahí todo lo demás tiene que cumplirla. Una prueba que mete su propio
`<script>` en la página (el parpadeo de `deteccion.spec.ts`) lo ve bloqueado: quita la CSP de su
copia de la página. Y cualquier etiqueta nueva de GTM cuyo dominio no esté en `astro.config.mjs` no
cargará: la consola lo dice como `Refused to load…`. · 2026-10-27

## Cloudflare delante de Pages: todo por reglas de `apolo`, y su robots.txt

Desde el 2026-10-27 `apolo` va con el proxy de Cloudflare (ADR 0034). Casi todos los ajustes de
Cloudflare son de la zona entera (`webcafeina.com`, la web principal incluida), así que lo de Apolo
va en reglas con `http.host eq "apolo.webcafeina.com"`:
- una **Configuration Rule** con SSL «Estricto»;
- una **Cache Rule** que omite la caché en `/apt/`, `/rpm/` y `/aur/`, o `apt update` y `dnf`
  verían el repositorio viejo tras publicar;
- una **Response Header Transform Rule** con HSTS, `nosniff`, `Referrer-Policy`,
  `Permissions-Policy`, COOP, `X-Frame-Options` y la CSP de `frame-ancestors`.

Bot Fight Mode está apagado en la zona: encendido, desafiaría a `apt` y `dnf`. AI Crawl Control no
bloquea a ningún rastreador. El **robots.txt gestionado de Cloudflare está activo** aunque no se
vea en el panel: sin robots.txt propio sirve el suyo (solo comentarios de «content signals», sin
ninguna señal), y con uno propio le añade ese texto delante. Se deja: es neutro y cambiarlo
afecta a toda la zona.

Al cliente, el certificado se lo da Cloudflare; el de GitHub (Let's Encrypt) solo lo ve
Cloudflare, que lo exige válido. **GitHub lo renueva hacia el 20 de diciembre de 2026** (caduca el
19 de enero de 2027): comprobar ese día que se renovó con el proxy delante
(`openssl s_client -connect webcafeina.github.io:443 -servername apolo.webcafeina.com`). Si no,
nube gris un rato, que GitHub lo emita, y otra vez naranja. · 2026-10-27

## El `gh` de Ubuntu no sabe verificar la procedencia

`gh attestation verify` llegó en `gh` 2.49, y Ubuntu trae la 2.46: con ella la orden ni existe, y
un bucle que la use da «sin procedencia» para todos los ficheros aunque las atestaciones estén bien
(la API lo confirma: `gh api repos/webcafeina/Apolo/attestations/sha256:<suma>`). **Arreglado el
2026-10-27**: el cliente añadió al VPS el repositorio APT oficial de GitHub CLI
(`/etc/apt/sources.list.d/github-cli.list`, clave en `/etc/apt/keyrings/`), así que `/usr/bin/gh` es
la 2.103.3 y se pone al día con `apt upgrade`. Fuera de un terminal, `gh attestation verify` no
imprime nada: el resultado es el código de salida (0 bien, 1 mal). · 2026-10-27
