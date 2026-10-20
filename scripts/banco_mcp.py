#!/usr/bin/env python3
"""El caso del MCP en el banco (scripts/banco.sh): `comparar_formatos` con una foto de 24 MP,
por las tuberías, como lo pide un agente. Se mide con /usr/bin/time desde fuera, y el pico es
el de este proceso o el de apolo-mcp, el mayor (GNU time da el del hijo más grande).

    banco_mcp.py <apolo-mcp> <foto> <carpeta de salida>
"""
import json
import subprocess
import sys

binario, foto, salida = sys.argv[1:4]
p = subprocess.Popen([binario], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                     stderr=subprocess.DEVNULL)
n = 0


def pedir(metodo, params):
    global n
    n += 1
    p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": n, "method": metodo,
                               "params": params}) + "\n").encode())
    p.stdin.flush()
    while True:
        r = json.loads(p.stdout.readline())
        if r.get("id") == n:
            return r.get("result", r)


pedir("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                     "clientInfo": {"name": "banco", "version": "1"}})
p.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
p.stdin.flush()
r = pedir("tools/call", {"name": "comparar_formatos", "arguments": {
    "ruta": foto, "formatos": ["webp", "avif", "jpeg"], "salida": salida}})
p.stdin.close()
p.wait(timeout=60)
if r.get("isError") or "structuredContent" not in r:
    sys.exit(f"comparar_formatos falló: {json.dumps(r, ensure_ascii=False)[:400]}")
print(json.dumps(r["structuredContent"].get("resumen", ""), ensure_ascii=False))
