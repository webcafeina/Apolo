//! Compila libavif (con aom y libyuv) y libjxl desde los submódulos de vendor/,
//! estáticas, y los main de avifenc, avifdec, cjxl y djxl con otro nombre, para
//! llamarlos desde Apolo con sus mismos argumentos (ADR 0021).
//!
//! Tres cosas que no se ven a simple vista:
//! - **libjxl se compila con clang**, también en Linux: con gcc, cjxl da otros
//!   bytes, por cómo redondea la coma flotante (docs/trampas.md). El cjxl
//!   oficial sale de clang.
//! - libjxl espera sus dependencias en `third_party/`, con rutas fijas. Se copia
//!   a OUT_DIR con las de vendor/ dentro, y de paso se cambian en la copia el
//!   nombre de los main y sus `exit()` (ver c/puente.c).
//! - zlib y libpng van una sola vez, las de libjxl, y libavif las comparte. El
//!   JPEG es el de mozjpeg-sys y sharpyuv el de libwebp-sys: los dos ya están
//!   en Apolo, y una segunda copia daría símbolos repetidos.

use std::fs;
use std::path::{Path, PathBuf};

const SUBMODULOS: [&str; 10] = [
    "libavif", "aom", "libyuv", "libjxl", "brotli", "highway", "skcms", "sjpeg", "libpng", "zlib",
];

/// Lo que va dentro de `libjxl/third_party/`, desde vendor/.
const TERCEROS_JXL: [&str; 6] = ["brotli", "highway", "skcms", "sjpeg", "libpng", "zlib"];

fn main() {
    let raiz = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = raiz.join("vendor");
    let salida = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for sub in SUBMODULOS {
        assert!(
            fs::read_dir(vendor.join(sub)).is_ok_and(|mut d| d.next().is_some()),
            "falta vendor/{sub}: git submodule update --init"
        );
        println!("cargo:rerun-if-changed=vendor/{sub}");
    }
    let sistema = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc";

    let libjxl = preparar_libjxl(&vendor, &salida.join("fuentes"));
    let jpeg = cabeceras_jpeg(&salida.join("jpeg"));

    let mut c = cmake::Config::new(raiz.join("cmake"));
    c.profile("Release")
        // Las opciones de compilación son las de cada proyecto, sin las que
        // añadiría el crate cc: cambiarlas puede cambiar los bytes.
        .no_default_flags(true)
        .define("APOLO_LIBJXL", &libjxl)
        .define("APOLO_LIBAVIF", vendor.join("libavif"))
        .define("APOLO_JPEG", &jpeg)
        .define("APOLO_SHARPYUV", raiz.join("cabeceras"))
        .define("APOLO_PUENTE", raiz.join("c").join("puente.c"))
        // aom y libyuv salen de vendor/, nunca de la red.
        .define("FETCHCONTENT_FULLY_DISCONNECTED", "ON")
        .define("FETCHCONTENT_SOURCE_DIR_LIBAOM", vendor.join("aom"))
        .define("FETCHCONTENT_SOURCE_DIR_LIBYUV", vendor.join("libyuv"))
        .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
        .build_target("apolo");
    // aom y libyuv eligen su SIMD por el procesador. El Mac universal compila
    // x86_64 en un Mac ARM: hay que decírselo, o compilarían NEON para x86.
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let cpu = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        otra => panic!("arquitectura sin probar para aom: {otra}"),
    };
    c.define("AOM_TARGET_CPU", cpu);
    let cruzada = std::env::var("HOST").unwrap() != std::env::var("TARGET").unwrap();
    match sistema.as_str() {
        "macos" => {
            let minimo =
                std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "11.0".into());
            c.define("CMAKE_OSX_DEPLOYMENT_TARGET", &minimo);
            if cruzada {
                c.define("CMAKE_SYSTEM_NAME", "Darwin")
                    .define("CMAKE_SYSTEM_PROCESSOR", cpu);
            }
        }
        "windows" if msvc => {
            c.generator_toolset("ClangCL");
        }
        _ => {
            let (cc, cxx) = clang();
            c.define("CMAKE_C_COMPILER", cc)
                .define("CMAKE_CXX_COMPILER", cxx);
        }
    }
    let construido = c.build().join("build");

    // De quien usa a quien es usado: el enlazador de Linux lee en una pasada.
    // apolo_avif va la primera: lleva el puente (c/puente.c), al que llama
    // Rust, y que llama a los cuatro main y define apolo_salir, que usan cjxl y
    // djxl. Sin +whole-archive: con él, rustc la saca del paquete y la pone
    // detrás de las demás, y en Linux ARM64 no enlaza (docs/trampas.md).
    let orden = [
        "apolo_avif",
        "apolo_cjxl",
        "apolo_djxl",
        "jxl_tool",
        "jxl_extras_codec",
        "jxl_threads",
        "jxl",
        "jxl_cms",
        "sjpeg",
        "hwy",
        "brotlienc",
        "brotlidec",
        "brotlicommon",
        "avif_apps",
        "avif_internal",
        "aom",
        "yuv",
        "png16",
        "z",
    ];
    let encontradas = bibliotecas(&construido);
    let mut carpetas: Vec<PathBuf> = Vec::new();
    for nombre in orden {
        let (carpeta, enlace) = encontradas
            .iter()
            .find(|(_, n, _)| es_la(n, nombre, msvc))
            .map(|(c, _, e)| (c.clone(), e.clone()))
            .unwrap_or_else(|| panic!("no se ha compilado {nombre}"));
        if !carpetas.contains(&carpeta) {
            println!("cargo:rustc-link-search=native={}", carpeta.display());
            carpetas.push(carpeta);
        }
        println!("cargo:rustc-link-lib=static={enlace}");
    }
    match sistema.as_str() {
        "macos" | "ios" => println!("cargo:rustc-link-lib=c++"),
        "windows" => {}
        _ => {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=m");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cmake");
    println!("cargo:rerun-if-changed=c");
    println!("cargo:rerun-if-changed=cabeceras");
}

/// Copia libjxl a `destino` con sus dependencias en third_party/ y cambia, en
/// la copia, los main de cjxl y djxl y sus salidas con exit().
fn preparar_libjxl(vendor: &Path, destino: &Path) -> PathBuf {
    // Sin borrar lo copiado antes: un fichero igual no se vuelve a escribir, y
    // CMake no lo recompila.
    let libjxl = destino.join("libjxl");
    copiar(&vendor.join("libjxl"), &libjxl, &[]);
    for t in TERCEROS_JXL {
        let dentro = libjxl.join("third_party").join(t);
        // Lo que no hace falta para compilar: las pruebas de brotli y los
        // perfiles de muestra de skcms, que son casi todo lo que pesan.
        copiar(&vendor.join(t), &dentro, &["tests", "profiles"]);
    }

    let herramientas = libjxl.join("tools");
    cambiar(
        &herramientas.join("cmdline.h"),
        &[
            (
                "#define JPEGXL_TOOLS_ABORT(M)",
                "extern \"C\" [[noreturn]] void apolo_salir(int);\n\n#define JPEGXL_TOOLS_ABORT(M)",
            ),
            ("std::exit(EXIT_FAILURE)", "apolo_salir(EXIT_FAILURE)"),
        ],
    );
    cambiar(
        &herramientas.join("cjxl_main.cc"),
        &[
            ("exit(EXIT_FAILURE)", "apolo_salir(EXIT_FAILURE)"),
            (
                "int main(int argc, char** argv) {",
                "extern \"C\" int apolo_cjxl_main(int argc, char** argv) {",
            ),
        ],
    );
    cambiar(
        &herramientas.join("djxl_main.cc"),
        &[(
            "int main(int argc, const char* argv[]) {",
            "extern \"C\" int apolo_djxl_main(int argc, const char* argv[]) {",
        )],
    );
    libjxl
}

/// Sustituye cada texto en el fichero; si alguno no está, la versión de libjxl
/// ha cambiado y hay que revisar el puente.
fn cambiar(fichero: &Path, cambios: &[(&str, &str)]) {
    let original = fs::read_to_string(fichero).unwrap();
    let mut texto = original.clone();
    for (antes, despues) in cambios {
        if texto.contains(despues) {
            continue; // ya cambiado en una compilación anterior
        }
        assert!(
            texto.contains(antes),
            "{} ya no tiene «{antes}»: revisa c/puente.c",
            fichero.display()
        );
        texto = texto.replace(antes, despues);
    }
    if texto != original {
        fs::write(fichero, texto).unwrap();
    }
}

/// Copia una carpeta sin `.git` y sin las carpetas `saltar` de primer nivel.
fn copiar(origen: &Path, destino: &Path, saltar: &[&str]) {
    fs::create_dir_all(destino).unwrap();
    for entrada in fs::read_dir(origen).unwrap() {
        let entrada = entrada.unwrap();
        let nombre = entrada.file_name();
        if nombre == ".git" || saltar.iter().any(|s| nombre == *s) {
            continue;
        }
        let tipo = entrada.file_type().unwrap();
        let hacia = destino.join(&nombre);
        if tipo.is_dir() {
            copiar(&entrada.path(), &hacia, &[]);
        } else if tipo.is_file() && !igual(&entrada.path(), &hacia) {
            fs::copy(entrada.path(), hacia).unwrap();
        }
    }
}

/// Si `copia` es ya igual que `original`, o es la versión cambiada de una
/// compilación anterior (más nueva que el original).
fn igual(original: &Path, copia: &Path) -> bool {
    let (Ok(a), Ok(b)) = (fs::metadata(original), fs::metadata(copia)) else {
        return false;
    };
    match (a.modified(), b.modified()) {
        (Ok(ma), Ok(mb)) => mb >= ma,
        _ => false,
    }
}

/// Junta en una carpeta las cabeceras de mozjpeg-sys (jpeglib.h y la jconfig.h
/// que genera), porque CMake busca el JPEG en una sola.
fn cabeceras_jpeg(destino: &Path) -> PathBuf {
    let rutas = std::env::var_os("DEP_JPEG_INCLUDE").expect("mozjpeg-sys no da sus cabeceras");
    fs::create_dir_all(destino).unwrap();
    for carpeta in std::env::split_paths(&rutas) {
        for nombre in [
            "jpeglib.h",
            "jmorecfg.h",
            "jerror.h",
            "jconfig.h",
            "jpegint.h",
        ] {
            let f = carpeta.join(nombre);
            if f.exists() {
                fs::copy(&f, destino.join(nombre)).unwrap();
            }
        }
    }
    assert!(
        destino.join("jconfig.h").exists(),
        "falta jconfig.h de mozjpeg-sys"
    );
    destino.to_path_buf()
}

/// clang en Linux: `APOLO_CLANG` (la carpeta bin de un LLVM), o clang-18, o
/// clang. Sin clang no se compila: con gcc, cjxl no daría los mismos bytes.
fn clang() -> (PathBuf, PathBuf) {
    println!("cargo:rerun-if-env-changed=APOLO_CLANG");
    if let Some(bin) = std::env::var_os("APOLO_CLANG") {
        let bin = PathBuf::from(bin);
        return (bin.join("clang"), bin.join("clang++"));
    }
    for (cc, cxx) in [("clang-18", "clang++-18"), ("clang", "clang++")] {
        if let Some(cc) = en_path(cc) {
            return (cc, en_path(cxx).unwrap_or_else(|| PathBuf::from(cxx)));
        }
    }
    panic!("hace falta clang para libjxl (docs/trampas.md): instala clang-18 o pon APOLO_CLANG");
}

fn en_path(programa: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(programa))
        .find(|p| p.is_file())
}

/// Todas las bibliotecas estáticas de la compilación: (carpeta, nombre del
/// fichero sin extensión, nombre para enlazar).
fn bibliotecas(carpeta: &Path) -> Vec<(PathBuf, String, String)> {
    let mut todas = Vec::new();
    let mut pendientes = vec![carpeta.to_path_buf()];
    while let Some(c) = pendientes.pop() {
        for entrada in fs::read_dir(&c).unwrap().flatten() {
            let ruta = entrada.path();
            if ruta.is_dir() {
                pendientes.push(ruta);
                continue;
            }
            let fichero = entrada.file_name().to_string_lossy().into_owned();
            let base = if let Some(b) = fichero.strip_suffix(".a") {
                b.strip_prefix("lib").unwrap_or(b)
            } else if let Some(b) = fichero.strip_suffix(".lib") {
                b
            } else {
                continue;
            };
            todas.push((c.clone(), base.to_string(), base.to_string()));
        }
    }
    todas
}

/// Si el fichero `fichero` es la biblioteca `nombre`. Con MSVC, libpng y zlib
/// llevan otro nombre.
fn es_la(fichero: &str, nombre: &str, msvc: bool) -> bool {
    if fichero == nombre {
        return true;
    }
    match nombre {
        "png16" => fichero == "libpng16_static" || (msvc && fichero.starts_with("libpng16")),
        "z" => fichero == "zlibstatic" || fichero == "zs",
        _ => false,
    }
}
