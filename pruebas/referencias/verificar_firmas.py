# Revisa el borrador de una Release antes de publicarlo (ADR 0018): las firmas
# minisign de Tauri (cada .sig) con la clave pública de tauri.conf.json, y que
# latest.json apunte, para cada plataforma, a un fichero de esa Release con su
# firma. Necesita el paquete cryptography.
#
#     gh release download vX.Y.Z -D carpeta
#     python3 -I pruebas/referencias/verificar_firmas.py src-tauri/tauri.conf.json carpeta
import base64, hashlib, json, sys, os
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

conf, carpeta = sys.argv[1], sys.argv[2]
pub = base64.b64decode(json.load(open(conf))["plugins"]["updater"]["pubkey"]).decode().splitlines()[1]
pk = base64.b64decode(pub)
clave_id, clave = pk[2:10], Ed25519PublicKey.from_public_bytes(pk[10:42])

def comprobar(fichero, firma_b64):
    lineas = base64.b64decode(firma_b64).decode().splitlines()
    s = base64.b64decode(lineas[1]); alg, kid, sig = s[:2], s[2:10], s[10:74]
    assert kid == clave_id, "otra clave"
    datos = open(fichero, "rb").read()
    msg = hashlib.blake2b(datos, digest_size=64).digest() if alg == b"ED" else datos
    clave.verify(sig, msg)
    comentario = lineas[2].split(": ", 1)[1].encode()
    clave.verify(base64.b64decode(lineas[3]), sig + comentario)

n = 0
for f in sorted(os.listdir(carpeta)):
    if f.endswith(".sig"):
        comprobar(os.path.join(carpeta, f[:-4]), open(os.path.join(carpeta, f)).read()); n += 1
print(f"{n} .sig válidas")
j = json.load(open(os.path.join(carpeta, "latest.json")))
print("versión", j["version"])
for p, d in j["platforms"].items():
    nombre = d["url"].rsplit("/", 1)[1]
    assert "/releases/download/v" + j["version"] + "/" in d["url"], d["url"]
    comprobar(os.path.join(carpeta, nombre), d["signature"])
print(f"latest.json: {len(j['platforms'])} plataformas, todas con firma válida y fichero en la Release")
