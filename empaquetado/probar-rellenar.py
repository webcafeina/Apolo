#!/usr/bin/env python3
"""Pasa los tres rellenar.py (AUR, Chocolatey y Homebrew) por sus plantillas de verdad, sin red.

Con una versión inventada y un SHA256SUMS.txt con los nombres que tendría una Release de esa versión y una suma
distinta para cada fichero, cada guion tiene que dejar su salida sin ningún marcador {{…}} (salvo {{appdir}}, que
es de Homebrew), con la versión y con todas las sumas que le tocan. Va en `make comprobar` (`make empaquetado`):
una plantilla con un marcador nuevo que nadie rellena se descubriría si no al publicar, en el tap, en el AUR o en
la moderación de Chocolatey. Lo propuso la sesión de Esfinge (docs/siguiente.md), donde la primera prueba así
cazó un fallo en el acto.
"""

import hashlib
import re
import subprocess
import sys
import tempfile
from pathlib import Path

RAIZ = Path(__file__).resolve().parent
VERSION = "9.8.7"
# Los ficheros de una Release que leen los guiones (los demás no hacen falta).
FICHEROS = [
    f"Apolo_{VERSION}_amd64.deb",
    f"Apolo_{VERSION}_arm64.deb",
    f"Apolo_{VERSION}_x64-setup.exe",
    f"Apolo_{VERSION}_arm64-setup.exe",
    f"Apolo_{VERSION}_universal.dmg",
    f"apolo-cli-{VERSION}-macos-universal.tar.gz",
    f"apolo-cli-{VERSION}-linux-amd64.tar.gz",
    f"apolo-cli-{VERSION}-linux-arm64.tar.gz",
]
# Qué sumas tiene que llevar cada salida.
ESPERADO = {
    "aur/PKGBUILD": [f"Apolo_{VERSION}_amd64.deb", f"Apolo_{VERSION}_arm64.deb"],
    "chocolatey/tools/chocolateyinstall.ps1": [f"Apolo_{VERSION}_x64-setup.exe", f"Apolo_{VERSION}_arm64-setup.exe"],
    "homebrew/Casks/apolo.rb": [f"Apolo_{VERSION}_universal.dmg"],
    "homebrew/Formula/apolo.rb": [
        f"apolo-cli-{VERSION}-macos-universal.tar.gz",
        f"apolo-cli-{VERSION}-linux-amd64.tar.gz",
        f"apolo-cli-{VERSION}-linux-arm64.tar.gz",
    ],
}


def main() -> None:
    malos = []
    with tempfile.TemporaryDirectory() as d:
        d = Path(d)
        sumas = {f: hashlib.sha256(f.encode()).hexdigest() for f in FICHEROS}
        (d / "SHA256SUMS.txt").write_text("".join(f"{s}  {f}\n" for f, s in sumas.items()))
        ordenes = [
            ["aur/rellenar.py", VERSION, d / "SHA256SUMS.txt", d / "aur/PKGBUILD"],
            ["chocolatey/rellenar.py", VERSION, d / "SHA256SUMS.txt", d / "chocolatey"],
            ["homebrew/rellenar.py", VERSION, d / "SHA256SUMS.txt", d / "homebrew"],
        ]
        (d / "aur").mkdir()
        for guion, *args in ordenes:
            r = subprocess.run(
                [sys.executable, "-I", str(RAIZ / guion), *map(str, args)], capture_output=True, text=True
            )
            if r.returncode != 0:
                malos.append(f"{guion}: salió con {r.returncode}: {r.stderr.strip() or r.stdout.strip()}")
        for salida, ficheros in ESPERADO.items():
            ruta = d / salida
            if not ruta.exists():
                malos.append(f"{salida}: no se escribió")
                continue
            texto = ruta.read_text()
            sobran = [m for m in re.findall(r"\{\{[^}]*\}\}", texto) if m != "{{appdir}}"]
            if sobran:
                malos.append(f"{salida}: sin rellenar {sorted(set(sobran))}")
            if VERSION not in texto:
                malos.append(f"{salida}: sin la versión {VERSION}")
            for f in ficheros:
                if sumas[f] not in texto:
                    malos.append(f"{salida}: sin la suma de {f}")
        nuspec = d / "chocolatey/apolo.nuspec"
        if not nuspec.exists() or f"<version>{VERSION}</version>" not in nuspec.read_text():
            malos.append("chocolatey/apolo.nuspec: sin la versión")
    if malos:
        sys.exit("Los rellenar.py fallan:\n  " + "\n  ".join(malos))
    print(f"rellenar.py: AUR, Chocolatey y Homebrew, bien con la {VERSION}")


if __name__ == "__main__":
    main()
