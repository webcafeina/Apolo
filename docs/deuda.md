# Deuda y cabos sueltos

Última actualización: **2026-10-06**

Lo que sabemos que está a medias, mal o sin comprobar. Los bloqueantes primero. Lo saldado se tacha
y se queda.

## Sin comprobar

Lo más caro de esta lista no es lo que está mal, es lo que no sabemos si lo está.

| Elemento | Severidad | Impacto | Estado |
|---|---|---|---|
| **La ventana no se compila en el VPS** | Media | Falta `libwebkit2gtk-4.1-dev` (y `pkg-config`), que exige sudo. Mientras tanto, `src-tauri` solo se compila en CI y aquí se prueban el núcleo, la CLI y la interfaz en el navegador | Abierto · 2026-10-06. Lo arregla `sudo apt install build-essential cmake nasm pkg-config libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev webp` |
| **Ninguna compilación se ha abierto en un Mac, un Windows ni un Debian** | Media | Los instaladores salen de CI; que arranquen solo se sabe abriéndolos | Abierto · 2026-10-06 |

## Técnica

| Elemento | Severidad | Impacto | Estado |
|---|---|---|---|
| **El icono es provisional** | Baja | Un sol trazado en SVG para que haya algo en el paquete. El definitivo es parte de la entrega 6 | Abierto · 2026-10-06 |

## De producto

*(Nada todavía.)*

## Saldada

*(Nada todavía.)*
