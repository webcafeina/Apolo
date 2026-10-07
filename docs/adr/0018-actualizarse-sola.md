# ADR 0018 — Actualizarse desde la propia aplicación, como Esfinge

**Fecha:** 2026-10-07 · **Estado:** aceptada · matiza la [0014](0014-versiones-y-publicacion.md)

## Contexto

Con la v0.3.2 en su Mac, y antes de empezar Lotes, el cliente pidió poder actualizar Apolo desde la
propia aplicación, «sin tener que reinstalar con cada versión nueva, como hacemos en Esfinge». Hasta
ahora cada versión era bajar el `.dmg`, arrastrar y volver a pasar por «Abrir igualmente».

La actualización estaba prevista para la entrega 7 (la 1.0). Se adelanta.

Esfinge (`~/proyectos/Esfinge`, su ADR 0016) la tiene así:

- una vez al día pregunta a GitHub por la última versión. Lo hace al abrir y luego cada hora, pero
  con una **puerta de 24 horas** que se apunta antes de salir a la red;
- si hay versión nueva, aparece una **banda** encima del contenido con «Descargar» y «Ahora no». Al
  bajar enseña el porcentaje; al terminar, «Instalar y reiniciar»;
- en Ajustes tiene la casilla «Avisarme cuando haya una versión nueva», una nota de privacidad, la
  fecha de la última comprobación y «Buscar ahora».

Esfinge es Go + Wails y lo hace todo a mano. Apolo es Tauri, que trae `tauri-plugin-updater`.

## Decisión

**El sistema de Esfinge, con el plugin de Tauri debajo.**

- **El plugin** (`tauri-plugin-updater` 2.13, con `tauri-plugin-process` para reiniciar):
  - lee `https://github.com/webcafeina/Apolo/releases/latest/download/latest.json`;
  - baja el paquete de cada sistema y **comprueba su firma minisign** contra la clave pública de
    `tauri.conf.json`.
- **Cómo se instala en cada sistema**:
  - **macOS**: reemplaza el `.app` con el `.app.tar.gz`. Si no tiene permiso, pide la contraseña de
    administrador.
  - **Windows**: lanza el NSIS en modo `passive`, con una barra de progreso y sin preguntas. Cierra
    Apolo y lo vuelve a abrir.
  - **Linux `.deb`**: `dpkg -i` a través de `pkexec`, que **pide la contraseña**. Lo eligió el
    cliente frente a solo avisar y abrir la página.
  - **AppImage**: se reescribe a sí misma.
  - El plugin sabe con qué paquete se instaló porque Tauri lo deja escrito en el binario al
    empaquetar.
- **Dos pasos, como Esfinge**: «Descargar» y, ya bajada, «Instalar y reiniciar». Se avisa y no se
  instala nada solo.
- **La puerta de 24 horas**, en Rust (`crates/servicio/src/ajustes.rs`):
  - bajo un cerrojo, mira la casilla y la hora de la última comprobación;
  - si toca, apunta la hora **antes** de preguntar;
  - si la red falla, el turno se gasta igual;
  - «Buscar ahora» se salta la casilla y las 24 horas;
  - los ajustes van a `ajustes.json`, junto a la carpeta de presets (`…/Apolo/`).
- **«Ahora no»** quita la banda hasta la próxima vez que se abra Apolo.
- **macOS traslocada**: una aplicación abierta sin moverla a Aplicaciones corre desde una copia de
  solo lectura (App Translocation) y no puede reemplazarse. La orden `plataforma` lo detecta (la
  ruta contiene `/AppTranslocation/`). En ese caso la banda no ofrece descargar y pide moverla a
  Aplicaciones.
- **Versiones normales, no pre-release.** `releases/latest` nunca apunta a una pre-release, así que
  con la 0014 tal cual el actualizador no vería nada. Lo eligió el cliente. **Matiza la ADR 0014**:
  todo lo demás (borrador, notas, CLI, sumas) se queda.
- **La clave del actualizador**:
  - minisign, generada con una contraseña hexadecimal;
  - la privada y su contraseña van en los secretos de GitHub (`TAURI_SIGNING_PRIVATE_KEY` y
    `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`);
  - una copia va a la bóveda de Esfinge del cliente;
  - la copia del VPS (`~/.config/apolo/claves/`, modo 600) se borra en cuanto el cliente la tenga.
- **`latest.json` se repasa al publicar**. tauri-action lo escribe a trozos, uno por trabajo, con
  URLs de la API de GitHub. Esas URLs tienen un límite de 60 peticiones por hora sin sesión. El
  trabajo `adjuntar` de `publicar.yml` hace tres cosas:
  - cambia cada URL por `releases/download/<etiqueta>/<fichero>`;
  - comprueba que cada plataforma tiene su fichero y su firma;
  - **falla si falta alguna de las 12 claves** (`darwin-*`, `windows-*`, `linux-*`, y las de
    paquete: `-deb`, `-appimage`, `-nsis`).
- **En el navegador** (`make dev-web`, Playwright) no hay plugin. Con `?novedad=9.9.9` en la URL se
  simula una versión nueva que se «descarga» sola, y así se prueba la banda.

## Alternativas descartadas

- **Hacerlo a mano como Esfinge** (preguntar a la API de GitHub, bajar y reemplazar con un guion).
  El plugin ya lo hace, comprueba firmas y conoce los paquetes de Tauri. Esfinge lo hizo a mano
  porque Wails no lo trae.
- **Instalar sola al encontrarla.** Esfinge avisa y deja elegir el momento. Una ventana que se
  cierra sola a media imagen es peor que un clic.
- **Seguir con pre-releases y un `latest.json` en otro sitio** (una rama, una web). Es una pieza más
  que mantener, para una distinción (0.x en pre-release) que al cliente no le importa.
- **En Linux, solo avisar y abrir la página.** Lo propuse por no pedir contraseña. El cliente eligió
  instalar.
- **Guardar la puerta en el navegador** (`localStorage`). Se borraría con los datos de la vista web,
  y la CLI y la ventana comparten carpeta de configuración. Mejor un fichero junto a los presets.

## Consecuencias

- **La v0.3.3 hay que instalarla a mano una última vez**: las anteriores no tienen actualizador.
  Hasta la v0.3.4 no se puede ver una actualización de verdad.
- **Perder la clave privada** es no poder volver a actualizar a nadie: cada instalación solo acepta
  lo firmado con ella. Cambiarla exige otra vez una instalación a mano. Por eso va a la bóveda.
- Apolo sale a la red una vez al día, y solo para esto. La nota de privacidad de Ajustes lo dice,
  y la casilla lo apaga.
- En macOS sin firmar, la aplicación reemplazada no lleva la marca de cuarentena (la baja Apolo, no
  el navegador), así que no debería volver a pedir «Abrir igualmente». Sin comprobar.

## Verificación

2026-10-07, en el VPS:

- **Pruebas de Rust** de la puerta (`crates/servicio/src/ajustes.rs`):
  - una vez al día;
  - apagada no pregunta, pero «Buscar ahora» sí;
  - se guarda entre sesiones.
- **Pruebas e2e** con la novedad simulada (`frontend/e2e/actualizar.spec.ts`):
  - la banda anuncia, descarga con porcentaje, pide «Instalar y reiniciar» y pasa a «Instalando…»;
  - «Ahora no» la quita;
  - sin novedad dice «Ya tienes la última versión.»;
  - la casilla se recuerda al recargar.
- **Capturas** de la banda en claro y oscuro (`make capturas`).
- **Clave**: generada y probada firmando un fichero con `tauri signer sign` y su contraseña. Está en
  los secretos de GitHub.
- **Licencias** de lo que trae el plugin: MIT, Apache-2.0, BSD-3, ISC, Zlib y CDLA-Permissive-2.0
  (los certificados raíz de `webpki-roots`). Nada AGPL.

- **CI**, con `publicar.yml` lanzado a mano y sin etiqueta: la ventana compila con los dos plugins en
  los seis objetivos. Salen los **siete paquetes del actualizador con su `.sig`**:
  - `Apolo.app.tar.gz` universal;
  - los NSIS x64 y ARM64;
  - los `.deb` y las AppImage amd64 y arm64.

  Las siete firmas se comprobaron una a una contra la clave pública de `tauri.conf.json`, con un
  verificador minisign escrito aparte (Ed25519 sobre el BLAKE2b del fichero, más el comentario de
  confianza).

**No verificado:**

- ~~el `latest.json` y el paso que lo repasa~~: **verificado al publicar la v0.3.3**. Son 14
  entradas que apuntan a `releases/download/v0.3.3/…`, todas con firma válida, y el endpoint de
  `tauri.conf.json` responde 0.3.3;
- una actualización de verdad en ningún sistema (hace falta la v0.3.4);
- el caso traslocado en un Mac.
