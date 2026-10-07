"""Descomprime un .lz (lzip) a la salida estándar, para cuando no está lzip.

El cjxl oficial de Linux viene en un .tar.lz. lzip es LZMA con una cabecera
propia: «LZIP», la versión y el tamaño del diccionario codificado en un byte;
detrás, el flujo LZMA (lc=3, lp=0, pb=2) con marca de fin, y 20 bytes de cola
(CRC32, tamaño de los datos y del miembro). Solo la biblioteca estándar.

    python3 -I deslzip.py fichero.tar.lz > fichero.tar
"""

import lzma
import struct
import sys
import zlib


def miembros(datos):
    pos = 0
    while pos < len(datos):
        if datos[pos : pos + 4] != b"LZIP" or datos[pos + 4] != 1:
            sys.exit("no es un fichero lzip de la versión 1")
        b = datos[pos + 5]
        base = 1 << (b & 0x1F)
        diccionario = base - (b >> 5) * (base // 16)
        filtro = {"id": lzma.FILTER_LZMA1, "dict_size": diccionario, "lc": 3, "lp": 0, "pb": 2}
        d = lzma.LZMADecompressor(format=lzma.FORMAT_RAW, filters=[filtro])
        salida = d.decompress(datos[pos + 6 :])
        cola = d.unused_data
        crc, tamano, miembro = struct.unpack("<IQQ", cola[:20])
        if zlib.crc32(salida) != crc or len(salida) != tamano:
            sys.exit("el CRC o el tamaño no cuadran")
        yield salida
        pos += miembro


def main():
    datos = open(sys.argv[1], "rb").read()
    for parte in miembros(datos):
        sys.stdout.buffer.write(parte)


main()
