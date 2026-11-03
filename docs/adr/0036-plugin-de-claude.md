# ADR 0036 — Apolo como plugin de Claude (y Smithery, descartado)

**Fecha:** 2026-11-03 · **Estado:** aceptada · después de la 1.0.1

## Contexto

El directorio de extensiones de Claude ya no acepta `.mcpb`: un servidor local tiene que ir dentro de
un *plugin* (docs/distribucion.md). Smithery publica servidores locales desde un `.mcpb`, pero admite
**uno solo por versión** y ejecuta su orden tal cual, sin `platform_overrides`. Los nuestros son uno
por sistema y llevan dentro el binario de ese sistema.

Los dos tienen el mismo problema: **una sola orden para todos los sistemas**. En cambio, `apolo-mcp`
está en un sitio distinto en cada uno: dentro de `Apolo.app` en macOS, en `%LOCALAPPDATA%\Apolo` en
Windows y en `/usr/bin` en Linux. La documentación de los plugins lo dice expresamente: no hay
configuración por sistema, y `command` es un único valor.

## Decisión

Con el cliente, el 2026-11-03: el plugin y Smithery a la vez, con una página de privacidad, y **sin
bajar ningún binario**. Si Apolo no está instalado, se pide que se instale. **Smithery se descartó el
mismo día**: había cerrado su publicación el 31 de octubre (abajo).

- **Un lanzador en Node, sin dependencias** (`empaquetado/claude/apolo/servidor/apolo-mcp.mjs`):
  - busca el `apolo-mcp` instalado: la aplicación (también en `~/Applications`), Homebrew, el
    `PATH`, o la ruta de la variable `APOLO_MCP`;
  - lo arranca con la entrada y la salida heredadas, así que la conversación va directa, sin pasar
    por él;
  - si no lo encuentra, contesta él mismo como un servidor MCP mínimo: unas instrucciones y una
    herramienta, `instalar_apolo`, con el enlace de descarga de ese sistema. Así el agente se lo
    explica a la persona, en vez de que Claude enseñe un error.
- **El plugin** (`empaquetado/claude/apolo/`):
  - `plugin.json`, con logo, web, documentación, soporte y `privacyPolicyUrl`;
  - `.mcp.json`, que arranca `node ${CLAUDE_PLUGIN_ROOT}/servidor/apolo-mcp.mjs`;
  - la *skill* `optimizar-imagenes`, que dice cuándo usar Apolo y repite las costumbres que funcionan
    (pedir la ruta, comparar formatos, la nota 80, no tocar los originales);
  - el README.
  
  El repositorio es su propio marketplace (`.claude-plugin/marketplace.json`, llamado
  `webcafeina`): `/plugin marketplace add webcafeina/Apolo` y `/plugin install apolo@webcafeina`. El
  directorio de Anthropic lo recibe desde claude.ai/directory/manage, con el plan de pago del
  cliente. La versión del plugin es la de Apolo, y una prueba vigila que coincidan.
- ~~**Smithery**: `armar-mcpb --lanzador` arma `Apolo-X.Y.Z-node.mcpb`, de tipo node, con el mismo
  guion, el manifiesto con las herramientas del servidor y ningún binario (unos 80 KB).
  `publicar.yml` lo añade a la Release, y `smithery.yml`, al publicarla, lo sube a la API de
  Smithery (`PUT /servers/webcafeina%2Fapolo/releases`) con el secreto `SMITHERY_API_KEY`.~~
  **Quitado el 2026-11-03**, el mismo día (`e2869ce` lo trajo y el commit siguiente lo saca):
  Smithery es de Arcade.dev y el 31 de octubre cerró las cuentas, las claves de API, la API, la CLI
  y los SDK (https://smithery.ai/future). La clave recién creada daba 401. Queda como directorio
  público, «powered by Arcade.dev», sin decir cómo se añaden los servidores. Si abre una vía, el
  lanzador sirve igual: el paquete se rehace desde el historial.
- **La privacidad del programa** en la web (`/privacidad/` y `/en/privacy/`):
  - la aplicación solo pregunta a GitHub por las actualizaciones, y se puede apagar;
  - la CLI, el MCP y el plugin no se conectan a nada;
  - lo que lee el agente lo ve el proveedor del modelo;
  - la web mide visitas solo con permiso.
  
  El «Privacidad» del pie lleva ahora a esa página. La de Webcafeína, como responsable, queda
  enlazada desde ella y desde el mapa del sitio.

## Alternativas descartadas

- **Bajar `apolo-mcp` de la Release si no está instalado.** Sería más cómodo con Smithery, pero
  supone que un plugin descargue y ejecute un binario, con su revisión más estricta. Rompe además
  la idea de que el MCP va dentro de la aplicación y se actualiza con ella. Descartado por el
  cliente.
- **Un `.mcpb` gordo para Smithery con los cinco binarios** (unos 73 MB): roza su límite de 25 MB
  por paquete, y daría una copia que no se actualiza con la aplicación.
- **Meter el binario en el plugin**: se queda pendiente de una persona que lo revise, choca con la
  restricción de `bin/` en el chat y en Cowork, y harían falta cinco.
- **La orden `apolo-mcp` a secas, confiando en el `PATH`**: en macOS y en Windows los instaladores
  no lo ponen en el `PATH`, y la documentación no dice que la orden del MCP se busque ahí.
- **Usar la CLI de Smithery en CI**: es AGPL-3.0. Ejecutarla se podría, pero su API con `curl` hace
  lo mismo sin instalar nada.

## Consecuencias

- **El plugin necesita Node.js 18 o más en el `PATH`.** La instalación nativa de Claude Code no lo
  trae. Sin Node, Claude dice que el servidor no arranca. Queda
  en el README del plugin, en la web y en la deuda.
- El plugin funciona en Claude Code y en Cowork cuando se ejecuta en el ordenador. En el chat de
  claude.ai, no: el servidor es local. Para Claude Desktop sigue el `.mcpb`.
- La AppImage no deja `apolo-mcp` en una ruta fija: en Linux hace falta el `.deb`, el `.rpm`, el
  PKGBUILD o la CLI. El aviso del lanzador lo dice.
- Una pieza más que mantener en cada versión: la versión de `plugin.json` sube con la de
  `Cargo.toml` (lo vigila `make plugin`).
- En el `PATH`, el lanzador puede encontrar un `apolo-mcp` que no sea el nuestro. Es el mismo
  riesgo que cualquier orden: quien controla el `PATH` controla lo que se ejecuta.

## Verificación

- 2026-11-03, en el VPS:
  - `make plugin`: 9 pruebas del lanzador (rutas por sistema, idioma, el servidor mínimo y la
    tubería con `cat`) y `claude plugin validate --strict` del plugin y del marketplace, en verde;
  - **con un agente** (`claude -p --plugin-dir`): sin Apolo, explica cómo instalarlo y da el enlace;
    con el `apolo-mcp` de la CLI 1.0.1 publicada, responde «Apolo 1.0.1» por el lanzador;
  - el `-node.mcpb` descomprimido, arrancado como lo arranca Smithery (`node server/apolo-mcp.mjs`),
    respondía con las 11 herramientas (antes de quitarlo).
  - La API de Smithery, con la clave del cliente: `/health` responde, y `/namespaces` da 401
    («Invalid API key»), como dice su aviso de cierre.
- 2026-11-03, **el cliente en su Mac** (Apple Silicon, Apolo 1.0.0 en `/Applications`, Claude
  Code): instalado desde el marketplace, `/mcp` da `plugin:apolo:apolo` con 11 herramientas; la
  *skill* se cargó sola, la conversión a AVIF salió bien y el agente avisó de que el plugin era el
  1.0.1 y la aplicación, la 1.0.0.
- 2026-11-03, **la validación del directorio**: sin nada que bloquee; quedan para el revisor el
  guion en Node, un falso positivo de credencial y el parecido de `apolo` con Apollo.
- **Sin ver todavía**:
  - Windows, con su ruta y sin Node;
  - Cowork;
  - el veredicto del revisor de Anthropic.
