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

## «HTTP 502: Server Error» al subir el `.dmg` a la Release

Pasó con la v0.3.2: todo compilado, el icono del volumen puesto, y al reemplazar el `.dmg` en la
Release la API de GitHub dio un 502 y el trabajo de macOS cayó; el de adjuntar ya no corrió, y el
borrador se quedó a medias. Es pasajero: `gh run rerun <id> --failed` lo arregla, y desde entonces
`publicar.yml` reintenta la subida cuatro veces. · 2026-10-07
