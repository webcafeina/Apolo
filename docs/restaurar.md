# Copias de seguridad y cómo restaurar

Qué se copia, dónde y cómo se recupera cada cosa si se pierde el VPS, la cuenta de GitHub o las dos.
Decidido con el cliente el 2026-10-27, antes de la 1.0.0: las copias van a **Cloudflare R2**, en el
bucket `apolo-copias`, y la frase de la copia de las claves está en **Coda**.

## Qué hay y dónde

| Qué | Dónde | Cuándo |
|---|---|---|
| **Las claves de firma** (actualizador, GPG de los paquetes, SSH del AUR): `~/.config/apolo/claves/` cifrado con GPG (AES-256) | `r2:apolo-copias/claves/apolo-claves-AAAA-MM-DD.tar.gpg`, y una copia en el ordenador del cliente | A mano, con `scripts/copias/claves.sh`: la primera el 2026-10-27, y otra cada vez que cambie una clave |
| **La frase de esa copia** | Coda, en un documento privado del cliente (y en `~/.config/apolo/claves-copia.frase`) | Una vez |
| **Los repositorios**: Apolo y el tap de Homebrew, enteros (todas las ramas y etiquetas) | `r2:apolo-copias/repos/AAAA-MM-DD/*.bundle`, las 4 últimas semanas | Los domingos a las 04:00, `scripts/copias/semanal.sh` |
| **Los motores de terceros** (submódulos: libheif, aom, libjxl…), en el commit que usa Apolo | `r2:apolo-copias/motores/<nombre>-<commit>.tar.gz` | En la copia semanal, solo cuando cambia un commit |
| **Las versiones publicadas**: los ficheros de las 3 últimas, comprobados con su `SHA256SUMS.txt` | `r2:apolo-copias/versiones/vX.Y.Z/` | En la copia semanal |
| **El contenedor de Tag Manager** | Exportado a mano por el cliente (Administrar › Exportar contenedor) y guardado con sus documentos | Cada vez que se cambie |
| Las credenciales de R2 y la URL del aviso | `~/.config/apolo/copias.env` (600). El token solo puede tocar este bucket | — |

**El día 1 de cada mes a las 05:00, `scripts/copias/ensayo.sh` restaura desde R2**: clona el
repositorio de la última semana, descifra la última copia de las claves y la compara con las del
VPS, y comprueba las sumas de la última versión guardada.

Si la copia semanal falla o no se hace, **healthchecks.io avisa** (check «Apolo · copia semanal», con
una semana de periodo y un día de margen). El registro está en `~/backups/apolo/copias.log`.

Lo que no se copia porque ya está fuera, o se rehace solo:
- las dependencias de Rust y npm (crates.io y npm, fijadas por `Cargo.lock` y los `pnpm-lock.yaml`);
- la web (se compila desde el repositorio);
- Cloudflare (sus reglas están en trampas.md), GA4 y Search Console (en estado.md).

## Restaurar

### Desde cualquier ordenador: bajar la copia

Hace falta Docker y las credenciales de R2: o el `copias.env` de siempre, o un token nuevo del bucket
creado en Cloudflare › R2. Con esas variables en un fichero `rclone.env` como el que escribe
`scripts/copias/comun.sh`:

```sh
docker run --rm --env-file rclone.env -v "$PWD:/data" rclone/rclone:1.71.2 \
  --config /dev/null --s3-no-check-bucket copy r2:apolo-copias/claves /data/claves
```

(lo mismo con `repos/AAAA-MM-DD`, `motores` o `versiones/vX.Y.Z`).

### Las claves de firma

```sh
gpg -d apolo-claves-AAAA-MM-DD.tar.gpg > claves.tar   # pide la frase: la de Coda
tar -xf claves.tar                                    # deja la carpeta claves/
```

Vuelven a `~/.config/apolo/claves/` (carpeta 700, ficheros 600). Si además se perdieron los secretos
de GitHub, se vuelven a crear desde aquí: `TAURI_SIGNING_PRIVATE_KEY` y su contraseña
(`actualizaciones.key`, `actualizaciones.contrasena`), `PAQUETES_GPG_CLAVE` y `PAQUETES_GPG_FRASE`
(`paquetes.key.asc`, `paquetes.frase`) y `AUR_CLAVE` (`aur.key`), con `gh secret set … < fichero`,
**sin escribirlos nunca en un chat ni en un documento**. `HOMEBREW_TAP_CLAVE` no está en la copia: es
una deploy key del tap y se crea otra.

### Los repositorios

```sh
git clone Apolo.bundle Apolo                 # con todas sus ramas y etiquetas
git -C Apolo remote set-url origin https://github.com/webcafeina/Apolo.git
```

Si el repositorio de GitHub se perdió, se crea vacío con el mismo nombre y se empuja todo:
`git push --mirror`. Lo mismo con `homebrew-apolo.bundle`.

### Los motores de terceros

Si algún submódulo ya no se puede bajar de su origen, se descomprime su `tar.gz` en su ruta
(`crates/avifjxl/vendor/aom`, …) y se compila igual.

### Las versiones publicadas

Los ficheros de `versiones/vX.Y.Z/` son los de la Release, con su `SHA256SUMS.txt`. Si la Release se
perdió, se recrea con `gh release create vX.Y.Z <ficheros> --notes-file .github/notas/vX.Y.Z.md`.

## Pruebas de restauración

Una copia que nunca se ha restaurado no se sabe si sirve. Apuntar aquí cada prueba.

| Fecha | Qué se probó | Resultado |
|---|---|---|
| 2026-10-27 | `claves.sh`: la copia de R2 se baja, se descifra con la frase del VPS y es idéntica al original | Bien |
| 2026-10-27 | `semanal.sh` (108 ficheros, 1,78 GB: repositorios, 12 motores, v0.18.1 a v0.18.3) y `ensayo.sh`: el repositorio se clona con 33 etiquetas, las claves se descifran y son las del VPS, las sumas de la v0.18.3 cuadran | Bien |
| 2026-10-27 | El cliente, en su Mac: baja la copia de las claves y la frase de Coda la descifra (lista los ficheros); healthchecks.io recibe el aviso; contenedor de Tag Manager exportado y guardado | Bien |
