# Un solo proyecto con libjxl y libavif, para que compartan zlib y libpng y no
# haya dos copias de los mismos símbolos en Apolo (ADR 0021). Lo configura
# build.rs, que le pasa:
#   APOLO_LIBJXL       la copia de libjxl con sus third_party ya puestos
#   APOLO_LIBAVIF      vendor/libavif
#   APOLO_JPEG         las cabeceras de mozjpeg-sys (la biblioteca la enlaza Rust)
#   APOLO_SHARPYUV     las cabeceras de sharpyuv (el código viene de libwebp-sys)
#   APOLO_PUENTE       c/puente.c
#   APOLO_MEDIR        c/medir.cc
cmake_minimum_required(VERSION 3.16)
project(apolo_avifjxl C CXX)

set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# JPEG: solo las cabeceras. Las dos buscan una biblioteca, y se les da un
# fichero que existe; Apolo enlaza la de mozjpeg-sys.
set(JPEG_INCLUDE_DIR "${APOLO_JPEG}" CACHE PATH "" FORCE)
set(JPEG_LIBRARY "${APOLO_JPEG}/jpeglib.h" CACHE FILEPATH "" FORCE)

# ── libjxl, como la compila su ci.sh release ──────────────────────────────────
set(JPEGXL_VERSION "0.12.0" CACHE STRING "" FORCE)
foreach(opcion BENCHMARK EXAMPLES MANPAGES DOXYGEN OPENEXR JNI PLUGINS VIEWERS
               DEVTOOLS FUZZERS JPEGLI)
  set(JPEGXL_ENABLE_${opcion} OFF CACHE BOOL "" FORCE)
endforeach()
set(JPEGXL_ENABLE_TOOLS ON CACHE BOOL "" FORCE)
set(JPEGXL_ENABLE_SJPEG ON CACHE BOOL "" FORCE)
set(JPEGXL_ENABLE_SKCMS ON CACHE BOOL "" FORCE)
set(JPEGXL_BUNDLE_LIBPNG ON CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
# En Windows, CMake encuentra el awk de Git, y libpng intenta generar su
# pnglibconf.h compilando un fichero que no ve zlib.h. Sin awk usa el que trae
# hecho, que es la configuración por defecto. OFF, y no NOTFOUND: find_program
# no vuelve a buscar, y if(AWK) es falso.
if(WIN32)
  set(AWK OFF CACHE FILEPATH "" FORCE)
endif()
add_subdirectory("${APOLO_LIBJXL}" libjxl EXCLUDE_FROM_ALL)

# ── libavif, como su CI de Linux, con la zlib y la libpng de libjxl ───────────
# FindZLIB y FindPNG se conforman con las rutas; los objetivos ZLIB::ZLIB y
# PNG::PNG ya existen o se crean vacíos, y avif_apps se enlaza a png_static.
set(ZLIB_INCLUDE_DIR "${APOLO_LIBJXL}/third_party/zlib" CACHE PATH "" FORCE)
set(ZLIB_LIBRARY "${APOLO_LIBJXL}/third_party/zlib/zlib.h" CACHE FILEPATH "" FORCE)
set(PNG_PNG_INCLUDE_DIR "${APOLO_LIBJXL}/third_party/libpng" CACHE PATH "" FORCE)
set(PNG_LIBRARY "${APOLO_LIBJXL}/third_party/libpng/png.h" CACHE FILEPATH "" FORCE)

add_library(sharpyuv::sharpyuv INTERFACE IMPORTED)
target_include_directories(sharpyuv::sharpyuv INTERFACE "${APOLO_SHARPYUV}")

set(AVIF_CODEC_AOM LOCAL CACHE STRING "" FORCE)
set(AVIF_CODEC_AOM_ENCODE ON CACHE BOOL "" FORCE)
set(AVIF_CODEC_AOM_DECODE ON CACHE BOOL "" FORCE)
set(AVIF_CODEC_DAV1D OFF CACHE STRING "" FORCE)
set(AVIF_LIBYUV LOCAL CACHE STRING "" FORCE)
set(AVIF_LIBSHARPYUV SYSTEM CACHE STRING "" FORCE)
set(AVIF_JPEG SYSTEM CACHE STRING "" FORCE)
set(AVIF_ZLIBPNG SYSTEM CACHE STRING "" FORCE)
set(AVIF_LIBXML2 OFF CACHE STRING "" FORCE)
set(AVIF_BUILD_APPS ON CACHE BOOL "" FORCE)
set(AVIF_BUILD_TESTS OFF CACHE BOOL "" FORCE)
# Con MSVC (y clang-cl), el CMake de libyuv no compila sus ficheros NEON, pero
# sus cabeceras los activan en ARM64: sin esto, no enlaza. Se queda con el C.
if(MSVC AND CMAKE_SYSTEM_PROCESSOR MATCHES "^(ARM64|arm64|aarch64)$")
  add_compile_definitions(LIBYUV_DISABLE_NEON LIBYUV_DISABLE_SVE LIBYUV_DISABLE_SME)
endif()
add_subdirectory("${APOLO_LIBAVIF}" libavif EXCLUDE_FROM_ALL)
target_link_libraries(avif_apps PRIVATE png_static)

# ── Las herramientas, con su main renombrado ──────────────────────────────────
add_library(apolo_avif STATIC
  "${APOLO_LIBAVIF}/apps/avifenc.c"
  "${APOLO_LIBAVIF}/apps/avifdec.c"
  "${APOLO_PUENTE}")
set_source_files_properties("${APOLO_LIBAVIF}/apps/avifenc.c" PROPERTIES
  COMPILE_DEFINITIONS "main=apolo_avifenc_main")
set_source_files_properties("${APOLO_LIBAVIF}/apps/avifdec.c" PROPERTIES
  COMPILE_DEFINITIONS "main=apolo_avifdec_main")
target_link_libraries(apolo_avif PRIVATE avif_apps avif)

# cjxl y djxl ya traen su main renombrado y sus exit() cambiados (build.rs).
add_library(apolo_cjxl STATIC "${APOLO_LIBJXL}/tools/cjxl_main.cc")
add_library(apolo_djxl STATIC "${APOLO_LIBJXL}/tools/djxl_main.cc")
foreach(t apolo_cjxl apolo_djxl)
  target_link_libraries(${t} PRIVATE jxl jxl_extras_codec jxl_threads jxl_tool)
  target_include_directories(${t} PRIVATE "${APOLO_LIBJXL}")
endforeach()

# Las medidas (ADR 0022): SSIMULACRA 2 de las herramientas de libjxl.
add_library(apolo_medir STATIC
  "${APOLO_MEDIR}"
  "${APOLO_LIBJXL}/tools/ssimulacra2.cc"
  # packed_image_convert y compañía: la parte de extras que usa lo interno
  # de libjxl, que cjxl no necesita y no va en jxl_extras_codec.
  $<TARGET_OBJECTS:jxl_extras_internal-obj>)
target_link_libraries(apolo_medir PRIVATE jxl_gauss_blur jxl_extras_codec jxl_tool jxl_cms)
target_include_directories(apolo_medir PRIVATE "${APOLO_LIBJXL}")

add_custom_target(apolo ALL)
add_dependencies(apolo apolo_avif apolo_cjxl apolo_djxl apolo_medir jxl_gauss_blur avif_apps avif aom yuv
  jxl jxl_cms jxl_threads jxl_extras_codec jxl_tool hwy brotlienc brotlidec
  brotlicommon png_static zlibstatic sjpeg)
