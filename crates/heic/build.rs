//! Compila libde265 (el decodificador HEVC) y libheif desde su código fuente
//! (los submódulos de vendor/), estáticas y solo con lo necesario para leer
//! HEIC: ningún otro códec, ni plugins, ni herramientas.
//!
//! El crate libheif-sys tiene un modo «embebido», pero compila libheif sin
//! decodificador HEVC —lo espera instalado en el sistema— y sin él no se abre
//! ninguna foto de iPhone (docs/trampas.md). Por eso se hace aquí.

use std::path::PathBuf;

fn main() {
    let vendor = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor");
    for sub in ["libde265", "libheif"] {
        assert!(
            vendor.join(sub).join("CMakeLists.txt").exists(),
            "falta vendor/{sub}: git submodule update --init --recursive"
        );
    }
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc";
    let sistema = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // libde265: solo la biblioteca. Con optimización siempre: en un perfil de
    // depuración, decodificar una foto de 12 MP tardaría segundos.
    let de265 = cmake::Config::new(vendor.join("libde265"))
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("ENABLE_SDL", "OFF")
        .define("ENABLE_DECODER", "OFF")
        .define("ENABLE_ENCODER", "OFF")
        .define("ENABLE_AVX512", "OFF")
        .build();
    let lib_de265 = de265
        .join("lib")
        .join(if msvc { "libde265.lib" } else { "libde265.a" });

    let mut heif = cmake::Config::new(vendor.join("libheif"));
    heif.profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("ENABLE_PLUGIN_LOADING", "OFF")
        .define("WITH_LIBDE265", "ON")
        .define("WITH_LIBDE265_PLUGIN", "OFF")
        // Se le da libde265 a mano: así no busca nada en el sistema.
        .define("LIBDE265_INCLUDE_DIR", de265.join("include"))
        .define("LIBDE265_LIBRARY", &lib_de265)
        .define("WITH_LIBSHARPYUV", "OFF")
        .define("WITH_EXAMPLES", "OFF")
        .define("WITH_EXAMPLE_HEIF_THUMB", "OFF")
        .define("WITH_EXAMPLE_HEIF_VIEW", "OFF")
        .define("WITH_GDK_PIXBUF", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_DOCUMENTATION", "OFF")
        .define("WITH_HEADER_COMPRESSION", "OFF")
        .define("WITH_UNCOMPRESSED_CODEC", "OFF")
        // libde265 es estática: sin esto, en Windows sus cabeceras piden la DLL.
        .cflag("-DLIBDE265_STATIC_BUILD")
        .cxxflag("-DLIBDE265_STATIC_BUILD");
    for codec in [
        "X265",
        "KVAZAAR",
        "UVG266",
        "VVDEC",
        "VVENC",
        "X264",
        "OpenH264_DECODER",
        "DAV1D",
        "AOM_DECODER",
        "AOM_ENCODER",
        "SvtEnc",
        "RAV1E",
        "JPEG_DECODER",
        "JPEG_ENCODER",
        "OpenJPEG_ENCODER",
        "OpenJPEG_DECODER",
        "FFMPEG_DECODER",
        "OPENJPH_ENCODER",
    ] {
        heif.define(format!("WITH_{codec}"), "OFF");
    }
    let heif = heif.build();

    println!(
        "cargo:rustc-link-search=native={}",
        heif.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        de265.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=heif");
    println!(
        "cargo:rustc-link-lib=static={}",
        if msvc { "libde265" } else { "de265" }
    );
    // Las dos son C++: hace falta su biblioteca estándar.
    match sistema.as_str() {
        "macos" | "ios" => println!("cargo:rustc-link-lib=c++"),
        "windows" => {}
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }
    println!("cargo:rerun-if-changed=build.rs");
}
