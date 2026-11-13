#!/usr/bin/env python3
"""Libera disco en target/ sin perder nada que no se rehaga solo (`make limpiar`).

Cada cambio de dependencias o de opciones deja en `deps/` (y en `examples/`) una copia nueva de lo mismo y no borra la anterior:
en el VPS llegaron a ser decenas de GB (docs/deuda.md, «El disco del VPS se llena con target/»). Esto deja, de
cada nombre y extensión, solo la más reciente, y borra la caché incremental de depuración (la próxima
compilación tarda un poco más). No toca lo compilado de las herramientas de referencia ni las capturas.

Con --ver, solo dice cuánto liberaría."""

import collections
import os
import re
import shutil
import sys

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target")
# Las carpetas de compilación de cargo: la de depuración, la de release y la del banco de vectorizar.
PERFILES = ["debug", "release", "laboratorio/debug", "laboratorio/release"]
NOMBRE = re.compile(r"^(.*?)-([0-9a-f]{16})(\..*)?$")


def tamano(ruta: str) -> int:
    if os.path.isdir(ruta) and not os.path.islink(ruta):
        return sum(tamano(os.path.join(ruta, f)) for f in os.listdir(ruta))
    try:
        return os.lstat(ruta).st_size
    except OSError:
        return 0


def borrar(ruta: str, ver: bool) -> int:
    n = tamano(ruta)
    if not ver:
        if os.path.isdir(ruta) and not os.path.islink(ruta):
            shutil.rmtree(ruta, ignore_errors=True)
        else:
            try:
                os.remove(ruta)
            except OSError:
                return 0
    return n


def main() -> None:
    ver = "--ver" in sys.argv[1:]
    total, ficheros = 0, 0
    for perfil in PERFILES:
        # Y en examples/: los programas de ejemplo (rasterizar, banco_estudio…) también se copian de nuevo con
        # cada cambio; el 2026-11-13 había veinte de cada uno, 3,3 GB.
        for carpeta in ("deps", "examples"):
            total_c, ficheros_c = limpiar_copias(os.path.join(RAIZ, perfil, carpeta), ver)
            total += total_c
            ficheros += ficheros_c
        incremental = os.path.join(RAIZ, perfil, "incremental")
        if perfil.endswith("debug") and os.path.isdir(incremental):
            total += borrar(incremental, ver)
    verbo = "Liberaría" if ver else "Liberados"
    print(f"{verbo} {total / 1e9:.1f} GB ({ficheros} copias viejas en deps/ y examples/, y la caché incremental de depuración).")


def limpiar_copias(carpeta: str, ver: bool) -> tuple[int, int]:
    """De cada nombre y extensión de la carpeta, deja solo la copia más reciente."""
    total, ficheros = 0, 0
    if not os.path.isdir(carpeta):
        return total, ficheros
    grupos = collections.defaultdict(list)
    for f in os.listdir(carpeta):
        m = NOMBRE.match(f)
        if not m:
            continue
        ruta = os.path.join(carpeta, f)
        try:
            grupos[(m.group(1), m.group(3) or "")].append((os.lstat(ruta).st_mtime, ruta))
        except OSError:
            continue
    for copias in grupos.values():
        copias.sort(reverse=True)
        for _, ruta in copias[1:]:
            total += borrar(ruta, ver)
            ficheros += 1
    return total, ficheros


if __name__ == "__main__":
    main()
