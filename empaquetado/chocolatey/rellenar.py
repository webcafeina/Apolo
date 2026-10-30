#!/usr/bin/env python3
"""Rellena el paquete apolo de Chocolatey con una versión y las sumas de sus instaladores.

    rellenar.py <versión> <SHA256SUMS.txt> <carpeta de salida>

Escribe apolo.nuspec y tools/*.ps1 en la carpeta de salida, listos para `choco pack`. Solo cambia
{{VERSION}} y las sumas {{SHA_*}}, y quita el comentario de cada plantilla (hasta la primera línea
en blanco). Falla si falta una suma, para no subir a Chocolatey un paquete a medias. Solo usa la
biblioteca estándar.
"""
import sys
from pathlib import Path

AQUI = Path(__file__).resolve().parent
PLANTILLAS = ["apolo.nuspec", "tools/chocolateyinstall.ps1", "tools/chocolateyuninstall.ps1"]


def sumas(ruta, version):
    tabla = {}
    for linea in Path(ruta).read_text().splitlines():
        partes = linea.split()
        if len(partes) == 2:
            tabla[partes[1].lstrip("*")] = partes[0]
    ficheros = {
        "SHA_X64": f"Apolo_{version}_x64-setup.exe",
        "SHA_ARM64": f"Apolo_{version}_arm64-setup.exe",
    }
    faltan = [f for f in ficheros.values() if f not in tabla]
    if faltan:
        sys.exit(f"Faltan sumas en {ruta}: {', '.join(faltan)}")
    return {clave: tabla[f] for clave, f in ficheros.items()}


def main():
    version, ruta_sumas, salida = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    valores = {"VERSION": version, **sumas(ruta_sumas, version)}
    for nombre in PLANTILLAS:
        texto = (AQUI / nombre).read_text(encoding="utf-8")
        texto = texto.split("\n\n", 1)[1]
        for clave, valor in valores.items():
            texto = texto.replace("{{" + clave + "}}", valor)
        if "{{" in texto:
            sys.exit(f"Queda algo sin rellenar en {nombre}")
        destino = salida / nombre
        destino.parent.mkdir(parents=True, exist_ok=True)
        destino.write_text(texto, encoding="utf-8")
    print(f"{salida}: apolo {version}")


if __name__ == "__main__":
    main()
