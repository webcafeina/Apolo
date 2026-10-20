#!/usr/bin/env python3
"""Pruebas de humo de lo publicado (entrega 8a, ADR 0024), en las máquinas de CI de cada sistema.

Lo que nadie ha visto en un equipo: Windows x64 y Linux x64. Con los binarios ya empaquetados:

- todo en una carpeta **con tildes y eñes**, también la temporal (TEMP, TMP, TMPDIR): avifenc y
  cjxl abren sus ficheros con fopen, que en Windows no usa UTF-8 (deuda);
- la CLI frente a los binarios oficiales (cwebp, avifenc, cjxl) si se le dan, byte a byte;
- en Windows, que ningún .exe pida las DLL de Visual C++ (MSVCP140.dll…), que las máquinas de CI
  tienen y un Windows recién instalado no, y que la ventana arranque;
- `apolo-mcp` por sus tuberías: initialize, las once herramientas, optimizar a AVIF y JPEG XL;
- con --reinstalar (Windows): un apolo-mcp.exe en marcha, como el de un agente abierto, y el
  instalador otra vez encima: su gancho tiene que cerrarlo y el instalador terminar bien.

    python humo.py --cli DIR [--mcp RUTA ...] [--cwebp RUTA] [--avifenc RUTA] [--cjxl RUTA]
                   [--reinstalar INSTALADOR --instalado RUTA] [--solo-informar-equivalencia]

Sale con 1 si algo falla. Solo usa la biblioteca estándar.
"""
import argparse
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
CORPUS = RAIZ / "pruebas" / "corpus"
EXE = ".exe" if os.name == "nt" else ""

fallos = []
avisos = []

# La consola de Windows es cp1252 y no sabe escribir «✓»: en UTF-8, siempre.
for flujo in (sys.stdout, sys.stderr):
    try:
        flujo.reconfigure(encoding="utf-8")
    except AttributeError:
        pass


def bien(msg):
    print(f"  ✓ {msg}", flush=True)


def mal(msg):
    print(f"  ✗ {msg}", flush=True)
    fallos.append(msg)


def correr(orden, entorno, cwd=None):
    return subprocess.run([str(x) for x in orden], env=entorno, cwd=cwd, capture_output=True, timeout=600)


def preparar(cli, entorno):
    """La carpeta de trabajo con tildes, y las entradas: PNG, JPEG y la girada."""
    base = Path(tempfile.gettempdir()) / "Pruebas de Ñandú" / "árbol con espacios"
    shutil.rmtree(base, ignore_errors=True)
    tmp = base / "temporal ñ"
    tmp.mkdir(parents=True)
    for v in ("TEMP", "TMP", "TMPDIR"):
        entorno[v] = str(tmp)
    shutil.copy(CORPUS / "foto.webp", base / "foto.webp")
    shutil.copy(CORPUS / "orientacion-6.png", base / "girada.png")
    apolo = cli / f"apolo{EXE}"
    for orden, salida in (([apolo, "png", "foto.webp", "--out", "foto.png"], "foto.png"),
                          ([apolo, "jpeg", "-quality", "92", "foto.webp", "-outfile", "foto.jpg"], "foto.jpg")):
        r = correr(orden, entorno, cwd=base)
        if r.returncode != 0 or not (base / salida).exists():
            mal(f"preparar {salida}: {r.stderr.decode(errors='replace')[-300:]}")
    return base


def equivalencia(cli, base, entorno, oficiales, solo_informar):
    print("Equivalencia con las herramientas oficiales, en una ruta con tildes:")
    casos = {
        "cwebp": ("webp", ["", "-q 50", "-lossless", "-m 6 -q 80"], ["foto.png", "foto.jpg"]),
        "avifenc": ("avif", ["", "-q 40 -s 9", "-l -s 9"], ["foto.png", "foto.jpg"]),
        "cjxl": ("jxl", ["", "-d 1 -e 7", "-q 90"], ["foto.png"]),
        # Un JPEG, cjxl lo recomprime sin pérdida por defecto; con pérdida hay que apagarlo.
        "cjxl-jpeg": ("jxl", ["", "--lossless_jpeg=0 -d 1"], ["foto.jpg"]),
    }
    if oficiales.get("cjxl"):
        oficiales["cjxl-jpeg"] = oficiales["cjxl"]
    for herramienta, ruta in oficiales.items():
        if not ruta:
            continue
        sub, opciones, entradas = casos[herramienta]
        iguales = distintos = 0
        for e in entradas:
            for o in opciones:
                a, b = base / "oficial.out", base / "apolo.out"
                for f in (a, b):
                    f.unlink(missing_ok=True)
                op = o.split()
                if herramienta == "cwebp":
                    r1 = correr([ruta, *op, e, "-o", a], entorno, cwd=base)
                    r2 = correr([cli / f"apolo{EXE}", sub, *op, e, "-o", b], entorno, cwd=base)
                else:
                    r1 = correr([ruta, *op, e, a], entorno, cwd=base)
                    r2 = correr([cli / f"apolo{EXE}", sub, *op, e, b], entorno, cwd=base)
                if r2.returncode != 0 or not b.exists():
                    mal(f"apolo {sub} {o} {e} falló: {r2.stderr.decode(errors='replace')[-300:]}")
                    continue
                if r1.returncode != 0 or not a.exists():
                    avisos.append(f"{herramienta} oficial {o} {e} no dio fichero (¿no lee esa entrada?)")
                    continue
                if a.read_bytes() == b.read_bytes():
                    iguales += 1
                else:
                    distintos += 1
                    print(f"    distinto: {herramienta} {o or '(por defecto)'} {e}")
        msg = f"{herramienta}: {iguales} iguales, {distintos} distintos"
        if distintos and not solo_informar:
            mal(msg)
        elif distintos:
            avisos.append(msg)
            print(f"  ! {msg}")
        else:
            bien(msg)


class Mcp:
    """Un apolo-mcp en marcha, con quien se habla por sus tuberías."""

    def __init__(self, binario, entorno):
        self.p = subprocess.Popen([str(binario)], env=entorno, stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.n = 0

    def pedir(self, metodo, params=None):
        self.n += 1
        m = {"jsonrpc": "2.0", "id": self.n, "method": metodo, "params": params or {}}
        self.p.stdin.write((json.dumps(m) + "\n").encode())
        self.p.stdin.flush()
        while True:
            linea = self.p.stdout.readline()
            if not linea:
                raise RuntimeError("apolo-mcp cerró la salida")
            r = json.loads(linea)  # cualquier línea que no sea JSON-RPC rompe aquí
            if r.get("id") == self.n:
                return r.get("result", r)

    def avisar(self, metodo):
        self.p.stdin.write((json.dumps({"jsonrpc": "2.0", "method": metodo}) + "\n").encode())
        self.p.stdin.flush()

    def cerrar(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=20)
        except Exception:
            self.p.kill()


def mcp(binario, base, entorno):
    print(f"apolo-mcp: {binario}")
    if not Path(binario).exists():
        mal(f"no existe {binario}")
        return
    e = dict(entorno, APOLO_CONFIG=str(base / "config ñ"))
    m = Mcp(binario, e)
    try:
        ini = m.pedir("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                     "clientInfo": {"name": "humo", "version": "1"}})
        m.avisar("notifications/initialized")
        v = ini.get("serverInfo", {}).get("version")
        bien(f"initialize: apolo {v}") if v else mal(f"initialize: {ini}")
        n = len(m.pedir("tools/list").get("tools", []))
        bien(f"{n} herramientas") if n == 11 else mal(f"{n} herramientas, esperaba 11")
        for formato in ("avif", "jxl", "webp"):
            r = m.pedir("tools/call", {"name": "optimizar_imagen", "arguments": {
                "ruta": str(base / "foto.png"), "formato": formato, "calidad": 70,
                "salida": str(base / "salidas ñ")}})
            sal = (r.get("structuredContent") or {}).get("salida")
            if r.get("isError") or not sal or not Path(sal).exists():
                mal(f"optimizar a {formato}: {json.dumps(r, ensure_ascii=False)[:400]}")
            else:
                bien(f"optimizar a {formato} → {Path(sal).name}")
        r = m.pedir("tools/call", {"name": "acerca_de_apolo", "arguments": {}})
        bien("acerca_de_apolo") if not r.get("isError") else mal(f"acerca_de_apolo: {r}")
    except Exception as x:  # noqa: BLE001
        mal(f"apolo-mcp: {x}")
    finally:
        m.cerrar()


# Las DLL de «Visual C++ Redistributable»: un Windows recién instalado no las
# trae, y las máquinas de CI sí, así que un .exe que las pida pasa aquí y falla
# en casa (MSVCP140.dll, 2026-10-19). Se mira lo que el .exe importa.
DLL_DE_VC = ("msvcp", "vcruntime", "concrt", "vccorlib")


def dlls_importadas(ruta):
    """Los nombres de las DLL que importa un .exe de Windows (PE32+)."""
    d = Path(ruta).read_bytes()
    u16 = lambda o: struct.unpack_from("<H", d, o)[0]  # noqa: E731
    u32 = lambda o: struct.unpack_from("<I", d, o)[0]  # noqa: E731
    pe = u32(0x3C)
    secciones, opcional = u16(pe + 6), u16(pe + 20)
    tabla = pe + 24 + opcional
    importaciones = u32(pe + 24 + 120)  # PE32+: directorio 1, el de importaciones

    def desplazamiento(rva):
        for i in range(secciones):
            s = tabla + 40 * i
            va, tam, crudo = u32(s + 12), max(u32(s + 8), u32(s + 16)), u32(s + 20)
            if va <= rva < va + tam:
                return rva - va + crudo
        raise ValueError(f"RVA {rva:#x} fuera de las secciones")

    nombres, o = [], desplazamiento(importaciones)
    while u32(o + 12):
        n = desplazamiento(u32(o + 12))
        nombres.append(d[n:d.index(b"\0", n)].decode())
        o += 20
    return nombres


def sin_dll_de_vc(rutas):
    print("Sin las DLL de Visual C++:")
    for r in rutas:
        try:
            dlls = dlls_importadas(r)
        except Exception as x:  # noqa: BLE001
            mal(f"{Path(r).name}: no se pudo leer ({x})")
            continue
        pide = [n for n in dlls if n.lower().startswith(DLL_DE_VC)]
        if pide:
            mal(f"{Path(r).name} pide {', '.join(pide)}")
        else:
            bien(f"{Path(r).name}: {', '.join(sorted(dlls, key=str.lower))}")


def abrir_ventana(app, entorno):
    """Que la ventana arranque y siga en marcha unos segundos. Un manifiesto roto no deja ni
    crear el proceso («la configuración en paralelo no es correcta», 2026-10-20)."""
    print(f"La ventana: {app}")
    try:
        p = subprocess.Popen([str(app)], env=entorno)
    except OSError as x:
        mal(f"no arranca: {x}")
        return
    time.sleep(10)
    if p.poll() is None:
        bien("arranca y sigue abierta")
        p.kill()
    else:
        mal(f"se cerró sola con el código {p.returncode}")


def reinstalar(instalador, instalado, base, entorno):
    """Un apolo-mcp.exe en marcha (un agente abierto) y el instalador encima, en silencio."""
    print("Actualizar con un agente abierto:")
    m = Mcp(instalado, dict(entorno, APOLO_CONFIG=str(base / "config ñ")))
    m.pedir("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                           "clientInfo": {"name": "humo", "version": "1"}})
    antes = os.path.getmtime(instalado)
    r = correr([instalador, "/S"], entorno)
    time.sleep(3)
    if r.returncode != 0:
        mal(f"el instalador terminó con {r.returncode}")
    else:
        bien("el instalador terminó bien con apolo-mcp.exe en marcha")
    if m.p.poll() is None:
        mal("apolo-mcp.exe sigue en marcha: el gancho no lo cerró")
        m.p.kill()
    else:
        bien("el gancho cerró apolo-mcp.exe")
    if os.path.getmtime(instalado) >= antes and Path(instalado).exists():
        bien("apolo-mcp.exe está, reemplazado")
    else:
        mal("apolo-mcp.exe no se reemplazó")


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--cli", required=True, type=Path)
    a.add_argument("--mcp", action="append", default=[])
    a.add_argument("--cwebp")
    a.add_argument("--avifenc")
    a.add_argument("--cjxl")
    a.add_argument("--reinstalar")
    a.add_argument("--instalado")
    a.add_argument("--solo-informar-equivalencia", action="store_true")
    o = a.parse_args()
    # Absolutas: las órdenes corren dentro de la carpeta de trabajo.
    o.cli = o.cli.resolve()
    for k in ("cwebp", "avifenc", "cjxl", "reinstalar", "instalado"):
        if getattr(o, k):
            setattr(o, k, str(Path(getattr(o, k)).resolve()))
    o.mcp = [str(Path(x).resolve()) for x in o.mcp]
    entorno = dict(os.environ)
    base = preparar(o.cli, entorno)
    print(f"Carpeta de trabajo: {base}")
    equivalencia(o.cli, base, entorno, {"cwebp": o.cwebp, "avifenc": o.avifenc, "cjxl": o.cjxl},
                 o.solo_informar_equivalencia)
    if os.name == "nt":
        exes = [o.cli / "apolo.exe", o.cli / "apolo-mcp.exe", *o.mcp]
        if o.instalado:
            exes.append(Path(o.instalado).parent / "apolo-app.exe")
        sin_dll_de_vc(exes)
        if o.instalado:
            abrir_ventana(Path(o.instalado).parent / "apolo-app.exe", entorno)
    for b in [str(o.cli / f"apolo-mcp{EXE}"), *o.mcp]:
        mcp(b, base, entorno)
    if o.reinstalar:
        reinstalar(o.reinstalar, o.instalado, base, entorno)
    print()
    for x in avisos:
        print(f"AVISO: {x}")
    if fallos:
        print(f"{len(fallos)} fallos")
        sys.exit(1)
    print("Todo bien")


if __name__ == "__main__":
    main()
