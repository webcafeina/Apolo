#!/usr/bin/env python3
"""Libera disco en target/ sin perder nada que no se rehaga solo (`make limpiar`).

Cada cambio de dependencias o de opciones deja en `deps/` una copia nueva de lo mismo y no borra la anterior:
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
        deps = os.path.join(RAIZ, perfil, "deps")
        if os.path.isdir(deps):
            grupos = collections.defaultdict(list)
            for f in os.listdir(deps):
                m = NOMBRE.match(f)
                if not m:
                    continue
                ruta = os.path.join(deps, f)
                try:
                    grupos[(m.group(1), m.group(3) or "")].append((os.lstat(ruta).st_mtime, ruta))
                except OSError:
                    continue
            for copias in grupos.values():
                copias.sort(reverse=True)
                for _, ruta in copias[1:]:
                    total += borrar(ruta, ver)
                    ficheros += 1
        incremental = os.path.join(RAIZ, perfil, "incremental")
        if perfil.endswith("debug") and os.path.isdir(incremental):
            total += borrar(incremental, ver)
    verbo = "Liberaría" if ver else "Liberados"
    print(f"{verbo} {total / 1e9:.1f} GB ({ficheros} copias viejas en deps/ y la caché incremental de depuración).")


if __name__ == "__main__":
    main()
