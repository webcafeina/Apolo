# Estado

Última actualización: **2026-10-26**

## Dónde se paró, y por dónde se sigue

> **PARADA AQUÍ (2026-10-26, 07:51 UTC): v0.18.2 publicada.** `Publicar` 40118422741 acabó en
> verde en todo, macOS incluido (la primera compilación del cambio de `ventana_macos.rs`). Borrador
> revisado: 31 ficheros, 30 sumas, 9 firmas, `latest.json` con 16 plataformas, CLI, `apolo-mcp` y
> los cinco `.mcpb` en 0.18.2, sin DLL de Visual C++, manifiesto ARM64 con UTF-8, AVIF 3/3, `.deb`
> con «Webcafeína», `.rpm` con `digests signatures OK`. **Lo siguiente**: que el cliente la vea en
> su Mac junto a Finder (la barra lateral, ahora con `NSVisualEffectMaterial::Sidebar`); si sigue
> más transparente, el plan B es un velo claro encima de la barra lateral (sistema-macos.css). Luego,
> la prueba guiada final y la 1.0.0.
>
> **(2026-10-24, noche): v0.18.2 etiquetada, sin publicar.** El
> cliente la pidió como versión, para verla actualizando. Etiqueta `v0.18.2` en `bac46b0`
> (Comprobar en verde); `Publicar` **40118422741** lanzado y sin esperar: al cerrar iban bien Linux,
> Windows x64, las CLI y la CLI de macOS, y seguían compilando `macos-universal` y `windows-arm64`.
> **Es la primera compilación de macOS con el cambio de `ventana_macos.rs`**: Comprobar solo compila
> en Linux, así que si `macos-universal` falló, el fallo está ahí (`AnyClass::get(c"NSGlassEffectView")`,
> imports de `window_vibrancy`); arreglarlo, borrar el borrador y la etiqueta
> (`gh release delete v0.18.2 --yes && git push --delete origin v0.18.2 && git tag -d v0.18.2`) y
> volver a etiquetar. Qué lleva la 0.18.2:
> - macOS: la ventana con `NSVisualEffectMaterial::Sidebar` en todas las versiones, en vez de
>   `NSGlassEffectView` en macOS 26+ (dejaba pasar mucho más que Finder: `#8c8fa0` frente a
>   `#e7e7ea` sobre una terminal oscura, medido en la captura del cliente
>   `capturas/cliente/Captura de pantalla 2026-10-24 a las 19.09.10.png`); el vidrio de los
>   controles sigue desde macOS 26;
> - macOS claro: los iconos de la barra lateral en `--oro-lateral` `#d99a00`, elegido por el cliente;
> - Windows: la bienvenida del Estudio, centrada como Lotes, Presets y Ajustes.
>
> **Lo siguiente, el lunes**:
> 1. Ver cómo acabó `Publicar` 40118422741 (`gh run view 40118422741`).
> 2. Si está en verde, revisar el borrador como siempre (sumas, firmas, `latest.json`, versiones,
>    DLL, manifiesto, AVIF, `.deb`, `.rpm`) y publicarlo: **el cliente ya pidió publicarla**.
> 3. Que la vea en su Mac junto a Finder. Si la barra lateral sigue más transparente, el plan B es
>    un velo claro encima de la barra lateral (sistema-macos.css), ajustado hasta quedar como Finder.
> 4. Después, la prueba guiada final y la 1.0.0.
>
> **(2026-10-24, 16:44 UTC): v0.18.1 publicada**, con los tres arreglos de la prueba de
> la v0.18.0. `Publicar` 40110006183 en verde a la primera, humo incluido (Arch también). Borrador
> revisado: 31 ficheros, 30 sumas, 9 firmas, `latest.json` con 16 plataformas, CLI, `apolo-mcp` y
> los cinco `.mcpb` en 0.18.1, sin DLL de Visual C++, manifiesto ARM64 con UTF-8, AVIF 3/3, `.deb`
> con «Webcafeína», `.rpm` con `digests signatures OK`. Todos los sistemas probados por el cliente,
> Arch incluido. **Lo siguiente**: la prueba guiada final y la 1.0.0. Cuando el AUR reabra, el
> cliente crea la cuenta y `gh workflow enable aur.yml`.
>
> **(2026-10-24, tarde): la v0.18.0, probada por el cliente actualizando** en el Mac,
> Windows ARM, Ubuntu, Kubuntu y Fedora KDE: «todo perfecto», salvo tres cosas ya arregladas en
> `main`, **sin versión**: «Partir de» dice el preset guardado aplicado; el panel de Windows, de 320
> a 240; Lotes, Presets y Ajustes centrados en Windows. **Lo siguiente**: que el cliente diga si se
> publican (como 0.18.1); luego la prueba guiada final y la 1.0.0. **Arch, probado**: la v0.18.0
> con el PKGBUILD en una máquina virtual de Arch Linux ARM (VMPal, KDE), «todo parece correcto».
>
> **(2026-10-24, 12:01 UTC): v0.18.0 publicada, Recortar y Redimensionar sobre la
> imagen** (ADR 0032). El cliente prefirió probarla actualizando desde el aviso en vez de con la
> compilación de prueba (40086447019, en verde). `Publicar` 40090110022 en verde a la primera, humo
> incluido. Borrador revisado: 31 ficheros, 30 sumas, 9 firmas, `latest.json` con 16 plataformas,
> CLI, `apolo-mcp` y los cinco `.mcpb` en 0.18.0, sin DLL de Visual C++, manifiesto ARM64 bien,
> AVIF 3/3, `.deb` con «Webcafeína», `.rpm` con `digests signatures OK`. Web (tarjeta y `apolo lote
> --lado-mayor 1920`), APT, RPM, PKGBUILD y Homebrew en 0.18.0. **Lo siguiente**: que el cliente la
> pruebe actualizando (Mac, Windows ARM, Ubuntu, Kubuntu o Fedora KDE): la barra, la ventanita con
> ratón, teclado y trackpad, el recorte, los dos lados, los presets y Lotes con tamaño; arreglar lo
> que salga. Luego la prueba guiada final y la 1.0.0. El disco del VPS se llenó compilando:
> `rm -rf target/debug/incremental` libera 14 GB.
>
> **(2026-10-24): v0.18.0 hecha y con la galería aprobada.** El cliente vio la galería
> (https://claude.ai/artifact/74j6quMT3n3XCiUHpM3VrC): «todo perfecto». Al sacarla se arreglaron
> dos cosas (`63b1741`): los botones de la ventanita salían planos (las reglas de la barra los
> alcanzaban) y en Lotes se abría cortada (trampas.md). ADR 0032, matices en la 0020 y la 0023.
> Todo subido, `make comprobar` y e2e (147) en verde. **Lo siguiente**: que el cliente pruebe la
> compilación de prueba en sus máquinas (Mac, Windows ARM, Ubuntu, Kubuntu o Fedora KDE): la barra,
> la ventanita con ratón, teclado y trackpad, el recorte, Lotes con tamaño. Después, con su sí, la
> v0.18.0: versión, notas bilingües, **la web y el README** (las tarjetas de «Recortar,
> redimensionar…» y el ejemplo de `apolo lote --lado-mayor 1920`; no antes, que `web/` se despliega
> solo), CI, etiqueta, revisión del borrador y publicar. Luego la prueba guiada final y la 1.0.0.
>
> **(2026-10-23, tarde): v0.18.0 hecha en el código hasta la fase 4 de 5.** Plan aprobado por el cliente en
> `~/.claude/plans/buenos-d-as-vamos-a-vivid-whale.md` (léelo: tiene todas sus decisiones). Hecho
> y con `make comprobar` y el e2e entero (147) en verde, en dos commits **sin subir todavía**
> (`f5cb51c` núcleo, MCP y CLI; `d14b49a` interfaz):
> - núcleo: `Medida` (medidas, `LadoMayor`, `Porcentaje`) en `Redimension`, guardada plana con
>   `modo` (los presets de antes se leen igual); `Proceso::efectivo` y `salida::efectivo` (una
>   redimensión o un recorte que no cambian nada ya no cuestan la orden equivalente); `lote::ejecutar`
>   nunca recorta;
> - MCP: `lado_mayor` y `porcentaje` (errores `tamano_doble`, `porcentaje_fuera`), `guardar_preset`
>   rechaza `recorte`, `empezar_lote` ya no lo tiene; catálogo regenerado; dos escenarios de
>   evaluación (`o-lado-mayor`, `o-porcentaje`), **sin correr** `make evaluar-mcp` (gasta: pedir el sí);
> - CLI: `apolo lote --lado-mayor/--porcentaje`;
> - interfaz: `BarraImagen.tsx` bajo la cabecera del Estudio (Recortar; Redimensionar con la
>   ventanita de `VentanaTamano.tsx` sobre `Emergente.tsx`), `tamano.ts` (+ test), recorte y tamaño
>   comunes a los dos lados (`comun` en Estudio.tsx), presets con tamaño y sin recorte, «Tamaño» en
>   Lotes; fuera del panel; el aspecto de cada sistema (CSS e iconos Fluent, Yaru y Breeze).
>
> **Lo siguiente**: (1) fase 5 de la galería: capturas nuevas de `capturas.spec.ts` («redimensionar»,
> «recorte», «lotes con tamaño») en los cinco aspectos con `CAPTURAS=1 … pnpm -C frontend exec
> playwright test capturas -g "redimensionar|recorte|lotes con tama"` (las variables de cada sistema,
> en el Makefile: `capturas-windows`, `-macos`, `-linux`, `-kde`), y un artefacto de galería para el
> cliente; (2) escribir la **ADR 0032** «Recortar y redimensionar, sobre la imagen y comunes a la
> imagen», su línea en decisiones.md, los matices de las ADR 0020 y 0023, y la trampa de WKWebView
> sin `popover`; (3) `make comprobar`, subir y CI; compilación de prueba (`publicar.yml` a mano) para
> sus máquinas; (4) v0.18.0 solo con su sí; luego la prueba guiada final y la 1.0.0.
>
> **(2026-10-23): v0.17.0 publicada (11:26 UTC), KDE con Breeze** (ADR 0031). El
> cliente la vio «perfecto» en **Kubuntu ARM** (compilación de prueba 39919081725, tras arreglar el
> icono de la barra lateral, Diferencias y el grupo de modos) y en **Fedora KDE ARM** (el rpm de la
> misma). De Fedora salieron dos cosas, arregladas en `9dbc508`: Agentes de IA en KDE va en
> tarjetas (no en el formulario de Ajustes) y, en todo Linux, un nombre de archivo largo se corta
> en la cabecera del Estudio (trampas.md). También arregla tres iconos de Yaru rotos desde la
> v0.15.0. `Publicar` 39941359700 en verde a la primera, humo incluido. Borrador revisado: 31
> ficheros, 30 sumas, 9 firmas, `latest.json` con 16 plataformas, CLI, `apolo-mcp` y los cinco
> `.mcpb` en 0.17.0, manifiesto ARM64 (ASCII, XML, UTF-8), sin DLL de Visual C++, AVIF 3/3, `.deb`
> con Maintainer «Webcafeína», los `.rpm` con `digests signatures OK` en el humo de Fedora. Publicada;
> `latest.json` público, Homebrew, APT, RPM y PKGBUILD en 0.17.0. **Lo siguiente**: la **prueba
> guiada final** (todas las pantallas en los cuatro sistemas, actualizando desde el aviso) y la
> **1.0.0**, con el sí del cliente. **El AUR sigue con el registro cerrado**: cuando abra, el
> cliente crea la cuenta, `gh workflow enable aur.yml` y se lanza con la versión publicada; la web y
> el README vuelven a `yay -S apolo-bin`.
>
> **(2026-10-22, noche): v0.17.0, KDE con Breeze (ADR 0031), hecha en el código y
> pendiente de verla en Kubuntu.** El cliente eligió sobre la galería
> (https://claude.ai/artifact/AGwN7QZdye1UBGJa8bKkDR) la **barra de Apolo también en KDE** (sin la de
> KWin, con el marco de Breeze: `MARCO_KDE`, radio 5) y el **botón principal relleno de oro**.
> Hecho (`70066a4`, `72d5d14`, subidos; `make comprobar`, e2e 121 y `Comprobar` de CI en verde):
> `escritorio()` en `ventana_linux.rs` (`gnome`/`kde`/`otro`) → `data-escritorio`; paletas
> `kde_claro`/`kde_oscuro` (10 en total); `sistema-kde.css`; `iconos-breeze.ts` (LGPL-3.0,
> generados con `~/.cache/apolo-revision/breeze.py` desde `~/.cache/kde-ref/`); `kde.spec.ts`;
> `make capturas-kde`. **Compilación de prueba 39847459084 en verde** (humo incluido). **Lo
> siguiente**: el cliente la prueba mañana en **Kubuntu ARM** (`kubuntu-desktop` sobre un clon de su
> Ubuntu 26.10, sesión Plasma; `apolo-linux-arm64` → `sudo apt install ./Apolo_0.16.0_arm64.deb`):
> la ventana sin barra con sombra y radio 5, arrastrar, doble clic, redimensionar, los botones de
> Breeze en su lado, todas las pantallas en claro y oscuro, Ajustes en formulario, Noto Sans, y si
> salen los huecos al repintar. Después Fedora KDE (rpm), arreglos, y la v0.17.0 (versión, notas
> bilingües, CI, etiqueta, revisión del borrador de siempre, publicar con su sí). Luego la prueba
> guiada final y la 1.0.0. **El AUR sigue con el registro cerrado** (`aur.yml` desactivado). **Para
> revisar borradores**: bajarlos a `~/.cache/apolo-revision/` y el `7zz` está en
> `~/.cache/apolo-revision/tool/7zz` (el scratchpad de `/tmp` se llena: cuota).
>
> **v0.16.0 publicada (2026-10-22, 19:03 UTC), Linux más allá de Debian** (ADR 0030): `.rpm`
> firmado con GPG (Fedora, openSUSE) que se actualiza desde la banda, repositorios APT y RPM firmados
> en apolo.webcafeina.com (`/apt`, `/rpm`, solo la última versión, montados por `web.yml` al
> publicar) y el PKGBUILD de `apolo-bin` en `/aur/PKGBUILD` (**el AUR tiene cerrado el registro**:
> `aur.yml` desactivado hasta que el cliente tenga cuenta; deuda.md). Clave GPG nueva en
> `~/.config/apolo/claves/` (`paquetes.*`, `gpg/`) y la SSH del AUR (`aur.key`); secretos
> `PAQUETES_GPG_CLAVE`, `PAQUETES_GPG_FRASE`, `AUR_CLAVE`. **No se borran ni se escriben en el chat.**
> Compilación de prueba 39816771619 vista «perfecto» en la Fedora ARM (GNOME) del cliente, sin los
> huecos de la VM de Ubuntu. Borrador revisado: 31 ficheros, 30 sumas, 9 firmas, `latest.json` con
> **16** plataformas (las `-rpm`), versiones y `.mcpb` en 0.16.0, manifiesto ARM64, sin DLL, AVIF
> 3/3, los dos `.rpm` con firma, humo de Fedora (amd64 y arm64) y Arch en verde. Publicada; web
> (repositorio RPM con `repomd.xml.asc` válido, APT y PKGBUILD en 0.16.0) y tap de Homebrew al día.
> **Lo siguiente**: que el cliente vea en su Fedora la actualización del rpm desde la banda (con la
> siguiente versión, o reinstalando la 0.15.0 de prueba) y el repositorio dnf; luego la **fase B,
> KDE con Breeze (v0.17.0)** del plan aprobado (`~/.claude/plans/buenos-d-as-vamos-a-vivid-whale.md`):
> el cliente probará primero en **Kubuntu** (`kubuntu-desktop` sobre su Ubuntu ARM) y después en
> **Fedora KDE**; la barra en KDE se decide sobre capturas. Después, la prueba guiada final y la 1.0.0.
>
> **v0.15.0 publicada (2026-10-22, 14:59 UTC)**, Linux con Ubuntu 26.10 (ADR 0029), con el
> interruptor de macOS con su burbuja y «Webcafeína» como editor. `publicar.yml` 39778214906 en
> verde a la primera (humo incluido). Borrador revisado como siempre: 27 ficheros, 26 sumas, 7
> firmas, `latest.json` con 14 plataformas, CLI y `apolo-mcp` en 0.15.0, los cinco `.mcpb` en
> 0.15.0, manifiesto de `apolo-app.exe` ARM64 válido, en ASCII y con UTF-8, sin DLL de Visual C++,
> AVIF 3/3; «Webcafeína» en los dos `apolo-app.exe`; el `.deb` arm64 con Maintainer «Webcafeína»,
> la descripción en español ASCII, el lanzador y el `metainfo.xml`. `latest.json` público en 0.15.0
> y el tap de Homebrew actualizado (fórmula y cask). **Lo siguiente**: que el cliente actualice desde
> el aviso en el Mac (el interruptor con su burbuja), en Windows (el editor) y en Ubuntu; luego la
> prueba guiada final y la 1.0.0.
>
> **Los huecos al repintar en la máquina virtual de Linux no son nuestros** (`7659847`): salen
> también con la primera compilación de prueba (gris oscuro) y sin marco, y no los quitan ni CPU,
> ni sin composición, ni sin DMA-BUF. El marco se queda; fuera el pintado por CPU; en trampas.md,
> deuda.md y la ADR 0029. Falta verlo en un Linux de verdad.
>
> **v0.15.0, Linux con Ubuntu 26.10 (2026-10-22)** (ADR 0029). Hecho y revisado con el
> cliente en capturas: libadwaita 1.10 con Yaru medidos en su código, barra lateral adaptativa,
> listas en tarjeta, controles de 34 con Ubuntu Sans 11 pt, iconos de Yaru; eligió la **barra
> propia como GNOME** (`ventana_linux.rs`: sin barra del sistema solo en GNOME, botones de la
> ventana en el lado de `gtk-decoration-layout`). Van también dos arreglos: el interruptor de
> macOS con su botón propio y la burbuja de vidrio, y «Webcafeína» como editor (`publisher`).
> `make comprobar` y e2e (107) en verde; **`ventana_linux.rs` lo compila CI** (aquí no hay GTK).
> **Lo siguiente**: CI y una compilación de prueba (`publicar.yml` a mano) para la **prueba guiada en
> la máquina virtual Ubuntu 26.10 ARM64**: la ventana sin barra del sistema, los botones, arrastrar,
> doble clic, redimensionar, la barra lateral, todas las pantallas, claro y oscuro, el editor en
> `dpkg -I`. Después, la v0.15.0 con su sí (en el Mac, el interruptor con su burbuja); luego la
> prueba guiada final y la 1.0.0.
>
> **macOS con Golden Gate: v0.14.0 y v0.14.1 publicadas el 2026-10-21 y vistas bien en el Mac del
> cliente** (ADR 0028). Hecho y revisado con el
> cliente en capturas («está perfecto como está ahora»; iconos de la barra lateral en oro): el
> diseño de macOS 27 medido en el kit oficial de Apple (Figma), con la barra lateral de borde a
> borde que se oculta, la barra de herramientas unificada con cápsulas de vidrio, controles de 24,
> filas agrupadas y la burbuja de vidrio al pulsar. `ventana_macos.rs` pone Liquid Glass (o el
> material de siempre) y los semáforos van en `trafficLightPosition`. `make comprobar` y e2e en
> verde; clippy para Apple silicon. **Nada visto aún en un Mac.**
> **Publicada la v0.14.1 (2026-10-21, 20:12 UTC)**, lo que salió de ver la v0.14.0 en el Mac del
> cliente (macOS 27): la barra lateral en el tamaño grande, el interruptor como píldora, el selector
> que no bota y cuya selección se desliza, y **el vidrio del sistema en los controles** con la
> propiedad privada de WebKit `-apple-visual-effect` (ADR 0028, matiz). Borrador revisado como
> siempre (27 ficheros, sumas, 7 firmas, `latest.json` con 14 plataformas, versiones y `.mcpb` en
> 0.14.1, manifiesto ARM64, sin DLL de Visual C++, AVIF 3/3); tap actualizado. La v0.14.0 se
> publicó a las 18:40 UTC.
> **Vista en el Mac del cliente (macOS 27): «todo perfecto»**, con el vidrio del sistema y los
> semáforos en su sitio. Solo un saltito del botón del interruptor al pulsarlo, apuntado en la deuda
> para la siguiente mejora.
> **Lo siguiente**: la prueba guiada final (Mac, Windows ARM, Ubuntu ARM) y la 1.0.0.
>
> **v0.13.0, Windows con Fluent (2026-10-21), en curso** (ADR 0027). Hecho y revisado con el
> cliente en capturas («ahora está perfecto»): en Windows, la paleta, la escala, los controles y
> los iconos de WinUI con el oro de Apolo, y la estructura de una aplicación de WinUI (barra de
> título propia de 48 px con ☰, panel de navegación plegable que abre plegado, capa del contenido,
> SettingsCards, InfoBar). La ventana sin marco y con Mica se crea desde código
> (`src-tauri/src/ventana_windows.rs`), con una ventana Win32 invisible sobre «Maximizar» para el
> menú de Snap. Interruptores con su forma en los tres sistemas. `make comprobar` y e2e en verde;
> el código de Windows compila y pasa clippy, **pero no se ha ejecutado nunca**.
> **Prueba guiada en la máquina virtual Windows 11 ARM, bien** (compilación de prueba
> 39590749049, CI y humo en verde): Mica, la barra, Snap, redimensionar, ☰ y las pantallas. Pidió
> animar el panel al plegarse: hecho. El tema oscuro, sin ver (Windows sin activar).
> **Publicada la v0.13.0 (2026-10-21, 13:56 UTC)**. `publicar.yml` 39601235133: el `.dmg` de
> macOS falló por `bundle_dmg.sh` (la trampa de siempre) y salió a la segunda con `gh run rerun
> --failed`; humo en verde en los tres sistemas. Borrador revisado como siempre: 27 ficheros,
> sumas, 7 firmas, `latest.json` con 14 plataformas, CLI y `apolo-mcp` en 0.13.0, los cinco `.mcpb`
> en 0.13.0, manifiesto de `apolo-app.exe` ARM64 válido, en ASCII y con UTF-8, sin DLL de Visual
> C++, AVIF 3/3 frente a avifenc 1.4.2. `latest.json` público en 0.13.0 y el tap de Homebrew
> actualizado solo (fórmula y cask en 0.13.0).
> **Lo siguiente**: la prueba guiada final (Mac, Windows ARM, Ubuntu ARM) y la 1.0.0.
>
> **La 8c está hecha y publicada la v0.12.0 (2026-10-20 por la noche)**, revisada como siempre
> (27 ficheros, sumas, 7 firmas, `latest.json` con 14 plataformas, versiones en 0.12.0, los cinco
> `.mcpb` en inglés, la CLI en los dos idiomas, manifiesto ARM64 válido, sin DLL de Visual C++,
> AVIF 3/3). ADR 0025:
> en Rust, cada texto con sus dos versiones (`tr!`, `trs!`); el idioma, uno por proceso (CLI con
> `--idioma` o el del sistema, `apolo-mcp` con `APOLO_IDIOMA`, la ventana con su selector en
> Ajustes › General); `en.json` con las 578 claves; las descripciones del MCP, con una tabla.
> Hecho con cuatro agentes en paralelo por crates. `make comprobar`, e2e 65/65, equivalencia y
> evaluación del MCP 37/37 en verde. README.en.md.
> ~~Prueba guiada en Windows ARM~~: **perfecta el 2026-10-21** (actualizar de la 0.10.2 a la
> 0.12.0 desde la banda, Lotes con una foto suelta en su subcarpeta, Apolo en inglés).
> **La 8d, casi (2026-10-21)**: la **web** publicada en `apolo.webcafeina.com` (Pages, DNS puesto
> por el cliente; el certificado HTTPS, pendiente de GitHub al parar) y **Homebrew** (tap
> `webcafeina/homebrew-apolo` en verde: fórmula en macOS y Linux, cask en macOS; se actualiza solo
> al publicar una versión con `homebrew.yml` y la deploy key `HOMEBREW_TAP_CLAVE`). ADR 0026.
> **Lo que queda para la 1.0**: «Enforce HTTPS» cuando GitHub emita el certificado; que el cliente
> mire la web y pruebe `brew install` en su Mac; la prueba guiada final (Mac, Windows ARM, Ubuntu
> ARM) y la 1.0.0.
>
> **La 8b está hecha y publicada la v0.11.0 (2026-10-20 por la tarde)**, revisada como siempre
> (27 ficheros, sumas, 7 firmas, `latest.json` con 14 plataformas, versiones y `.mcpb` en 0.11.0,
> manifiesto ARM64 válido, sin DLL de Visual C++, AVIF 3/3). Con 200 fotos de 24 MP, todo por debajo de 2,3 GB
> (antes, hasta 6 GB): docs/rendimiento.md. SSIMULACRA 2 canal a canal, AVIF descomprimido con un
> hilo, el turno de medir, `mallopt` en Linux, el banco. Además, decidido con el cliente: con
> imágenes sueltas, el lote deja el resultado en una subcarpeta junto a ellas (ADR 0019), y
> `comparar_formatos` manda a `empezar_lote` con fotos de más de 20 MP. Evaluación del MCP: los
> cinco escenarios tocados, bien (incluido `comp-grande`, nuevo).
> **Lo siguiente**: que el cliente vea **actualizar desde la banda en Windows ARM** (de la 0.10.2
> a la 0.11.0) y, si quiere, la subcarpeta de Lotes con una foto suelta. Después, la **8c**
> (inglés).
>
> **Publicada la v0.10.0** (2026-10-19), la 8a de la 1.0 (ADR 0024): `humo.yml` en CI (Windows x64,
> Linux x64 y arm64, en verde), AVIF y JPEG XL con tildes en Windows arreglados (manifiesto UTF-8) y
> el README con la tabla de descargas. Borrador revisado como siempre, con el humo en verde.
> ~~**Ubuntu ARM**~~: **bien el 2026-10-19 por la tarde**, en Ubuntu 26.10 ARM64. El `.deb` de la
> 0.9.1 se instala, la banda la pasa a la 0.10.0 con `pkexec`, el Estudio exporta AVIF y JPEG XL
> en una carpeta con tilde, Lotes da los mismos números que la CLI del VPS y la AppImage arranca.
> El MCP, sin agente (no hay Claude en esa máquina): `apolo-mcp --version` y la fila MCP, bien; las
> tuberías ya las prueba `humo.yml`.
> **Windows ARM, primer intento: «No se encontró MSVCP140.dll»** al instalar la 0.9.1. Ningún
> Windows sin Visual C++ Redistributable podía abrir Apolo, en ninguna versión (trampas.md).
> Arreglado con todo en /MT y **publicada la v0.10.1** (2026-10-19): humo en verde, ningún `.exe`
> pide las DLL de Visual C++, borrador revisado como siempre (27 ficheros, sumas, 7 firmas,
> `latest.json` con 14 plataformas, versiones y `.mcpb` en 0.10.1, AVIF 3/3).
> **Windows ARM, segundo intento (2026-10-20): «la configuración en paralelo no es correcta»** con
> la v0.10.1: el manifiesto UTF-8 de la v0.10.0 entraba roto en `apolo-app.exe` (trampas.md). La
> CLI y `apolo-mcp` iban bien. Arreglado, el humo abre ahora la ventana, y las imágenes del
> instalador al doble (se veían borrosas). **Publicada la v0.10.2** (2026-10-20), revisada como
> siempre más el manifiesto de la ventana ARM64 válido y sin DLL de Visual C++.
> ~~**Windows ARM**~~: **bien el 2026-10-20 con la v0.10.2** instalada directamente: el instalador
> (en español, imágenes nítidas), el Estudio exportando AVIF y JPEG XL en una carpeta con tilde,
> Lotes con los mismos números, y `apolo-mcp --version` en 0.10.2. Sin agente (no hay Claude en esa
> máquina); el MCP de Windows lo cubre `humo.yml` en x64.
> **La 8a está hecha.** Queda para la próxima versión ver **actualizar desde la banda en Windows**
> (de la 0.10.2 a la siguiente: las anteriores no arrancaban).
> **Lo siguiente: la 8b** (rendimiento, siguiente.md): el banco `scripts/banco.sh` con fotos de 24 MP
> y un lote de 200, medir, arreglar lo que pase de un Mac de 8 GB, v0.11.0. Ojo con el disco del VPS
> (deuda): el banco escribe mucho.
>
> **La v0.9.1** (2026-10-19): el recorte del agente llega desde Claude Desktop (esquema
> sin `$ref` y lectura tolerante), y el motivo dice qué proceso se hizo. Borrador revisado como
> siempre, y el `apolo-mcp` de la Release recorta con el recorte escrito como texto. Antes salió una
> comprobación de CI en rojo tres veces por clippy (memoria: `make comprobar` antes de subir).
> ~~Que el cliente repita el recorte en Claude Desktop~~: **bien el 2026-10-19** (recorte de Apolo,
> 800×1600, buscando la nota 80: calidad 69, SSIMULACRA 2 80,3, con transparencia). **La prueba de
> la v0.9.x está completa**: ventana, paleta, recorte del agente y AVIF en Mac 16/16.
> ~~Adaptar la evaluación del MCP a las fotos reales~~: hecho el 2026-10-19, 30 de 30.
> **Lo siguiente**: la 1.0 (entrega 8, siguiente.md), a plantear con el cliente.
>
> **La v0.9.0** (2026-10-16), detalle abajo. Antes, la **v0.8.3** (2026-10-16): las rutas de Apolo con `~` (el agente comentaba la cuenta
> «alvaro») y, en las descripciones, usar Apolo y no el intérprete de la nube con imágenes adjuntas.
> Borrador revisado como siempre (27 ficheros, sumas, firmas, `latest.json`, versiones, `.mcpb`,
> AVIF igual que avifenc 3/3). La entrega 7 (agentes por MCP) está hecha: 7a, 7b y 7c.

### Al parar el viernes 2026-10-16: lo primero el lunes 2026-10-19

**v0.9.0** (2026-10-16): recortar con el ratón en el Estudio (comparando con el deslizador), el
agente recorta y reduce la paleta, y el CI del Mac en `macos-15`.
**Publicada** (https://github.com/webcafeina/Apolo/releases/tag/v0.9.0), borrador revisado como
siempre: 27 ficheros, 26 sumas, 7 `.sig`, `latest.json` con 14 plataformas; CLI, `apolo-mcp` y los
cinco `.mcpb` en 0.9.0; el mismo AVIF que avifenc 3/3; la CLI de Mac, universal y con macOS mínimo
11.0.

1. **Prueba guiada de la v0.9.0 en el Mac del cliente**:
   - Recortar con el ratón y con el trackpad: activar Recortar, «Recortar en la imagen», mover el
     marco, estirarlo por esquinas y lados, una proporción (16:9) y «Listo»; comparar con el
     deslizador. Con una foto grande, que no vaya lento.
   - El agente (Claude Desktop): «Quédate con la mitad izquierda de <ruta> en WebP» y «Deja <un
     logo> en 16 colores».
   - Si Claude Desktop estaba abierto al actualizar: el aviso de reiniciar (otra vez, ya visto).
2. **AVIF frente al avifenc oficial de Mac** con la CLI de la v0.9.0: primera compilación con
   `macos-15` (deuda). Como el 2026-10-08: unas imágenes de `vacaciones` por unas combinaciones.
3. Nada más pendiente en siguiente.md con prioridad; la 1.0 (entrega 8) es lo siguiente grande.

> **Entregas 0, 1, 2 y 3 hechas el 2026-10-06.** Publicadas la v0.2.0, la v0.3.0 y la **v0.3.1**,
> todas como pre-release. La v0.3.1 añade:
> - la ventana rehecha con las medidas de Esfinge;
> - el icono del volumen (una unidad de aluminio con el sol);
> - el instalador de Windows en español y con imágenes.
>
> Lo pidió el cliente tras ver la v0.3.0 en su Mac, donde ya vio bien la ventana del `.dmg`, el
> vidrio y el icono del Dock.
>
> - **La 1** cumple la promesa central: `apolo webp` da **el mismo fichero, byte a byte, que el
>   cwebp 1.6.0 oficial**. Son 1015 de 1015 en Linux, y **25 de 25 en el Mac del cliente con Apple
>   Silicon**, que lo comprobó él a mano.
> - **La 2** es el Estudio ([ADR 0013](adr/0013-el-estudio-por-dentro.md)).
> - **La 3** es la identidad ([ADR 0015](adr/0015-identidad-nativa-con-marca.md) y
>   [0016](adr/0016-ventana-translucida.md)), que pidió el cliente al probar la v0.2.0:
>   - el **sol con laurel** como icono, elegido entre cuatro propuestas;
>   - el oro del sol como acento;
>   - la barra translúcida en macOS;
>   - el sello, la firma y la bienvenida;
>   - el `.dmg` con fondo propio y sin licencia.
>
> **La prueba guiada con el cliente quedó a medias**, a propósito. Los pasos 1 a 3 los hizo con la
> v0.2.0:
> - la CLI;
> - la equivalencia en su Mac;
> - instalar el `.dmg`.
>
> Al ver que la ventana «apenas tiene diseño», pidió hacer antes la identidad, y se sigue desde el
> **paso 4** con la v0.3.0.

### La siguiente acción, al retomar

0. **Entrega 7, agentes por MCP** ([ADR 0023](adr/0023-mcp.md)), decidida con el cliente el
   2026-10-14 en modo plan. Pasa delante de la 1.0, que queda como entrega 8.
   - **La 7a está hecha** (sin versión publicada): `crates/mcp` con el binario `apolo-mcp` y las
     diez herramientas, sus pruebas (las herramientas, el protocolo por tuberías con AVIF y JXL a la
     vez, el fd 1), el catálogo generado ([mcp/catalogo.md](mcp/catalogo.md)) y el paseo con Claude
     Code ([mcp/paseo.md](mcp/paseo.md)).
   - **La 7b está hecha en el código** (sin empujar): `apolo-mcp` dentro de cada instalador, el
     `.mcpb` con el relevo a la Apolo instalada, la herramienta `acerca_de_apolo` y la versión en
     las instrucciones, la fila «MCP» encima de Ajustes con la pestaña «Agentes de IA», CI, README y
     las notas de la v0.8.0. Lo pidió el cliente antes de empezarla: que se actualice sola, que diga
     su versión y la fila en la barra.
   - **Al parar (2026-10-14)**: empujado. `Comprobar` en verde (la ventana compila, e2e y
     equivalencia). Publicación de prueba **38558247128** (sin etiqueta): Linux y Windows bien. El
     `.deb` lleva `/usr/bin/apolo-mcp`, y el de la CI responde a `acerca_de_apolo`. El instalador de
     Windows x64 pasa de 9 a 16 MiB, que es `apolo-mcp.exe`. Hay `.mcpb` de Linux y de Windows (11
     herramientas), y la CLI trae `apolo-mcp`. ~~El Mac seguía compilando al parar~~: **acabó bien**
     (2026-10-15). `Apolo.app/Contents/MacOS/apolo-mcp` está junto a `apolo-app`, universal (x86_64 y
     arm64), y el `.mcpb` de macOS lleva `server/apolo-mcp` con el bit de ejecución. Los seis objetivos
     y las cinco CLI, en verde.
   - **Lo siguiente, mañana**:
     1. ~~Mirar cómo acabó el Mac de la publicación de prueba~~: bien (2026-10-15).
     2. ~~La v0.8.0~~: **publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.0),
        con el sí del cliente. Borrador revisado: 27 ficheros (los 22 de siempre y 5 `.mcpb`), 26
        sumas, 7 `.sig` y las 14 plataformas de `latest.json` con firma válida; la CLI y `apolo-mcp`
        de Linux dicen 0.8.0, `apolo-mcp` optimiza por el protocolo y sus instrucciones llevan la
        versión; los cinco `.mcpb` en 0.8.0 con su servidor ejecutable y once herramientas; la CLI da
        el mismo AVIF que avifenc 1.4.2 en 3 de 3.
     3. ~~La prueba guiada en su Mac~~: **bien el 2026-10-15** (pasos 1 a 5; Claude Code, aparte).
        Actualizó a la 0.8.0; la extensión, en Claude Desktop, cede el turno a la aplicación y el
        agente dice «Apolo 0.8.0»; optimizó un PNG con transparencia (AVIF −94 % a nota 80,1)
        mirando el mapa y zonas, sin tocar el original. Dos retoques para la 7c (siguiente.md).
     4. ~~Probar Claude Code con el cliente~~: **bien el 2026-10-15** (Sonnet 5.5, esfuerzo medio):
        rutas relativas, comparar, redimensionar, dos lotes. En la adversaria, Apolo no sobrescribió y
        Claude Code borró el original con su shell **tras pedir permiso** (el cliente lo aceptó): retoque de instrucciones para la 7c.
     5. ~~La 7c~~: **hecha el 2026-10-15**. `scripts/evaluar-mcp.mjs` con 28 escenarios
        (`make evaluar-mcp`): 26/28 con Sonnet en la pasada completa, y los dos que quedaban eran de
        las comprobaciones (corregidas, y pasan). Arreglos en Apolo: `ancho` y `alto` son un máximo
        que no agranda (`Redimension.solo_reducir`); los presets dicen su proceso; los tres retoques
        de las instrucciones.
     6. ~~La v0.8.1~~: **publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.1),
        con el sí del cliente y, a petición suya, el aviso de reinicio (Claude Desktop mantiene el
        servidor viejo hasta reiniciarse). Borrador revisado: 27 ficheros, 26 sumas, 7 `.sig` y 14
        plataformas en `latest.json` con firma válida; CLI y `apolo-mcp` en 0.8.1; cinco `.mcpb` en
        0.8.1 con su servidor ejecutable; el mismo AVIF que avifenc en 3 de 3.
     7. Prueba guiada de la v0.8.1: bien en lo nuevo (no agranda, presets, Claude Code avisa antes de
        sobrescribir); dos retoques de instrucciones y el aviso destacado, en `main` (siguiente.md).
        **Por repetir**: la imagen adjunta en Claude Desktop, en conversación nueva.
     8. **v0.8.2 publicada el 2026-10-15** (https://github.com/webcafeina/Apolo/releases/tag/v0.8.2)
        con lo que estaba en `main`: el aviso destacado de reiniciar Claude Desktop y los dos retoques
        de instrucciones (la cuenta del sistema; usar Apolo en vez del intérprete en la nube).
        Borrador revisado: 27 ficheros, sumas, 7 `.sig`, `latest.json` con 14 plataformas, CLI y
        `apolo-mcp` en 0.8.2 con las instrucciones nuevas, cinco `.mcpb`, AVIF igual que avifenc 3/3.

     9. Pendiente de antes: en
        la siguiente actualización, ver el aviso en Claude Desktop (deuda). Después: recortar con
        el ratón en el Estudio, recortar y paleta desde el agente, y el CI a `macos-15` antes del 2
        de noviembre. Las notas están en
        `.github/notas/v0.8.1.md`. Después: recortar con el ratón en el Estudio, y recortar y paleta
        desde el agente; pasar el CI a `macos-15` antes del 2 de noviembre (deuda).
   - Probarlo a mano: `make mcp`, y en Claude Code
     `claude mcp add --scope user apolo -- ~/proyectos/apolo/target/release/apolo-mcp`.
1. **La entrega 5b está publicada como v0.6.0** (2026-10-08,
   https://github.com/webcafeina/Apolo/releases/tag/v0.6.0) ([ADR 0021](adr/0021-avif-y-jpeg-xl.md)).
   Lleva también los tres retoques de la v0.5.1. Antes de publicar se revisó el borrador:
   - 22 ficheros, y las sumas cuadran;
   - `latest.json` con 14 plataformas, todas con firma válida (`verificar.py`, minisign con la
     clave pública de `tauri.conf.json`);
   - la CLI de Linux de la Release da los mismos bytes que avifenc y cjxl oficiales: 23 de 23.
   - **AVIF y JPEG XL** dan el mismo fichero que **avifenc 1.4.2 y cjxl 0.12.0**: 376 de 376 y 280
     de 280 con `make equivalencia`, también en CI.
   - Van compilados dentro (`crates/avifjxl`): sus `main`, con los argumentos de la orden. libjxl,
     con clang (en el VPS, `~/.local/llvm18`; el Makefile lo pone).
   - Se abren .avif y .jxl; Estudio, Lotes, pegar órdenes, `apolo avif` y `apolo jxl`.
   - CI: comprobar, e2e y equivalencia en verde. Los instaladores salen en macOS, Windows x64 y los
     dos Linux, y en Windows ARM64 tras apagar el NEON de libyuv: **los seis objetivos compilan**
     (ejecución 37687128943).
   - **v0.6.1 publicada el mismo día** con dos arreglos de la prueba: AVIF «Sin pérdida» con la
     calidad tocada antes daba «avifenc terminó con el código 1» (la orden llevaba lo que el panel
     apaga), y la barra de la descarga se veía llena antes de empezar. Revisada igual: 22
     ficheros, sumas, 14 firmas, y la CLI igual que avifenc y cjxl en 8 de 8.
2. ~~**Prueba guiada en el Mac del cliente**~~ **completa el 2026-10-08 con la v0.6.1**: los seis
   pasos bien, y `apolo avif` igual que el avifenc oficial de Mac en 16 de 16. Los pasos fueron: elegir AVIF y JPEG XL en el Estudio
   y moverse por sus controles; exportar y volver a abrir los dos; comparar AVIF con WebP; un JPEG a
   JPEG XL (recomprimido sin pérdida, y con la calidad al apagarlo); un lote con AVIF y «solo el más
   ligero»; `apolo avif` y `apolo jxl` frente a los oficiales de Mac, si los tiene.
3. **La clave del actualizador se queda en el VPS** (`~/.config/apolo/claves/`), por decisión del
   cliente (ADR 0018 y deuda). **No borrarla.** Tampoco se escribe nunca en el chat.
4. La carpeta de prueba de Lotes se regenera con `pruebas/lotes/generar.sh <destino>`.
5. **La entrega 6, métricas, está publicada como v0.7.0** (2026-10-08) ([ADR 0022](adr/0022-medir-la-perdida.md)).
   Borrador revisado como siempre (22 ficheros, sumas, 14 firmas; la CLI busca y mide, y su nota
   coincide con la de `ssimulacra2`).
6. ~~**Prueba guiada de la v0.7.0 en el Mac del cliente**~~ **completa el 2026-10-14**: los cinco
   pasos bien. En el paso 5, la CLI de Mac dio lo mismo que el VPS con `playa-1.jpg` (calidad 90 para
   la nota 80, SSIMULACRA 2 83,2). **v0.7.1 publicada el 2026-10-14** con los tres arreglos; borrador
   revisado (22 ficheros, sumas, 14 firmas, la CLI busca, mide y da el mismo AVIF que avifenc).
   ~~Lo siguiente era la entrega 7, la 1.0~~: el cliente metió antes los agentes por MCP (punto 0), y la 1.0 es la entrega 8. Los pasos 1 (la nota), 2 (calidad por nota) y 3 (Diferencias) salieron bien.
   - El paso 4 es **Lotes midiendo**: `vacaciones` en WebP con «Medir la calidad» (nota por fila,
     nota media, «Las de peor nota»), y opcional otro lote con «Buscar la calidad por nota» a 80.
     El texto del paso está en sesiones.md (2026-10-08, tarde).
   - Después, el paso 5 (opcional): la CLI, con `apolo medir` y `apolo jpeg -apolo_objetivo 80`.
   - **Tres arreglos de la prueba, ya en `main` y sin publicar**, para una **v0.7.1** al terminar:
     - Exportar se cortaba con medidas largas: ahora medidas y Exportar bajan juntos de línea si no
       caben;
     - no se veía que buscaba o medía: ahora hay una rueda y «Buscando la calidad para una nota de
       80…» o «Midiendo la calidad…»;
     - el aviso «Vista reducida» pisaba el selector y el rótulo de Diferencias: ahora va abajo,
       sobre el zoom.
   - Lo que salió y no era fallo: con la nota en 95, JPEG «no llega ni con calidad 100» en
     `captura-1.png` (llega como mucho a 89,8). Con 80 llega con calidad 90.
   El cliente eligió SSIMULACRA 2 con PSNR y SSIM, el modo «Diferencias», medir opcional en Lotes y
   **la calidad por nota en esta entrega**.
   - La nota es la de la herramienta `ssimulacra2` de libjxl: 56 de 56 con `make equivalencia`.
   - Al publicar: etiqueta `v0.7.0`, revisar el borrador como siempre, y prueba guiada en el Mac
     (la nota, la calidad por nota, Diferencias, un lote midiendo).

## Completado

- Planteamiento y ADR 0001–0010 (2026-10-06).
- Entrega 0, cimientos (2026-10-06).
- Entrega 1, núcleo WebP y CLI, con la equivalencia byte a byte (2026-10-06). ADR 0011.
- Entrega 2, el Estudio, con enderezar (2026-10-06). ADR 0012 y 0013.
- v0.2.0 publicada como pre-release (2026-10-06). ADR 0014.
- Entrega 3, identidad (2026-10-06). ADR 0015 y 0016.
- v0.3.0 publicada como pre-release (2026-10-06).
- v0.3.1 publicada: ventana de Esfinge, disco y Windows (2026-10-06).
- Prueba guiada de las entregas 1 y 2 en el Mac del cliente, completa (2026-10-06).
- v0.3.2: arreglos de la prueba y HEIC (2026-10-06). ADR 0017. Publicada el 2026-10-07.
- v0.3.3: el actualizador (2026-10-07). ADR 0018. Publicada el mismo día, ya no como pre-release.
  El cliente la instaló a mano, y «Buscar ahora» funciona.
- Entrega 4, Lotes (2026-10-07). ADR 0019. v0.4.0 publicada el mismo día.
- Prueba de Lotes en el Mac del cliente, completa (2026-10-07). v0.4.1 publicada con sus arreglos.
- Entrega 5a: JPEG, PNG, QOI, el proceso, un formato por lado y Lotes con varios formatos
  (2026-10-07). ADR 0020. v0.5.0 publicada el mismo día.
- Prueba guiada de la v0.5.0 en el Mac del cliente, completa (2026-10-07), con tres retoques.
- Entrega 5b: AVIF y JPEG XL con avifenc y cjxl compilados dentro (2026-10-07). ADR 0021. v0.6.0
  publicada el 2026-10-08, y la v0.6.1 con los arreglos de la prueba.
- Prueba guiada de la v0.6.1 en el Mac del cliente, completa (2026-10-08), con AVIF igual que el
  avifenc oficial de Mac en 16 de 16.
- Entrega 6, métricas (2026-10-08). ADR 0022. v0.7.0 publicada el mismo día.
- Prueba guiada de la v0.7.0, completa, y v0.7.1 publicada (2026-10-14).
- Entrega 7a: `apolo-mcp`, el servidor MCP con diez herramientas (2026-10-14). ADR 0023.
- Entrega 7b en el código: empaquetado, relevo, versión y la fila «MCP» (2026-10-14).

## En curso

- Entrega 7 (MCP): hechas la 7a y la 7b en el código; faltan CI, la v0.8.0 con su prueba guiada y la 7c (evaluación).
