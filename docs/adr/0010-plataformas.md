# ADR 0010 — Plataformas, con ARM

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

Mac, Windows y Linux (Debian), como Esfinge. Quedaba por decidir el mínimo de cada uno y si entraba ARM.

## Decisión

Seis objetivos:

| Sistema | Mínimo | Arquitecturas | Paquete |
|---|---|---|---|
| macOS | 11 Big Sur | universal (Apple Silicon + Intel) | `.dmg` |
| Windows | 10 | x64 y ARM64 | instalador NSIS |
| Linux | Debian 12 / Ubuntu 22.04 | amd64 y arm64 | `.deb` y AppImage |

Linux se compila en Ubuntu 22.04 para que la glibc pedida (2.35) sea la de Debian 12 o menor.

## Alternativas descartadas

- **Sin ARM** en Windows y Linux. Los portátiles Windows ARM ya se venden en serie, y GitHub da
  máquinas ARM gratis a los repositorios públicos.

## Consecuencias

- Cada códec tiene que compilar en los seis: algunos (aom, libjxl) traen ensamblador por arquitectura.
- macOS 11 es el mínimo de Tauri 2 con WebKit moderno.

## Verificación

Elegida con el cliente el 2026-10-06.

- 2026-10-06: `publicar.yml`, lanzado a mano, **compila y empaqueta los cinco trabajos** (seis
  objetivos: el de macOS es universal). El NSIS de Windows ocupa ~1 MB en x64 y en ARM64, el DMG ~4 MB, y el
  artefacto de Linux ~78 MB, porque la AppImage lleva webkit dentro. El `.deb` no lo lleva.
- **No verificado**: que alguno **arranque**. Ningún paquete se ha abierto todavía en su sistema.
