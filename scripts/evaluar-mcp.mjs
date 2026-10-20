#!/usr/bin/env node
/**
 * Evaluación de apolo-mcp con Claude Code (entrega 7c, ADR 0023), como la de Cronos.
 *
 *   make mcp && node scripts/evaluar-mcp.mjs [--solo id1,id2] [--modelo sonnet] [--informe]
 *
 * Por cada escenario de evaluacion/escenarios.mjs: una carpeta temporal con copias de las
 * imágenes de prueba y su propia carpeta de presets, `claude -p` arrancado en ella con solo
 * apolo-mcp (sin herramientas integradas: ni shell ni ficheros, así que todo lo que pase en el disco
 * lo ha hecho Apolo), y la comprobación del escenario. En todos, además: ningún fichero que había
 * antes ha cambiado ni desaparecido.
 *
 * Las imágenes salen de la carpeta de pruebas de Lotes (pruebas/lotes/generar.sh), en
 * APOLO_EVAL_FUENTE o en ~/apolo-pruebas-lotes/vacaciones. Resultados crudos en
 * target/evaluacion/; con --informe, el resumen en docs/mcp/evaluacion.md.
 */
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { ESCENARIOS } from '../evaluacion/escenarios.mjs';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const MCP = join(raiz, 'target/release/apolo-mcp');
const FUENTE = process.env.APOLO_EVAL_FUENTE ?? join(homedir(), 'apolo-pruebas-lotes/vacaciones');
const arg = (n) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : undefined; };
const SOLO = arg('--solo')?.split(',');
const MODELO = arg('--modelo');
const INFORME = process.argv.includes('--informe');
const LIMITE_MS = 600_000;
const VERSION = readFileSync(join(raiz, 'Cargo.toml'), 'utf8').match(/^version = "(.+)"/m)[1];

if (!existsSync(MCP)) throw new Error('Falta target/release/apolo-mcp: make mcp');
if (!existsSync(FUENTE)) throw new Error(`Faltan las imágenes de prueba en ${FUENTE}: pruebas/lotes/generar.sh ~`);

// La carpeta de trabajo de cada escenario: nombres sencillos, como los de cualquiera.
const IMAGENES = {
  'playa-1.jpg': 'playa/playa-1.jpg',
  'playa-2.jpg': 'playa/playa-2.jpg',
  'logo.png': 'logos/logo-1.png',
  'fotos/cumbre-1.jpg': 'montaña/cumbre-1.jpg',
  'fotos/cumbre-2.jpg': 'montaña/cumbre-2.jpg',
  'fotos/cumbre-3.jpg': 'montaña/cumbre-3.jpg',
  'fotos/captura.png': 'capturas/captura-2.png',
};

function ficheros(dir) {
  const fuera = [];
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) fuera.push(...ficheros(p));
    else fuera.push(p);
  }
  return fuera;
}
const huella = (f) => createHash('sha256').update(readFileSync(f)).digest('hex');

/** Llama a una herramienta de apolo-mcp directamente, para preparar y comprobar. */
function apolo(config, cwd, herramienta, args, idioma = 'es') {
  return new Promise((resolver, rechazar) => {
    const p = spawn(MCP, [], { cwd, env: { ...process.env, APOLO_CONFIG: config, APOLO_IDIOMA: idioma }, stdio: ['pipe', 'pipe', 'ignore'] });
    let resto = '';
    p.stdout.on('data', (d) => {
      resto += d;
      for (;;) {
        const i = resto.indexOf('\n');
        if (i < 0) break;
        const m = JSON.parse(resto.slice(0, i));
        resto = resto.slice(i + 1);
        if (m.id === 2) {
          p.stdin.end();
          if (m.result?.isError) rechazar(new Error(`${herramienta}: ${m.result.content?.[0]?.text}`));
          else resolver(m.result.structuredContent);
        }
      }
    });
    p.on('error', rechazar);
    const enviar = (m) => p.stdin.write(`${JSON.stringify(m)}\n`);
    enviar({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'evaluar', version: '1' } } });
    enviar({ jsonrpc: '2.0', method: 'notifications/initialized' });
    enviar({ jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: herramienta, arguments: args } });
  });
}

// El idioma de apolo-mcp, el del escenario (ADR 0025): sin decirlo, arrancaría en el del sistema.
async function lanzarClaude(peticion, cwd, config, idioma = 'es') {
  const mcp = join(config, '..', 'mcp.json');
  writeFileSync(mcp, JSON.stringify({ mcpServers: { apolo: { command: MCP, env: { APOLO_CONFIG: config, APOLO_IDIOMA: idioma } } } }));
  const args = [
    '-p', peticion, '--output-format', 'stream-json', '--verbose', '--no-session-persistence',
    '--mcp-config', mcp, '--strict-mcp-config', '--tools', '', '--allowedTools', 'mcp__apolo', '--permission-mode', 'dontAsk',
    ...(MODELO ? ['--model', MODELO] : []),
  ];
  const inicio = Date.now();
  return new Promise((resolver) => {
    const p = spawn('claude', args, { cwd, stdio: ['ignore', 'pipe', 'pipe'] });
    let salida = '';
    let errores = '';
    p.stdout.on('data', (d) => { salida += d; });
    p.stderr.on('data', (d) => { errores += d; });
    const reloj = setTimeout(() => p.kill('SIGTERM'), LIMITE_MS);
    p.on('close', (codigo) => {
      clearTimeout(reloj);
      const eventos = salida.split('\n').filter(Boolean).flatMap((l) => { try { return [JSON.parse(l)]; } catch { return []; } });
      const init = eventos.find((e) => e.type === 'system' && e.subtype === 'init');
      const final = eventos.find((e) => e.type === 'result');
      const usos = new Map();
      const llamadas = [];
      for (const e of eventos) {
        for (const b of e.message?.content ?? []) {
          if (b.type === 'tool_use') { const l = { herramienta: b.name.replace(/^mcp__apolo__/, ''), args: b.input, error: false }; usos.set(b.id, l); llamadas.push(l); }
          if (b.type === 'tool_result' && usos.has(b.tool_use_id)) {
            const l = usos.get(b.tool_use_id);
            l.error = Boolean(b.is_error);
            if (l.error) l.textoError = (Array.isArray(b.content) ? b.content.map((x) => x.text ?? '').join(' ') : String(b.content ?? '')).slice(0, 300);
          }
        }
      }
      resolver({
        codigo, ms: Date.now() - inicio, texto: final?.result ?? '', llamadas, stderr: errores.slice(-500),
        herramientasCargadas: init?.tools ?? [], coste: final?.total_cost_usd ?? null, turnos: final?.num_turns ?? null,
      });
    });
  });
}

// Herramientas que solo leen: una consulta que llame a otra, falla. ToolSearch es de Claude Code.
const LECTURA = new Set(['analizar_imagen', 'medir_calidad', 'ver_imagen', 'ver_lote', 'listar_presets', 'acerca_de_apolo', 'ToolSearch']);

console.log(`Evaluando apolo-mcp ${VERSION}${MODELO ? ` con ${MODELO}` : ''}\n`);
const resultados = [];
const ejecucion = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
for (const e of ESCENARIOS.filter((x) => !SOLO || SOLO.includes(x.id))) {
  const base = mkdtempSync(join(tmpdir(), `apolo-eval-${e.id}-`));
  const trabajo = join(base, 'trabajo');
  const config = join(base, 'config');
  mkdirSync(config, { recursive: true });
  for (const [destino, origen] of Object.entries(IMAGENES)) {
    mkdirSync(dirname(join(trabajo, destino)), { recursive: true });
    copyFileSync(join(FUENTE, origen), join(trabajo, destino));
  }
  copyFileSync(join(raiz, 'pruebas/corpus/orientacion-6.png'), join(trabajo, 'girada.png'));
  const c = {
    version: VERSION,
    ruta: (f) => join(trabajo, f),
    apolo: (h, a) => apolo(config, trabajo, h, a, e.idioma),
  };
  let fila;
  try {
    await e.preparar?.(c);
    const antes = new Map(ficheros(trabajo).map((f) => [f, huella(f)]));
    c.nuevos = () => ficheros(trabajo).filter((f) => !antes.has(f)).map((f) => relative(trabajo, f));
    const r = await lanzarClaude(e.peticion, trabajo, config, e.idioma);
    const ajenas = r.herramientasCargadas.filter((t) => !t.startsWith('mcp__apolo__') && t !== 'ToolSearch');
    const tocados = [...antes].filter(([f, h]) => !existsSync(f) || huella(f) !== h).map(([f]) => relative(trabajo, f));
    let veredicto;
    if (r.codigo !== 0 && !r.texto) veredicto = `claude terminó con ${r.codigo}: ${r.stderr}`;
    else if (ajenas.length) veredicto = `herramientas fuera de Apolo cargadas: ${ajenas.join(', ')}`;
    else if (tocados.length) veredicto = `HA CAMBIADO O BORRADO: ${tocados.join(', ')}`;
    else if (e.soloLectura && r.llamadas.some((l) => !LECTURA.has(l.herramienta))) veredicto = 'escribió en una consulta';
    else veredicto = await e.comprobar(c, r);
    fila = {
      id: e.id, categoria: e.categoria, estado: veredicto === true ? 'ok' : 'fallo', motivo: veredicto === true ? null : String(veredicto), peticion: e.peticion,
      llamadas: r.llamadas.filter((l) => l.herramienta !== 'ToolSearch').length, errores: r.llamadas.filter((l) => l.error).length,
      segundos: Math.round(r.ms / 1000), turnos: r.turnos, coste: r.coste,
      secuencia: r.llamadas.filter((l) => l.herramienta !== 'ToolSearch').map((l) => `${l.herramienta}${l.error ? '✗' : ''}`),
      respuesta: r.texto, nuevos: c.nuevos(),
      detalleLlamadas: r.llamadas.map((l) => ({ herramienta: l.herramienta, args: l.args, error: l.error ? l.textoError : undefined })),
    };
  } catch (err) {
    fila = { id: e.id, categoria: e.categoria, estado: 'fallo', motivo: `error del ejecutor: ${err.message}` };
  }
  rmSync(base, { recursive: true, force: true });
  resultados.push(fila);
  console.log(`${fila.estado === 'ok' ? '✓' : '✗'} ${e.id} · ${fila.llamadas ?? '-'} llamadas, ${fila.errores ?? '-'} errores, ${fila.segundos ?? '-'} s${fila.motivo ? ` · ${fila.motivo}` : ''}`);
}

const ok = resultados.filter((r) => r.estado === 'ok').length;
const pct = resultados.length ? Math.round((ok / resultados.length) * 1000) / 10 : 0;
console.log(`\n${ok}/${resultados.length} escenarios correctos (${pct} %)`);

const dirDatos = join(raiz, 'target/evaluacion');
mkdirSync(dirDatos, { recursive: true });
writeFileSync(join(dirDatos, `${ejecucion}.json`), JSON.stringify({ version: VERSION, modelo: MODELO ?? 'por defecto', resultados }, null, 2));
console.log(`Resultados: target/evaluacion/${ejecucion}.json`);

if (INFORME) {
  const media = (k) => (resultados.length ? (resultados.reduce((n, r) => n + (r[k] ?? 0), 0) / resultados.length).toFixed(1) : '-');
  const md = [
    '# Evaluación de apolo-mcp',
    '',
    `> Generado por \`scripts/evaluar-mcp.mjs --informe\` el ${new Date().toISOString().slice(0, 16).replace('T', ' ')} UTC con apolo-mcp ${VERSION}.`,
    `> Modelo: ${MODELO ?? 'el de Claude Code por defecto'}. Decisiones en el [ADR 0023](../adr/0023-mcp.md).`,
    '',
    `**Resultado: ${ok}/${resultados.length} (${pct} %).** Media de llamadas por escenario: ${media('llamadas')}; errores de herramienta: ${media('errores')}; duración media: ${media('segundos')} s.`,
    '',
    '| Escenario | Categoría | Resultado | Llamadas | Errores | s | Secuencia |',
    '|---|---|---|---|---|---|---|',
    ...resultados.map((r) => `| \`${r.id}\` | ${r.categoria} | ${r.estado === 'ok' ? '✓' : `✗ ${String(r.motivo ?? '').replace(/\|/g, '\\|')}`} | ${r.llamadas ?? '-'} | ${r.errores ?? '-'} | ${r.segundos ?? '-'} | ${(r.secuencia ?? []).join(' → ')} |`),
    '',
    '## Método',
    '',
    '- **Escenarios:** `evaluacion/escenarios.mjs`. Peticiones en español, cada una en una carpeta nueva con copias de las imágenes de prueba (`pruebas/lotes/generar.sh`) y su propia carpeta de presets; el agente arranca en ella, como Claude Code en un proyecto.',
    '- **Comprobación determinista:** el disco y el propio Apolo (medidas, transparencia, nota SSIMULACRA 2 del fichero escrito). Sin un modelo como juez. En todos, además, **ningún fichero que había antes puede cambiar ni desaparecer**; una consulta que escribe, falla.',
    '- **Aislamiento:** `claude -p` con `--strict-mcp-config`, `--tools ""` (sin shell ni ficheros: lo que pasa en el disco lo ha hecho Apolo), `--allowedTools mcp__apolo` y `--permission-mode dontAsk`.',
    '- **Criterio:** los fallos se corrigen en las herramientas, sus descripciones y las instrucciones. Un escenario solo se retoca si su comprobación era errónea, y se apunta en [evaluacion-notas.md](evaluacion-notas.md).',
    '- **Datos crudos** (respuestas, secuencia y argumentos de cada llamada): `target/evaluacion/`.',
    '',
  ].join('\n');
  writeFileSync(join(raiz, 'docs/mcp/evaluacion.md'), md);
  console.log('Informe: docs/mcp/evaluacion.md');
}
process.exit(0);
