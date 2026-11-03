// Las pruebas del lanzador del plugin de Claude (ADR 0036): `make plugin`, dentro de
// `make comprobar`. Fuera de la carpeta del plugin, para que no viajen con él.

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { aviso, candidatos, idioma, responder } from "./apolo/servidor/apolo-mcp.mjs";

const LANZADOR = fileURLToPath(new URL("./apolo/servidor/apolo-mcp.mjs", import.meta.url));
const leer = (r) => readFileSync(new URL(r, import.meta.url), "utf8");

test("el plugin lleva la versión de Apolo", () => {
  // La del espacio de trabajo: la que sube el commit de cada versión.
  const cargo = leer("../../Cargo.toml").match(/\[workspace\.package\]\s*\nversion = "([^"]+)"/)[1];
  assert.equal(JSON.parse(leer("./apolo/.claude-plugin/plugin.json")).version, cargo);
});

test("el marketplace apunta a la carpeta del plugin, y el servidor, al lanzador", () => {
  const m = JSON.parse(leer("../../.claude-plugin/marketplace.json"));
  assert.deepEqual(m.plugins.map((p) => [p.name, p.source]), [["apolo", "./empaquetado/claude/apolo"]]);
  const s = JSON.parse(leer("./apolo/.mcp.json")).mcpServers.apolo;
  assert.deepEqual([s.command, ...s.args], ["node", "${CLAUDE_PLUGIN_ROOT}/servidor/apolo-mcp.mjs"]);
});

test("busca donde lo dejan los instaladores de cada sistema, y después en el PATH", () => {
  const mac = candidatos({ plataforma: "darwin", entorno: { PATH: "/usr/bin:/opt/x" }, inicio: "/Users/ana" });
  assert.equal(mac[0], "/Applications/Apolo.app/Contents/MacOS/apolo-mcp");
  assert.ok(mac.includes("/Users/ana/Applications/Apolo.app/Contents/MacOS/apolo-mcp"));
  assert.ok(mac.includes("/opt/homebrew/bin/apolo-mcp"));
  assert.equal(mac.at(-1), "/opt/x/apolo-mcp");

  const win = candidatos({
    plataforma: "win32",
    entorno: { LOCALAPPDATA: "C:\\Users\\ana\\AppData\\Local", Path: "C:\\bin;D:\\otra" },
    inicio: "C:\\Users\\ana",
  });
  assert.equal(win[0], "C:\\Users\\ana\\AppData\\Local\\Apolo\\apolo-mcp.exe");
  assert.ok(win.includes("D:\\otra\\apolo-mcp.exe"));

  const linux = candidatos({ plataforma: "linux", entorno: { PATH: "/usr/bin" }, inicio: "/home/ana" });
  assert.deepEqual(linux, ["/usr/bin/apolo-mcp", "/usr/local/bin/apolo-mcp", "/home/ana/.local/bin/apolo-mcp"]);
});

test("APOLO_MCP manda sobre todo lo demás", () => {
  assert.deepEqual(candidatos({ plataforma: "linux", entorno: { APOLO_MCP: "/x/apolo-mcp", PATH: "/usr/bin" } }), ["/x/apolo-mcp"]);
});

test("el idioma es el del sistema: español si lo es, y si no, inglés", () => {
  assert.equal(idioma({ LANG: "es_ES.UTF-8" }), "es");
  assert.equal(idioma({ LANG: "es" }), "es");
  assert.equal(idioma({ LC_ALL: "en_US.UTF-8", LANG: "es_ES.UTF-8" }), "en");
  assert.equal(idioma({ LANG: "C.UTF-8" }), "en");
  assert.equal(idioma({ LANG: "estonian" }), "en");
});

test("sin Apolo, contesta como un servidor MCP que explica cómo instalarlo", () => {
  const ini = responder({ jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18" } }, "es", "darwin");
  assert.equal(ini.result.protocolVersion, "2025-06-18");
  assert.equal(ini.result.serverInfo.name, "apolo");
  assert.match(ini.result.instructions, /https:\/\/apolo\.webcafeina\.com\/descargar\/mac\//);

  assert.equal(responder({ jsonrpc: "2.0", method: "notifications/initialized" }, "es"), null);
  assert.deepEqual(responder({ jsonrpc: "2.0", id: 2, method: "ping" }, "es").result, {});

  const lista = responder({ jsonrpc: "2.0", id: 3, method: "tools/list" }, "en", "win32").result.tools;
  assert.deepEqual(lista.map((h) => h.name), ["instalar_apolo"]);
  assert.equal(lista[0].annotations.readOnlyHint, true);

  const uso = responder({ jsonrpc: "2.0", id: 4, method: "tools/call", params: { name: "instalar_apolo", arguments: {} } }, "en", "win32");
  assert.match(uso.result.content[0].text, /\/en\/download\/windows\//);

  assert.equal(responder({ jsonrpc: "2.0", id: 5, method: "tools/call", params: { name: "optimizar_imagen" } }, "es").error.code, -32602);
  assert.equal(responder({ jsonrpc: "2.0", id: 6, method: "resources/list" }, "es").error.code, -32601);
});

test("el aviso dice qué ruta falló si se dio una, y lo de la AppImage en Linux", () => {
  assert.match(aviso("es", "linux", "/x/apolo-mcp"), /\/x\/apolo-mcp.*APOLO_MCP/);
  assert.match(aviso("en", "linux"), /AppImage/);
  assert.doesNotMatch(aviso("en", "darwin"), /AppImage/);
});

/** Arranca el lanzador con este entorno, le manda estas líneas y devuelve lo que contesta. */
function conversar(entorno, lineas) {
  return new Promise((resolver, fallar) => {
    const p = spawn(process.execPath, [LANZADOR], { env: { ...entorno, PATH: "" }, stdio: ["pipe", "pipe", "inherit"] });
    let salida = "";
    p.stdout.on("data", (d) => (salida += d));
    p.on("error", fallar);
    p.on("close", () => resolver(salida));
    p.stdin.end(lineas.map((l) => `${l}\n`).join(""));
  });
}

test("con apolo-mcp encontrado, la conversación pasa tal cual", { skip: process.platform === "win32" }, async () => {
  // cat devuelve lo que recibe: lo que entra por el lanzador llega al servidor y vuelve igual.
  const salida = await conversar({ APOLO_MCP: "/bin/cat" }, ['{"jsonrpc":"2.0","id":1,"method":"ping"}']);
  assert.equal(salida, '{"jsonrpc":"2.0","id":1,"method":"ping"}\n');
});

test("sin apolo-mcp, el servidor mínimo contesta por la tubería", async () => {
  const salida = await conversar({ APOLO_MCP: "/no/existe/apolo-mcp", LANG: "en_US.UTF-8" }, [
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}',
    '{"jsonrpc":"2.0","method":"notifications/initialized"}',
    "esto no es JSON",
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}',
  ]);
  const r = salida.trim().split("\n").map((l) => JSON.parse(l));
  assert.deepEqual(r.map((x) => x.id), [1, 2]);
  assert.match(r[0].result.instructions, /\/no\/existe\/apolo-mcp/);
  assert.equal(r[1].result.tools[0].name, "instalar_apolo");
});
