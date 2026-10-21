#!/usr/bin/env python3
"""Rellena la fórmula y el cask de Homebrew (ADR 0026) con una versión y sus sumas.

    rellenar.py <versión> <SHA256SUMS.txt> <carpeta del tap>

Escribe <tap>/Formula/apolo.rb y <tap>/Casks/apolo.rb desde las plantillas de esta carpeta. Solo
cambia {{VERSION}} y las sumas {{SHA_*}}: el {{appdir}} del cask es de Homebrew y se queda. Falla si
falta una suma, para no publicar un tap a medias. Solo usa la biblioteca estándar.
"""
import sys
from pathlib import Path

AQUI = Path(__file__).resolve().parent
CABECERA = (
    "# Generado por publicar.yml de webcafeina/Apolo desde empaquetado/homebrew/ (ADR 0026).\n"
    "# No se edita aquí: se cambia la plantilla en Apolo.\n"
)


def sumas(ruta, version):
    tabla = {}
    for linea in Path(ruta).read_text().splitlines():
        partes = linea.split()
        if len(partes) == 2:
            tabla[partes[1].lstrip("*")] = partes[0]
    ficheros = {
        "SHA_DMG": f"Apolo_{version}_universal.dmg",
        "SHA_MACOS": f"apolo-cli-{version}-macos-universal.tar.gz",
        "SHA_LINUX_AMD64": f"apolo-cli-{version}-linux-amd64.tar.gz",
        "SHA_LINUX_ARM64": f"apolo-cli-{version}-linux-arm64.tar.gz",
    }
    faltan = [f for f in ficheros.values() if f not in tabla]
    if faltan:
        sys.exit(f"Faltan sumas en {ruta}: {', '.join(faltan)}")
    return {clave: tabla[f] for clave, f in ficheros.items()}


def rellenar(plantilla, valores):
    texto = (AQUI / plantilla).read_text()
    # Fuera el comentario de la plantilla (las primeras líneas con #): en el tap va la cabecera.
    lineas = texto.splitlines(keepends=True)
    while lineas and lineas[0].startswith("#"):
        lineas.pop(0)
    texto = CABECERA + "".join(lineas)
    for clave, valor in valores.items():
        texto = texto.replace("{{" + clave + "}}", valor)
    return texto


def main():
    version, ruta_sumas, tap = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    valores = {"VERSION": version, **sumas(ruta_sumas, version)}
    for plantilla, destino in [("apolo.rb", "Formula/apolo.rb"), ("apolo-cask.rb", "Casks/apolo.rb")]:
        (tap / destino).parent.mkdir(parents=True, exist_ok=True)
        (tap / destino).write_text(rellenar(plantilla, valores))
        print(f"{destino}: {version}")


if __name__ == "__main__":
    main()
