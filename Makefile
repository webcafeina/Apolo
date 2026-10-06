# Apolo. `make` o `make ayuda` para ver los objetivos.
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(PATH)

.PHONY: ayuda comprobar rust interfaz contraste tokens cli app dev equivalencia

ayuda:
	@echo "make comprobar  formato, clippy, pruebas, contraste e interfaz (la puerta de CI)"
	@echo "make tokens     regenera frontend/src/tokens.css desde crates/tema"
	@echo "make contraste  solo la prueba de contraste"
	@echo "make equivalencia  mismo fichero que el cwebp oficial, byte a byte (descarga cwebp)"
	@echo "make cli        compila el binario apolo (target/release/apolo)"
	@echo "make app        compila la aplicación (necesita libwebkit2gtk-4.1-dev en Linux)"
	@echo "make dev        la aplicación con recarga en caliente"

comprobar: interfaz rust

# Sin webkit2gtk (el VPS) no se puede compilar src-tauri: se comprueba el resto.
# En CI, con webkit, va todo el workspace.
RUST_PAQUETES := $(shell pkg-config --exists webkit2gtk-4.1 2>/dev/null && echo --workspace || echo -p apolo-nucleo -p apolo-tema -p apolo-cli)

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
