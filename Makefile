# Apolo. `make` o `make ayuda` para ver los objetivos.
SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

.PHONY: ayuda comprobar rust interfaz contraste tokens cli app dev

ayuda:
	@echo "make comprobar  formato, clippy, pruebas, contraste e interfaz (la puerta de CI)"
	@echo "make tokens     regenera frontend/src/tokens.css desde crates/tema"
	@echo "make contraste  solo la prueba de contraste"
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
