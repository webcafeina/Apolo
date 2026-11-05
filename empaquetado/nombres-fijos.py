#!/usr/bin/env python3
"""Las copias con nombre fijo de una Release: `Apolo_universal.dmg` junto a `Apolo_1.0.2_universal.dmg`.

    nombres-fijos.py <versión> <carpeta con los ficheros de la Release>

Escribe en la carpeta una copia sin número de cada instalador, paquete, CLI y `.mcpb`, y dice cuáles.
Así `https://github.com/webcafeina/Apolo/releases/latest/download/<nombre fijo>` sirve siempre la
última, y los directorios que piden un enlace directo (MacUpdate, Uptodown, Softpedia, Glama) no hay
que tocarlos en cada versión. Lo pidió el cliente el 2026-10-30.

Quedan fuera las firmas `.sig`, `latest.json` y `SHA256SUMS.txt`: el actualizador, la web, APT, RPM,
Homebrew, Chocolatey y el registro de MCP siguen con los nombres con versión.
"""
import shutil
import sys
from pathlib import Path


def fijo(nombre: str, version: str) -> str | None:
    """El nombre sin versión, o None si no lleva o no se copia."""
    if nombre.endswith(".sig") or nombre in ("latest.json", "SHA256SUMS.txt"):
        return None
    nuevo = (
        nombre.replace(f"_{version}_", "_")  # Apolo_1.0.2_universal.dmg, _x64-setup.exe, _amd64.deb, .AppImage
        .replace(f"-{version}-1.", "_")  # Apolo-1.0.2-1.x86_64.rpm → Apolo_x86_64.rpm
        .replace(f"-{version}-", "-")  # apolo-cli-1.0.2-linux-amd64.tar.gz, Apolo-1.0.2-macos.mcpb
    )
    return nuevo if nuevo != nombre else None


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__.strip().splitlines()[2].strip())
    version, carpeta = sys.argv[1].removeprefix("v"), Path(sys.argv[2])
    hechas: dict[str, str] = {}
    for f in sorted(carpeta.iterdir()):
        n = fijo(f.name, version)
        if n is None or not f.is_file():
            continue
        if n in hechas:
            sys.exit(f"{hechas[n]} y {f.name} darían el mismo nombre fijo, {n}")
        # Si ya estaba (al repetir la publicación), se pisa: es la copia de la misma versión.
        shutil.copyfile(f, carpeta / n)
        print(f"{f.name} → {n}")
        hechas[n] = f.name
    if not hechas:
        sys.exit(f"Ningún fichero de la {version} en {carpeta}")


if __name__ == "__main__":
    main()
