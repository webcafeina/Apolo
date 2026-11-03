// El lanzador de apolo-mcp para el plugin de Claude y para Smithery (ADR 0036).
//
// Los dos admiten una sola orden para todos los sistemas, y apolo-mcp está en un sitio distinto en
// cada uno: dentro de la aplicación en macOS, en %LOCALAPPDATA%\Apolo en Windows, en /usr/bin en
// Linux. Este guion busca el que instaló Apolo y le pasa la conversación tal cual: el servidor de
// verdad es siempre el de la aplicación, que se actualiza con ella.
//
// No baja ni ejecuta nada de fuera. Si Apolo no está instalado, contesta él mismo como un servidor
// MCP mínimo cuyas instrucciones, y su única herramienta, explican cómo instalarlo: así el agente se
// lo dice a la persona en vez de enseñar un error.
//
// Solo usa lo que trae Node (18 o más), sin dependencias.

import { spawn } from "node:child_process";
import { realpathSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { posix, win32 } from "node:path";
import { createInterface } from "node:readline";
import { pathToFileURL } from "node:url";

/** Dónde buscar apolo-mcp, en orden. `APOLO_MCP`, si está, manda: es para rutas propias. */
export function candidatos({ plataforma = process.platform, entorno = process.env, inicio = homedir() } = {}) {
  if (entorno.APOLO_MCP) return [entorno.APOLO_MCP];
  const p = plataforma === "win32" ? win32 : posix;
  const exe = plataforma === "win32" ? "apolo-mcp.exe" : "apolo-mcp";
  const fijos =
    {
      darwin: [
        "/Applications/Apolo.app/Contents/MacOS/apolo-mcp",
        p.join(inicio, "Applications/Apolo.app/Contents/MacOS/apolo-mcp"),
        // La fórmula de Homebrew (la CLI, con apolo-mcp).
        "/opt/homebrew/bin/apolo-mcp",
        "/usr/local/bin/apolo-mcp",
      ],
      // El instalador de Windows es por usuario; el de Chocolatey, también.
      win32: [
        entorno.LOCALAPPDATA && p.join(entorno.LOCALAPPDATA, "Apolo", exe),
        entorno.ProgramFiles && p.join(entorno.ProgramFiles, "Apolo", exe),
      ],
      // El .deb, el .rpm y el PKGBUILD lo dejan en /usr/bin; la CLI suelta, donde se ponga.
      linux: ["/usr/bin/apolo-mcp", "/usr/local/bin/apolo-mcp", p.join(inicio, ".local/bin/apolo-mcp")],
    }[plataforma] ?? [];
  const enRuta = (entorno.PATH ?? entorno.Path ?? "")
    .split(p.delimiter)
    .filter(Boolean)
    .map((d) => p.join(d, exe));
  return [...new Set([...fijos.filter(Boolean), ...enRuta])];
}

/** El primero que existe y es un fichero. */
export function encontrar(opciones) {
  return candidatos(opciones).find((r) => {
    try {
      return statSync(r).isFile();
    } catch {
      return false;
    }
  });
}

/** Español si el sistema está en español; si no, inglés. Como Apolo (ADR 0025). */
export function idioma(entorno = process.env) {
  const del = entorno.LC_ALL || entorno.LC_MESSAGES || entorno.LANG || Intl.DateTimeFormat().resolvedOptions().locale;
  return /^es\b|^es[_-]/i.test(del ?? "") ? "es" : "en";
}

const DESCARGAS = {
  es: { darwin: "/descargar/mac/", win32: "/descargar/windows/", linux: "/descargar/linux/" },
  en: { darwin: "/en/download/mac/", win32: "/en/download/windows/", linux: "/en/download/linux/" },
};

/** Lo que el agente tiene que contar cuando Apolo no está. */
export function aviso(lengua, plataforma = process.platform, buscada = process.env.APOLO_MCP) {
  const web = `https://apolo.webcafeina.com${DESCARGAS[lengua][plataforma] ?? (lengua === "es" ? "/" : "/en/")}`;
  if (lengua === "es") {
    return [
      buscada
        ? `No hay ningún apolo-mcp en ${buscada}, la ruta de la variable APOLO_MCP.`
        : "Apolo no está instalado en este ordenador, o no está donde lo dejan sus instaladores.",
      `Para optimizar imágenes con este agente, hay que instalar la aplicación Apolo, gratis: ${web}`,
      "El servidor MCP va dentro de la aplicación y se actualiza con ella.",
      plataforma === "linux" ? "En Linux, el .deb, el .rpm o el PKGBUILD: la AppImage no lo deja en una ruta fija." : "",
      "Después, hay que reiniciar el agente. Si apolo-mcp está en otra ruta, se le indica con la variable APOLO_MCP.",
      "Cuéntale esto a la persona, con el enlace, en vez de intentar optimizar las imágenes de otra forma.",
    ]
      .filter(Boolean)
      .join(" ");
  }
  return [
    buscada
      ? `There's no apolo-mcp at ${buscada}, the path in the APOLO_MCP variable.`
      : "Apolo isn't installed on this computer, or isn't where its installers put it.",
    `To optimize images with this agent, install the free Apolo app: ${web}`,
    "The MCP server ships inside the app and updates with it.",
    plataforma === "linux" ? "On Linux, use the .deb, the .rpm or the PKGBUILD: the AppImage doesn't leave it at a fixed path." : "",
    "Then restart the agent. If apolo-mcp is somewhere else, set the APOLO_MCP variable to its path.",
    "Tell the person this, with the link, instead of trying to optimize the images some other way.",
  ]
    .filter(Boolean)
    .join(" ");
}

function herramienta(lengua) {
  return {
    name: "instalar_apolo",
    title: lengua === "es" ? "Instalar Apolo" : "Install Apolo",
    description:
      lengua === "es"
        ? "Apolo no está instalado en este ordenador: dice cómo instalarlo. Úsala si piden optimizar, convertir o medir imágenes."
        : "Apolo isn't installed on this computer: says how to install it. Use it when asked to optimize, convert or measure images.",
    inputSchema: { type: "object", properties: {} },
    annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false },
  };
}

/** La respuesta del servidor mínimo a un mensaje JSON-RPC, o null si no la lleva (las notificaciones). */
export function responder(mensaje, lengua, plataforma = process.platform, buscada = process.env.APOLO_MCP) {
  if (mensaje === null || typeof mensaje !== "object" || mensaje.id === undefined) return null;
  const bien = (result) => ({ jsonrpc: "2.0", id: mensaje.id, result });
  const mal = (code, message) => ({ jsonrpc: "2.0", id: mensaje.id, error: { code, message } });
  switch (mensaje.method) {
    case "initialize":
      return bien({
        protocolVersion: mensaje.params?.protocolVersion ?? "2025-06-18",
        capabilities: { tools: {} },
        serverInfo: { name: "apolo", title: "Apolo", version: "0.0.0" },
        instructions: aviso(lengua, plataforma, buscada),
      });
    case "ping":
      return bien({});
    case "tools/list":
      return bien({ tools: [herramienta(lengua)] });
    case "tools/call":
      if (mensaje.params?.name !== "instalar_apolo") {
        return mal(-32602, lengua === "es" ? "Herramienta desconocida" : "Unknown tool");
      }
      return bien({ content: [{ type: "text", text: aviso(lengua, plataforma, buscada) }] });
    default:
      return mal(-32601, lengua === "es" ? "Método desconocido" : "Method not found");
  }
}

/** El servidor mínimo: una línea JSON por mensaje, como el transporte stdio de MCP. */
function sinApolo() {
  const lengua = idioma();
  const lineas = createInterface({ input: process.stdin, crlfDelay: Infinity });
  lineas.on("line", (linea) => {
    if (!linea.trim()) return;
    let m;
    try {
      m = JSON.parse(linea);
    } catch {
      return;
    }
    for (const r of (Array.isArray(m) ? m : [m]).map((x) => responder(x, lengua)).filter(Boolean)) {
      process.stdout.write(`${JSON.stringify(r)}\n`);
    }
  });
}

function main() {
  const binario = encontrar();
  if (!binario) return sinApolo();
  // La conversación va directa entre el agente y apolo-mcp: este proceso solo espera.
  const hijo = spawn(binario, process.argv.slice(2), { stdio: "inherit" });
  for (const s of ["SIGINT", "SIGTERM"]) process.on(s, () => hijo.kill(s));
  hijo.on("error", (e) => {
    process.stderr.write(`apolo-mcp: ${binario}: ${e.message}\n`);
    sinApolo();
  });
  hijo.on("exit", (codigo, senal) => process.exit(codigo ?? (senal ? 1 : 0)));
}

const directo = process.argv[1] && import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href;
if (directo) main();
