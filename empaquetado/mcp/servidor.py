#!/usr/bin/env python3
# El server.json del registro oficial de MCP (ADR 0034): Apolo como servidor
# MCP, con los cinco .mcpb de una versión publicada y sus sumas. Lo usa
# .github/workflows/mcp.yml al publicar; a mano:
#   python3 empaquetado/mcp/servidor.py 1.0.0 SHA256SUMS.txt > server.json
import json
import sys

REPO = "https://github.com/webcafeina/Apolo"
# Los mismos que publicar.yml sube a la Release (y que release.ts espera en la web).
SISTEMAS = ["macos", "windows-x64", "windows-arm64", "linux-amd64", "linux-arm64"]


def servidor(version: str, sumas: dict[str, str]) -> dict:
    paquetes = []
    for s in SISTEMAS:
        nombre = f"Apolo-{version}-{s}.mcpb"
        if nombre not in sumas:
            sys.exit(f"Falta {nombre} en SHA256SUMS.txt")
        paquetes.append(
            {
                "registryType": "mcpb",
                "identifier": f"{REPO}/releases/download/v{version}/{nombre}",
                "version": version,
                "fileSha256": sumas[nombre],
                "transport": {"type": "stdio"},
            }
        )
    return {
        "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
        "name": "io.github.webcafeina/apolo",
        "title": "Apolo",
        "description": "Optimize, convert and compare images locally: WebP, AVIF, JPEG XL, JPEG and PNG.",
        "version": version,
        "websiteUrl": "https://apolo.webcafeina.com/en/agents/",
        "repository": {"url": REPO, "source": "github"},
        "icons": [{"src": "https://apolo.webcafeina.com/icono.png", "mimeType": "image/png", "sizes": ["256x256"]}],
        "packages": paquetes,
    }


def leer_sumas(fichero: str) -> dict[str, str]:
    sumas = {}
    for linea in open(fichero, encoding="utf-8"):
        partes = linea.split()
        if len(partes) == 2:
            sumas[partes[1].lstrip("*")] = partes[0]
    return sumas


if __name__ == "__main__":
    version, fichero = sys.argv[1], sys.argv[2]
    print(json.dumps(servidor(version, leer_sumas(fichero)), indent=2, ensure_ascii=False))
