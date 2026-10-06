# Apolo. `make` o `make ayuda` para ver los objetivos.
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(PATH)

.PHONY: ayuda comprobar rust interfaz contraste tokens cli app dev dev-web e2e capturas equivalencia iconos ventana-dmg

ayuda:
	@echo "make comprobar  formato, clippy, pruebas, contraste e interfaz (la puerta de CI)"
	@echo "make tokens     regenera frontend/src/tokens.css desde crates/tema"
	@echo "make contraste  solo la prueba de contraste"
	@echo "make equivalencia  mismo fichero que el cwebp oficial, byte a byte (descarga cwebp)"
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
RUST_PAQUETES := $(shell pkg-config --exists webkit2gtk-4.1 2>/dev/null && echo --workspace || echo -p apolo-nucleo -p apolo-tema -p apolo-cli -p apolo-servicio -p apolo-dev)

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

equivalencia: $(CWEBP)
	APOLO_CWEBP=$(abspath $(CWEBP)) APOLO_CWEBP_OBLIGATORIO=1 \
		cargo test --release -p apolo-nucleo --test equivalencia_cwebp -- --nocapture

# La marca: de los SVG de empaquetado/ a los PNG (Chromium de Playwright, como
# Esfinge) y de ahí a los iconos de cada sistema. Los PNG van a git.
iconos:
	pnpm -C frontend install --frozen-lockfile
	node frontend/herramientas/rasterizar.mjs
	pnpm tauri icon empaquetado/icono.png -o src-tauri/icons
	rm -rf src-tauri/icons/android src-tauri/icons/ios

ventana-dmg:
	node frontend/herramientas/ventana-dmg.mjs
