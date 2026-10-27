# ADR 0035 — Copias de seguridad

**Fecha:** 2026-10-27 · **Estado:** aceptada · antes de la 1.0.0

## Contexto

Antes de la 1.0.0 el cliente preguntó si había copias de seguridad de todo. No las había: el código
y las versiones vivían solo en GitHub (y el código, también en el clon del VPS), y **las claves de
firma solo eran legibles en el VPS** (deuda.md, ADR 0018). Perder la del actualizador deja a todas
las Apolo instaladas sin poder aceptar una versión nueva; perder la GPG obliga a los usuarios de APT
y RPM a importar otra. Con el lanzamiento, más gente dependerá de ellas.

## Decisión

Con el cliente: copias en **Cloudflare R2** (bucket `apolo-copias`, como los demás proyectos del
VPS) y la frase de las claves en su **Coda**; todo antes de la 1.0.0. Se hace con lo que ya usa la
casa: rclone en un contenedor efímero, credenciales en un fichero 600 y avisos de healthchecks.io.

- **Las claves**: `scripts/copias/claves.sh` las cifra con GPG (AES-256, frase aleatoria de 64
  caracteres hex), las sube, las vuelve a bajar y las compara. A mano, y cada vez que cambie una.
- **La copia semanal** (`semanal.sh`, domingos 04:00): los dos repositorios como git bundle (4
  semanas), los motores de terceros en su commit (una vez por commit) y las 3 últimas versiones con
  sus sumas comprobadas.
- **El ensayo mensual** (`ensayo.sh`, día 1 a las 05:00): restaura desde R2 y lo comprueba.
- **`docs/restaurar.md`**: qué hay, dónde y cómo se recupera cada cosa.

## Alternativas descartadas

- **Otra cuenta o servicio de git como espejo**: no guarda las Releases ni las claves, y suma otra
  cuenta que cuidar.
- **Las claves solo en el gestor de contraseñas del cliente**: son ficheros (la GPG, 3 KB), no
  contraseñas; mejor un archivo cifrado y la frase aparte.
- **Copiar las dependencias de Rust y npm**: están fijadas por los lockfiles en crates.io y npm, que
  no borran versiones publicadas; el coste no compensa.

## Consecuencias

- Unos 1,8 GB en R2 (la capa gratuita es de 10 GB). Crece con el tamaño de cada versión.
- El token de R2 solo puede tocar ese bucket (comprobado: 403 en cualquier otro).
- Las claves siguen legibles en el VPS: perderlo ya no las pierde, pero quien entre en él las tiene.
- `HOMEBREW_TAP_CLAVE` no se copia: es una deploy key y se crea otra.

## Verificación

- 2026-10-27: `claves.sh` (copia descifrada e idéntica), `semanal.sh` (108 ficheros, 1,78 GB) y
  `ensayo.sh` (repositorio clonado con 33 etiquetas, claves idénticas, sumas de la v0.18.3), todo en
  verde; el token, sin acceso a otros buckets.
- 2026-10-27, el cliente: la frase de Coda abre la copia en su Mac, healthchecks.io recibió el
  aviso y el contenedor de Tag Manager está exportado.
