#!/usr/bin/env python3
"""Rellena el PKGBUILD de apolo-bin (ADR 0030) con una versión y las sumas de sus .deb.

    rellenar.py <versión> <SHA256SUMS.txt> <PKGBUILD de salida>

Solo cambia {{VERSION}} y las sumas {{SHA_*}}, y quita el comentario de la plantilla (hasta la
primera línea en blanco). Falla si falta una suma, para no subir al AUR un paquete a medias. Solo
usa la biblioteca estándar.
"""
import sys
from pathlib import Path

AQUI = Path(__file__).resolve().parent


def sumas(ruta, version):
    tabla = {}
    for linea in Path(ruta).read_text().splitlines():
        partes = linea.split()
        if len(partes) == 2:
            tabla[partes[1].lstrip("*")] = partes[0]
    ficheros = {
        "SHA_DEB_AMD64": f"Apolo_{version}_amd64.deb",
        "SHA_DEB_ARM64": f"Apolo_{version}_arm64.deb",
    }
    faltan = [f for f in ficheros.values() if f not in tabla]
    if faltan:
        sys.exit(f"Faltan sumas en {ruta}: {', '.join(faltan)}")
    return {clave: tabla[f] for clave, f in ficheros.items()}


def main():
    version, ruta_sumas, salida = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    valores = {"VERSION": version, **sumas(ruta_sumas, version)}
    texto = (AQUI / "PKGBUILD").read_text()
    texto = texto.split("\n\n", 1)[1]
    for clave, valor in valores.items():
        texto = texto.replace("{{" + clave + "}}", valor)
    if "{{" in texto:
        sys.exit("Queda algo sin rellenar en el PKGBUILD")
    salida.parent.mkdir(parents=True, exist_ok=True)
    salida.write_text(texto)
    print(f"{salida}: apolo-bin {version}")


if __name__ == "__main__":
    main()
