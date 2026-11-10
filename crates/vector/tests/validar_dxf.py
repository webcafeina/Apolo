"""Valida ficheros DXF con ezdxf (MIT; solo para las pruebas, no va en Apolo): que se lean, y que ni la
auditoría ni la recuperación encuentren errores ni tengan que arreglar nada. Lo llama tests/formatos.rs con
`uvx --with ezdxf==1.4.2 python -I`. Sale con 1 si algo falla."""

import sys

import ezdxf
from ezdxf import recover

malos = 0
for ruta in sys.argv[1:]:
    try:
        doc = ezdxf.readfile(ruta)
    except Exception as e:  # noqa: BLE001
        print(f"{ruta}: no se lee ({type(e).__name__}: {e})")
        malos += 1
        continue
    auditoria = doc.audit()
    _, rec = recover.readfile(ruta)
    problemas = [x.message for x in [*auditoria.errors, *auditoria.fixes, *rec.errors, *rec.fixes]]
    if problemas or len(doc.modelspace()) == 0:
        print(f"{ruta}: {problemas[:5] or 'sin entidades'}")
        malos += 1
print(f"ezdxf {ezdxf.__version__}: {len(sys.argv) - 1} ficheros, {malos} con problemas")
sys.exit(1 if malos else 0)
