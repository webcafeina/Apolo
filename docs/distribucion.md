# Dónde más puede estar Apolo

Investigación del 2026-10-27, tras la 1.0.0: tiendas oficiales, gestores de paquetes, directorios
de software y catálogos de MCP. Se miró en las páginas oficiales de cada uno; lo que no se pudo
confirmar va marcado como «sin confirmar». Los requisitos y precios cambian: antes de enviar, se
vuelven a mirar. Cada envío, con el sí del cliente (`docs/lanzamiento.md` tiene los textos).

Punto de partida el 2026-10-27: el repositorio tiene **0 estrellas y 0 forks** y se creó el
2026-10-06, así que todo lo que pide notoriedad (homebrew-core y homebrew-cask, Scoop Extras,
awesome-rust) queda cerrado por ahora.

## Los dos envíos que ya están abiertos

- **winget**, [PR #463314](https://github.com/microsoft/winget-pkgs/pull/463314): etiquetas
  `Needs-CLA` y `New-Package`. **No avanza hasta firmar la CLA de Microsoft**, contestando en el PR
  con `@microsoft-github-policy-service agree company="Webcafeína"`. Lo decide el cliente: es un
  acuerdo legal en nombre de la empresa. La cola de paquetes nuevos es de semanas.
- **awesome-mcp-servers**, [PR #17812](https://github.com/punkpeye/awesome-mcp-servers/pull/17812):
  etiqueta `missing-glama`. Pide que Apolo esté en **Glama** y pase sus comprobaciones, y añadir al
  PR la insignia de su nota. Glama necesita un Dockerfile, que se pega en su web y no va en el
  repositorio: basta uno que baje el `apolo-mcp` de Linux x64 de la Release y lo arranque por stdio.

## Resumen por canal

| Canal | ¿Encaja? | Requisitos clave | Coste | Esfuerzo | Beneficio |
|---|---|---|---|---|---|
| **Microsoft Store** (el `.exe` de NSIS, sin MSIX) | Con cambios | Instalador y todos sus `.exe` firmados con Authenticode (CA del Trusted Root Program; ni autofirmados ni los firma Microsoft). Instalación silenciosa, WebView2 sin conexión. El actualizador propio **puede quedarse** (regla 10.2.5). Cuenta de **empresa** (D-U-N-S o escrituras) | Cuenta gratis. Certificado: OV 150–300 USD/año, o Azure Artifact Signing ~9,99 USD/mes (si vale para la Store, sin confirmar), o SignPath gratis (sin confirmar) | Medio | Medio-alto, y la firma quita SmartScreen en todas partes |
| **Mac App Store** | No sin cambios grandes | Sandbox obligatorio, sin actualizador propio, sin `macOSPrivateApi` (el vidrio), la CLI y el MCP fuera del paquete. La FSF ve los términos de Apple incompatibles con la GPL (VLC, 2011) | 99 USD/año | Alto | Medio |
| **Developer ID + notarización** (fuera de la tienda) | Sí | Cuenta de Apple Developer, firma y notarización en CI | 99 USD/año | Bajo-medio | **Alto**: se acaba lo de «Abrir igualmente», y abre homebrew-cask |
| **Snap Store** | Sí, con cambios pequeños | Confinamiento strict con `home`; ocultar el aviso de actualización dentro del snap; `apolo.cli` y `apolo.mcp` | Cuenta Ubuntu One (gratis, sin confirmar) | Bajo-medio | Alto en Ubuntu: su App Center muestra snaps, no Flatpak |
| **Flathub** | Con cambios y **riesgo serio de rechazo** | Su política de IA exige declarar el código hecho con IA y prohíbe que un agente escriba el manifiesto o abra el PR; los revisores pueden rechazar sin más. Además: todo compilado desde la fuente sin red, permisos mínimos, verificación del dominio | Gratis | Alto | Alto fuera de Ubuntu |
| **homebrew-cask / homebrew-core oficiales** | No | Notarización obligatoria (desde septiembre de 2026 desactivan los casks que no pasan Gatekeeper) y notoriedad (225★, 90 forks o 90 watchers). homebrew-core no deja meter libheif, aom ni libjxl dentro, y eso rompe la equivalencia | 99 USD/año (Apple) | Alto | Muy alto en macOS |
| **Scoop Extras** | Todavía no | ~100★ o 50 forks, y un issue aprobado antes del PR | Gratis | Bajo | Medio |
| **Chocolatey** | Sí | Paquete nuspec con la URL de la Release y su suma; revisión de cada versión (días o semanas). No pide firma | Gratis (sin confirmar) | Medio | Medio |
| **winget** | Sí (PR abierto) | Firmar la CLA | Gratis | Bajo | Alto |
| **AUR** | Bloqueado | Registro de cuentas cerrado tras el incidente de malware de junio de 2026; estado de hoy sin confirmar. Vía: accountsupport@archlinux.org | Gratis | Bajo (el PKGBUILD ya existe) | Medio |
| **nixpkgs** | Sí | PR con mantenedor comprometido, `rustPlatform` y el hook de Tauri | Gratis | Medio-alto | Medio |
| **openSUSE OBS** (proyecto home:) | Sí | Sin revisión; arreglaría la deuda de openSUSE con el `.rpm` de Fedora | Gratis | Medio | Medio |
| **COPR** | Sí | Cuenta FAS | Gratis | Bajo-medio | Bajo-medio (ya hay repo RPM propio) |
| **Debian, Ubuntu, Fedora oficiales; MacPorts** | No a corto plazo | Cada crate empaquetado aparte y las bibliotecas C del sistema: rompe la equivalencia o cuesta meses | Gratis | Muy alto | Alto a largo plazo |
| **Uptodown** (Málaga) | Sí (Windows y Mac) | Developer Console, icono de 256 px, revisión editorial | Gratis | Bajo | Alto para el público hispano |
| **Softpedia, MacUpdate, MajorGeeks** | Sí | Formulario o correo | Gratis (sin confirmar) | Bajo | Medio, buena reputación |
| **OpenAlternative, SaaSHub, LibHunt** | Sí | Formulario | Gratis (OpenAlternative cobra por ir rápido) | Bajo | Medio |
| **Chip.de, Softonic** | **Evitar** | Instaladores con adware o detectados como PUP | — | — | Negativo |
| **Genbeta / Xataka** (prensa) | Sí | Nota de prensa | Gratis | Bajo | Medio-alto en España |
| **awesome-mac, open-source-mac-os-apps, Awesome-Linux-Software** | Sí | Un PR cada una, sin mínimo de estrellas | Gratis | Bajo | Medio |
| **awesome-tauri, awesome-rust** | No / todavía no | awesome-tauri ya no acepta aplicaciones; awesome-rust pide 50★ | — | — | — |
| **GitHub MCP Registry** (VS Code, Copilot, JetBrains) | Sí | Correo a partnerships@github.com (cómo trata los `.mcpb`, sin confirmar) | Gratis | Mínimo | **Alto** |
| **Glama** | Sí | Reclamar la ficha y darle el Dockerfile | Gratis | Bajo-medio | Medio (y lo pide awesome-mcp-servers) |
| **Smithery** | Sí | `smithery mcp publish` con un `.mcpb` | Gratis | Bajo | Medio |
| **mcpservers.org, mcp.so** | Sí | Formulario | Gratis (o 39 USD por ir rápido) | Bajo | Medio |
| **PulseMCP, mcp.directory, Zed** | Solos | Recogen del registro oficial | — | Nada | Medio |
| **Docker MCP Catalog** | No | Exige contenedor y no admite GPL | — | — | — |
| **Directorio de Claude** | Solo como plugin | Ya no acepta `.mcpb`; un plugin con un ejecutable queda para revisión manual; plan de pago | Plan de pago | Medio-alto | Alto |
| **Marketplace propio de Claude Code** | Sí | `.claude-plugin/marketplace.json` en el repositorio, sin revisión | Gratis | Bajo | Medio |

Fuentes: las páginas oficiales enlazadas en el informe de la investigación (Microsoft Learn, Apple
Developer, Snapcraft, docs.flathub.org, docs.brew.sh, docs.chocolatey.org, archlinux.org,
claude.com/docs…), consultadas el 2026-10-27.

## Recomendación

**Ya (cuesta poco y no toca Apolo):**
1. Desbloquear los dos PR: la CLA de winget (lo decide el cliente) y Glama para awesome-mcp-servers.
2. MCP: el correo al GitHub MCP Registry, Smithery, mcpservers.org y mcp.so.
3. El *plugin* de Claude por un marketplace propio en el repositorio; el directorio oficial, después.
4. Directorios: Uptodown, Softpedia, MacUpdate, MajorGeeks, OpenAlternative, SaaSHub y LibHunt.
5. Listas: awesome-mac, open-source-mac-os-apps y Awesome-Linux-Software.
6. Prensa: una nota a Genbeta y Xataka.
7. Chocolatey, que tarda en moderar.

**Más adelante (con decisión y, a veces, coste):**
8. Developer ID y notarización en macOS (99 USD/año): lo que más mejora la primera vez en el Mac, y
   abre homebrew-cask cuando haya notoriedad.
9. Firma Authenticode en Windows y la Microsoft Store.
10. Snap Store, el camino más barato al App Center de Ubuntu.
11. openSUSE OBS, nixpkgs y COPR.
12. El AUR cuando abra; Scoop Extras y awesome-rust cuando haya estrellas.
13. El directorio oficial de Claude como plugin.

**Descartar por ahora:** la Mac App Store (sandbox, sin actualizador, sin el vidrio, la CLI y el MCP
fuera, y el choque con la GPL); Flathub mientras su política de IA siga así (si se intenta, lo hace
una persona de Webcafeína y lo declara); Debian, Ubuntu y Fedora oficiales, homebrew-core y
MacPorts; el Docker MCP Catalog; Chip.de, Softonic y opensourcealternative.to.

## Smithery por dentro (investigado el 2026-10-30)

Leído en el código de su CLI (`smithery-ai/cli`, **AGPL-3.0**: se puede ejecutar, no enlazar) y de
su SDK (`@smithery/api` 0.68.0, Apache-2.0):

- **El formulario web solo publica servidores remotos** (pide la URL `https://…/mcp`). Los locales se
  publican subiendo un `.mcpb` con la CLI (`smithery mcp publish ./x.mcpb -n espacio/servidor`) o con
  la API: `PUT https://api.smithery.ai/servers/{espacio}/{servidor}/releases`, multipart con
  `payload` (JSON `{"type":"stdio","runtime":"binary"|"node"|"python"|"bun","serverCard":…}`) y
  `bundle`, y la clave en `SMITHERY_API_KEY`. Se puede hacer con `curl`, sin la CLI.
- **Un solo bundle por versión, sin campo de sistema ni arquitectura.** Al instalarlo, la CLI lo
  descarga, lo descomprime y ejecuta `server.mcp_config.command` tal cual: **no aplica
  `platform_overrides`**. Nuestros `.mcpb` (uno por sistema, `type: binary`) solo funcionarían en el
  sistema del que se suba.
- Lo que hacen otros proyectos de Rust (patchloom#1867): un `.mcpb` de tipo **node** con un lanzador
  `server/run.mjs` que busca el binario instalado y, si no está, recurre a otra vía. Node siempre
  está, porque la CLI de Smithery corre con él y Claude Desktop lo trae dentro.
- Para Apolo serían dos opciones: **un lanzador ligero** (unos KB) que use el `apolo-mcp` instalado
  (Homebrew, APT, RPM, el instalador de Windows) o, si no lo hay, baje el de su sistema de la
  Release y compruebe su SHA-256; o **un bundle gordo** con los cinco binarios (unos 73 MB). Ambas
  piden un cambio en `crates/mcpb`, un paso en CI con el secreto `SMITHERY_API_KEY` y una ADR.
- El espacio que Smithery crea al registrarse sale del correo (`info-ixqz`): renombrarlo a
  `webcafeina` antes de publicar.

## Lo que exigiría cambios en Apolo

| Cambio | Para qué | Qué supone |
|---|---|---|
| Developer ID y notarización | homebrew-cask, los usuarios de Mac | Cuenta de 99 USD/año, certificados en los secretos de CI, `notarytool` en `publicar.yml`; revisar la ADR 0009 |
| Firma Authenticode | Microsoft Store (obligatoria), menos SmartScreen | Certificado y `signCommand` de Tauri para todos los `.exe` |
| Variante de la Store | Microsoft Store | `tauri.microsoftstore.conf.json` con WebView2 sin conexión; no hace falta MSIX |
| Snap | Snap Store | `snapcraft.yaml` y ocultar el aviso de actualización dentro del snap |
| Flatpak | Flathub | Manifiesto compilado desde la fuente, parche sin el actualizador, verificación del dominio; escrito por una persona |
| Mac App Store | Mac App Store | Sin el vidrio, sandbox, pedir la carpeta de destino de los lotes, sin actualizador, sin CLI ni MCP dentro |
| Dockerfile para Glama | awesome-mcp-servers | No va en el repositorio: se pega en Glama |
| Plugin de Claude | Marketplace propio, directorio de Anthropic | Subcarpeta con `plugin.json` y `.mcp.json`, y política de privacidad |
