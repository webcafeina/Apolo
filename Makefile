# Apolo. `make` o `make ayuda` para ver los objetivos.
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(PATH)
# libjxl se compila con clang (ADR 0021). En el VPS, un LLVM 18 sin instalar,
# que necesita su libtinfo.so.5 de pega (docs/trampas.md); en CI, clang-18.
ifneq ($(wildcard $(HOME)/.local/llvm18/bin/clang),)
export APOLO_CLANG := $(HOME)/.local/llvm18/bin
export LD_LIBRARY_PATH := $(HOME)/.local/llvm18/compat$(if $(LD_LIBRARY_PATH),:$(LD_LIBRARY_PATH))
endif

.PHONY: referencias cjpeg-oficial ayuda comprobar rust interfaz contraste tokens cli app dev dev-web e2e capturas equivalencia iconos ventana-dmg

ayuda:
	@echo "make comprobar  formato, clippy, pruebas, contraste e interfaz (la puerta de CI)"
	@echo "make tokens     regenera frontend/src/tokens.css desde crates/tema"
	@echo "make contraste  solo la prueba de contraste"
	@echo "make equivalencia  mismo fichero que cwebp, cjpeg, oxipng, qoiconv, avifenc y cjxl, byte a byte"
	@echo "make cli        compila el binario apolo (target/release/apolo)"
	@echo "make app        compila la aplicación (necesita libwebkit2gtk-4.1-dev en Linux)"
	@echo "make dev        la aplicación con recarga en caliente"
	@echo "make dev-web    el Estudio en el navegador, sin ventana (http://127.0.0.1:5173)"
	@echo "make e2e        las pruebas de la interfaz con Playwright, contra apolo-dev"
	@echo "make capturas   capturas del Estudio en claro y oscuro (frontend/capturas/)"
	@echo "make iconos     rasteriza el icono y el fondo del .dmg y regenera src-tauri/icons"
	@echo "make ventana-dmg  simula la ventana del .dmg sin un Mac (target/ventana-dmg.png)"

comprobar: interfaz rust

# Sin webkit2gtk (el VPS) no se puede compilar src-tauri: se comprueba el resto.
# En CI, con webkit, va todo el workspace.
RUST_PAQUETES := $(shell pkg-config --exists webkit2gtk-4.1 2>/dev/null && echo --workspace || echo -p apolo-heic -p apolo-avifjxl -p apolo-nucleo -p apolo-tema -p apolo-cli -p apolo-servicio -p apolo-dev)

rust:
	cargo fmt --all --check
	cargo clippy $(RUST_PAQUETES) --all-targets -- -D warnings
	cargo test $(RUST_PAQUETES)

interfaz:
	pnpm -C frontend install --frozen-lockfile
	pnpm -C frontend build
	pnpm -C frontend test

contraste:
	cargo test -p apolo-tema --test contraste

tokens:
	cargo run -q -p apolo-tema --bin tokens

cli:
	cargo build --release -p apolo-cli

app:
	pnpm install --frozen-lockfile
	pnpm tauri build

dev:
	pnpm install --frozen-lockfile
	pnpm tauri dev

# El Estudio sin ventana: apolo-dev (el mismo servicio que la aplicación, por
# HTTP) y Vite, que le reenvía /api y /pixeles. ADR 0013.
dev-web:
	cargo build -p apolo-dev
	pnpm -C frontend install --frozen-lockfile
	trap 'kill 0' EXIT; target/debug/apolo-dev & pnpm -C frontend exec vite --host 127.0.0.1

e2e:
	cargo build -p apolo-dev
	pnpm -C frontend install --frozen-lockfile
	pnpm -C frontend exec playwright test

capturas:
	cargo build -p apolo-dev
	CAPTURAS=1 pnpm -C frontend exec playwright test capturas

# La prueba de la ADR 0002 contra el cwebp oficial de Google, de la misma
# versión que la libwebp enlazada. El binario se descarga una vez y se
# comprueba su sha256. Solo Linux x86-64: es lo que hay para comparar.
CWEBP_VERSION := 1.6.0
CWEBP_SHA256 := 1c5ffab71efecefa0e3c23516c3a3a1dccb45cc310ae1095c6f14ae268e38067
CWEBP_DIR := target/cwebp-$(CWEBP_VERSION)
CWEBP := $(CWEBP_DIR)/libwebp-$(CWEBP_VERSION)-linux-x86-64/bin/cwebp

$(CWEBP):
	mkdir -p $(CWEBP_DIR)
	curl -sSfL https://storage.googleapis.com/downloads.webmproject.org/releases/webp/libwebp-$(CWEBP_VERSION)-linux-x86-64.tar.gz -o $(CWEBP_DIR)/cwebp.tar.gz
	echo "$(CWEBP_SHA256)  $(CWEBP_DIR)/cwebp.tar.gz" | sha256sum -c -
	tar xzf $(CWEBP_DIR)/cwebp.tar.gz -C $(CWEBP_DIR)

# El cjpeg oficial de MozJPEG (ADR 0020), de la misma versión que la que
# enlaza Apolo. Mozilla no publica binarios: se compila desde su fuente, con
# una libpng también compilada aquí (sin sudo no hay cabeceras de sistema).
# Sumas comprobadas de las dos fuentes. Su CMakeLists pide una versión de
# CMake tan vieja que CMake 4 la rechaza sin CMAKE_POLICY_VERSION_MINIMUM.
MOZJPEG_VERSION := 4.1.5
MOZJPEG_SHA256 := 9fcbb7171f6ac383f5b391175d6fb3acde5e64c4c4727274eade84ed0998fcc1
LIBPNG_VERSION := 1.6.50
LIBPNG_SHA256 := 71158e53cfdf2877bc99bcab33641d78df3f48e6e0daad030afe9cb8c031aa46
CJPEG_DIR := target/cjpeg-$(MOZJPEG_VERSION)
CJPEG := $(CJPEG_DIR)/instalado/bin/cjpeg

$(CJPEG):
	mkdir -p $(CJPEG_DIR)
	curl -sSfL https://github.com/pnggroup/libpng/archive/refs/tags/v$(LIBPNG_VERSION).tar.gz -o $(CJPEG_DIR)/libpng.tar.gz
	echo "$(LIBPNG_SHA256)  $(CJPEG_DIR)/libpng.tar.gz" | sha256sum -c -
	curl -sSfL https://github.com/mozilla/mozjpeg/archive/refs/tags/v$(MOZJPEG_VERSION).tar.gz -o $(CJPEG_DIR)/mozjpeg.tar.gz
	echo "$(MOZJPEG_SHA256)  $(CJPEG_DIR)/mozjpeg.tar.gz" | sha256sum -c -
	tar xzf $(CJPEG_DIR)/libpng.tar.gz -C $(CJPEG_DIR)
	tar xzf $(CJPEG_DIR)/mozjpeg.tar.gz -C $(CJPEG_DIR)
	cmake -S $(CJPEG_DIR)/libpng-$(LIBPNG_VERSION) -B $(CJPEG_DIR)/libpng-build \
		-DCMAKE_BUILD_TYPE=Release -DPNG_SHARED=OFF -DPNG_TESTS=OFF -DPNG_TOOLS=OFF \
		-DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_INSTALL_PREFIX=$(abspath $(CJPEG_DIR))/instalado
	cmake --build $(CJPEG_DIR)/libpng-build -j4 --target install
	cmake -S $(CJPEG_DIR)/mozjpeg-$(MOZJPEG_VERSION) -B $(CJPEG_DIR)/mozjpeg-build \
		-DCMAKE_BUILD_TYPE=Release -DENABLE_SHARED=OFF -DPNG_SUPPORTED=ON \
		-DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
		-DCMAKE_PREFIX_PATH=$(abspath $(CJPEG_DIR))/instalado \
		-DCMAKE_INSTALL_PREFIX=$(abspath $(CJPEG_DIR))/instalado
	cmake --build $(CJPEG_DIR)/mozjpeg-build -j4 --target install

cjpeg-oficial: $(CJPEG)

# El oxipng oficial (ADR 0020): el mismo crate que enlaza Apolo, instalado
# con --locked para que traiga las versiones exactas de libdeflate y zopfli.
OXIPNG_VERSION := 10.2.1
OXIPNG_DIR := target/oxipng-$(OXIPNG_VERSION)
OXIPNG := $(OXIPNG_DIR)/bin/oxipng

$(OXIPNG):
	cargo install oxipng --version =$(OXIPNG_VERSION) --locked --root $(OXIPNG_DIR)

# qoiconv, la herramienta del autor de QOI (ADR 0020), con stb_image. No hay
# versiones: van fijados los commits y las sumas de los cuatro ficheros.
QOI_COMMIT := ffb2d2cb74a1de60819b21b939f7209aa53e91c1
STB_COMMIT := 2c980bb59875b0d32144a71867fbdebb2f77cd20
QOICONV_DIR := target/qoiconv
QOICONV := $(QOICONV_DIR)/qoiconv

$(QOICONV):
	mkdir -p $(QOICONV_DIR)
	for f in qoi.h qoiconv.c; do curl -sSfL -o $(QOICONV_DIR)/$$f https://raw.githubusercontent.com/phoboslab/qoi/$(QOI_COMMIT)/$$f || exit 1; done
	for f in stb_image.h stb_image_write.h; do curl -sSfL -o $(QOICONV_DIR)/$$f https://raw.githubusercontent.com/nothings/stb/$(STB_COMMIT)/$$f || exit 1; done
	cd $(QOICONV_DIR) && printf '%s\n' \
		"7de6fca1a285b1c20d38f2723dec8b774eb9f144edb9710800a95feeea09375a  qoi.h" \
		"6abba2e650d93429c32b55ff5cc27ba18c56607385f5dfd4aed5d5bd017132ed  qoiconv.c" \
		"594c2fe35d49488b4382dbfaec8f98366defca819d916ac95becf3e75f4200b3  stb_image.h" \
		"cbd5f0ad7a9cf4468affb36354a1d2338034f2c12473cf1a8e32053cb6914a05  stb_image_write.h" \
		| sha256sum -c -
	cc -O2 -o $(QOICONV) $(QOICONV_DIR)/qoiconv.c -I$(QOICONV_DIR)

# avifenc y cjxl (ADR 0021): los binarios oficiales de Linux de cada versión.
# El de cjxl viene en .tar.lz; sin lzip, lo abre un Python de la biblioteca
# estándar (pruebas/referencias/deslzip.py).
AVIFENC_VERSION := 1.4.2
AVIFENC_SHA256 := faf58a670ffbfdc0e3559e6d37592cff277c447dd39453f1cd1d7d7f5a20b8ef
AVIFENC_DIR := target/avifenc-$(AVIFENC_VERSION)
AVIFENC := $(AVIFENC_DIR)/avifenc

$(AVIFENC):
	mkdir -p $(AVIFENC_DIR)
	curl -sSfL https://github.com/AOMediaCodec/libavif/releases/download/v$(AVIFENC_VERSION)/linux-artifacts.zip -o $(AVIFENC_DIR)/avif.zip
	echo "$(AVIFENC_SHA256)  $(AVIFENC_DIR)/avif.zip" | sha256sum -c -
	cd $(AVIFENC_DIR) && python3 -I -m zipfile -e avif.zip . && chmod +x avifenc avifdec

CJXL_VERSION := 0.12.0
CJXL_SHA256 := 5318a1ea40adad76d023e0c17a03d4627f8282f83cb2b575e69be8e74f1ff456
CJXL_DIR := target/cjxl-$(CJXL_VERSION)
CJXL := $(CJXL_DIR)/tools/cjxl

$(CJXL):
	mkdir -p $(CJXL_DIR)
	curl -sSfL https://github.com/libjxl/libjxl/releases/download/v$(CJXL_VERSION)/jxl-linux-x86_64-static.tar.lz -o $(CJXL_DIR)/jxl.tar.lz
	echo "$(CJXL_SHA256)  $(CJXL_DIR)/jxl.tar.lz" | sha256sum -c -
	python3 -I pruebas/referencias/deslzip.py $(CJXL_DIR)/jxl.tar.lz | tar x -C $(CJXL_DIR)

referencias: $(CWEBP) $(CJPEG) $(OXIPNG) $(QOICONV) $(AVIFENC) $(CJXL)

equivalencia: $(CWEBP) $(CJPEG) $(OXIPNG) $(QOICONV) $(AVIFENC) $(CJXL)
	APOLO_CWEBP=$(abspath $(CWEBP)) APOLO_CWEBP_OBLIGATORIO=1 \
		cargo test --release -p apolo-nucleo --test equivalencia_cwebp -- --nocapture
	APOLO_CJPEG=$(abspath $(CJPEG)) APOLO_CJPEG_OBLIGATORIO=1 \
		cargo test --release -p apolo-nucleo --test equivalencia_cjpeg -- --nocapture
	APOLO_OXIPNG=$(abspath $(OXIPNG)) APOLO_QOICONV=$(abspath $(QOICONV)) APOLO_REFERENCIAS_OBLIGATORIAS=1 \
		cargo test --release -p apolo-nucleo --test equivalencia_png_qoi -- --nocapture
	APOLO_AVIFENC=$(abspath $(AVIFENC)) APOLO_CJXL=$(abspath $(CJXL)) APOLO_REFERENCIAS_OBLIGATORIAS=1 \
		cargo test --release -p apolo-nucleo --test equivalencia_avif_jxl -- --nocapture

# La marca: de los SVG de empaquetado/ a los PNG (Chromium de Playwright, como
# Esfinge) y de ahí a los iconos de cada sistema. Los PNG van a git.
iconos:
	pnpm -C frontend install --frozen-lockfile
	node frontend/herramientas/rasterizar.mjs
	pnpm tauri icon empaquetado/icono.png -o src-tauri/icons
	rm -rf src-tauri/icons/android src-tauri/icons/ios

ventana-dmg:
	node frontend/herramientas/ventana-dmg.mjs
