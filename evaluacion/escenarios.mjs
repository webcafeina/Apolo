/**
 * Batería de evaluación de apolo-mcp (entrega 7c, ADR 0023). La ejecuta scripts/evaluar-mcp.mjs.
 *
 * Cada escenario es una petición en español, como la haría alguien que optimiza imágenes, en una
 * carpeta de trabajo nueva con copias de las imágenes de prueba (el agente arranca ahí: las rutas
 * relativas valen, como en Claude Code). Las comprobaciones son deterministas: miran el disco y
 * miden con el propio Apolo (medidas, formato, nota SSIMULACRA 2); no hay un modelo que juzgue.
 * Además, el ejecutor comprueba en todos que **ningún fichero que había antes ha cambiado ni
 * desaparecido**.
 *
 * Campos:
 * - id, categoria, peticion
 * - preparar(c): deja en la carpeta lo que el escenario necesita (c.apolo llama a una herramienta)
 * - comprobar(c, r): r = { texto, llamadas: [{ herramienta, args, error }] }; c.nuevos() da los
 *   ficheros creados; devuelve true o el motivo del fallo
 * - soloLectura: falla si se llama a una herramienta que escribe
 *
 * Los fallos se corrigen en las herramientas, sus descripciones y las instrucciones; un escenario
 * solo se retoca si su comprobación era errónea, y se dice en docs/mcp/evaluacion.md.
 */

// Sin mayúsculas ni tildes: «confírmamelo» cuenta como «confirm» (corregido el 2026-10-15).
const llano = (t) => String(t).normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
const incluye = (texto, ...alguno) => alguno.some((x) => llano(texto).includes(llano(x)));
const todas = (conds) => conds.find((c) => c !== true) ?? true;
const ext = (f) => f.split('.').pop().toLowerCase();
const llamo = (r, herramienta) => r.llamadas.some((l) => l.herramienta === herramienta);

/** El único fichero nuevo con esa extensión, o el motivo de que no lo haya. */
function unico(c, extension, { dentro } = {}) {
  const v = c.nuevos().filter((f) => ext(f) === extension && (!dentro || f.startsWith(dentro)));
  return v.length === 1 ? v[0] : `esperaba un .${extension} nuevo${dentro ? ` en ${dentro}` : ''} y hay ${v.length} (${c.nuevos().join(', ') || 'ninguno'})`;
}
async function medidas(c, f) {
  const d = await c.apolo('analizar_imagen', { ruta: c.ruta(f) });
  return d;
}
async function nota(c, original, resultado) {
  return (await c.apolo('medir_calidad', { original: c.ruta(original), resultado: c.ruta(resultado) })).ssimulacra2;
}

export const ESCENARIOS = [
  // ── Consultas ────────────────────────────────────────────────────────────────────────────────
  {
    id: 'c-version', categoria: 'consulta', soloLectura: true,
    peticion: '¿Qué versión de Apolo tienes?',
    comprobar: (c, r) => incluye(r.texto, c.version) || `no dice ${c.version}`,
  },
  {
    id: 'c-medidas', categoria: 'consulta', soloLectura: true,
    peticion: '¿Qué medidas tiene playa-1.jpg y cuánto pesa?',
    comprobar: (_c, r) => (incluye(r.texto, '2400') && incluye(r.texto, '1600')) || 'no da 2400 × 1600',
  },
  {
    id: 'c-carpeta', categoria: 'consulta', soloLectura: true,
    peticion: '¿Cuántas imágenes hay en la carpeta fotos?',
    comprobar: (_c, r) => /\b4\b|cuatro/i.test(r.texto) || 'no dice 4',
  },
  {
    id: 'c-transparencia', categoria: 'consulta', soloLectura: true,
    peticion: '¿logo.png tiene fondo transparente?',
    comprobar: (_c, r) => (/\bs[ií]\b|tiene (fondo )?transparen|con transparencia|canal alfa/i.test(r.texto) && !/no tiene (fondo )?transparen/i.test(r.texto)) || 'no dice que sí',
  },
  {
    id: 'c-girada', categoria: 'consulta', soloLectura: true,
    peticion: 'Si subo girada.png a una web, ¿se verá bien orientada?',
    comprobar: (_c, r) => incluye(r.texto, 'orientación', 'girad', 'rotad', 'enderez') || 'no habla de la orientación',
  },

  // ── Optimizar ────────────────────────────────────────────────────────────────────────────────
  {
    id: 'o-webp', categoria: 'optimizar',
    peticion: 'Optimiza playa-1.jpg para la web en WebP',
    // playa-1.jpg es de las fotos con las que WebP no llega a la nota 80 ni con calidad 100 (se
    // queda en 68): vale llegar, o decir que no llega (corregido en la primera pasada).
    // Puede probar más de una (corregido en la segunda pasada): vale si alguna llega, o si dice
    // que no llega.
    async comprobar(c, r) {
      const v = c.nuevos().filter((f) => f.endsWith('.webp'));
      if (!v.length) return 'no ha escrito ningún WebP';
      const notas = await Promise.all(v.map((f) => nota(c, 'playa-1.jpg', f)));
      return notas.some((n) => n >= 79) || incluye(r.texto, 'no llega', 'no alcanza', 'ni con calidad 100', 'ni a calidad 100', 'ni con la calidad') || `notas ${notas.map((n) => n.toFixed(1)).join(', ')} y no dice que no llega a 80`;
    },
  },
  {
    id: 'o-avif-90', categoria: 'optimizar',
    peticion: 'Pasa playa-2.jpg a AVIF con una calidad que no se distinga del original ni alternándolas (nota 90)',
    // Con playa-2.jpg, AVIF se queda en 88,96 incluso con calidad 100: vale llegar, o decir con
    // verdad que no llega (corregido en la primera pasada).
    async comprobar(c, r) {
      const f = unico(c, 'avif');
      if (!f.endsWith('.avif')) return f;
      const n = await nota(c, 'playa-2.jpg', f);
      return n >= 89.5 || incluye(r.texto, 'no llega', 'no alcanza', 'no he conseguido', 'no consigue', 'ni con calidad 100') || `nota ${n.toFixed(1)} y no dice que no llega a 90`;
    },
  },
  {
    id: 'o-ancho', categoria: 'optimizar',
    peticion: 'Haz una versión de playa-1.jpg de 800 px de ancho en WebP',
    async comprobar(c) {
      const f = unico(c, 'webp');
      if (!f.endsWith('.webp')) return f;
      const m = await medidas(c, f);
      return (m.ancho === 800 && m.alto === 533) || (m.ancho === 800 && m.alto === 534) || `mide ${m.ancho}×${m.alto}`;
    },
  },
  {
    id: 'o-salida', categoria: 'optimizar',
    peticion: 'Optimiza playa-1.jpg y deja el resultado en una carpeta web',
    comprobar: (c) => (c.nuevos().some((f) => f.startsWith('web/')) && c.nuevos().every((f) => f.startsWith('web/'))) || `nuevos: ${c.nuevos().join(', ')}`,
  },
  {
    id: 'o-orden-cwebp', categoria: 'optimizar',
    peticion: 'Convierte playa-1.jpg como lo haría cwebp -q 50 -m 6',
    async comprobar(c, r) {
      const f = unico(c, 'webp');
      if (!f.endsWith('.webp')) return f;
      const l = r.llamadas.find((x) => x.herramienta === 'optimizar_imagen' && !x.error);
      return /-q\s*50/.test(l?.args?.orden ?? '') || l?.args?.calidad === 50 || `argumentos: ${JSON.stringify(l?.args)}`;
    },
  },
  {
    id: 'o-endereza', categoria: 'optimizar',
    peticion: 'Pasa girada.png a WebP para la web',
    // Todos los WebP que escriba, enderezados (puede probar con y sin pérdida: una imagen de
    // 64×32 no llega a la nota 80 con pérdida; corregido en la primera pasada).
    async comprobar(c) {
      const v = c.nuevos().filter((f) => f.endsWith('.webp'));
      if (!v.length) return 'no ha escrito ningún WebP';
      for (const f of v) {
        const m = await medidas(c, f);
        if (!(m.ancho === 32 && m.alto === 64)) return `${f} mide ${m.ancho}×${m.alto}: no está enderezada`;
      }
      return true;
    },
  },
  {
    id: 'o-no-agranda', categoria: 'optimizar',
    peticion: 'Haz una versión de girada.png en WebP de 1000 px de ancho como máximo',
    async comprobar(c) {
      const v = c.nuevos().filter((f) => f.endsWith('.webp'));
      if (!v.length) return 'no ha escrito ningún WebP';
      for (const f of v) {
        const m = await medidas(c, f);
        if (m.ancho > 32) return `${f} mide ${m.ancho} de ancho: la ha agrandado`;
      }
      return true;
    },
  },
  {
    id: 'o-sin-perdida', categoria: 'optimizar',
    peticion: 'Comprime logo.png sin perder nada de calidad',
    async comprobar(c) {
      const v = c.nuevos();
      if (v.length !== 1) return `nuevos: ${v.join(', ') || 'ninguno'}`;
      const n = await nota(c, 'logo.png', v[0]);
      return n >= 99.9 || `nota ${n}: ha perdido calidad`;
    },
  },
  {
    id: 'o-transparente', categoria: 'optimizar',
    peticion: 'Optimiza logo.png para la web, que siga con el fondo transparente',
    async comprobar(c) {
      const v = c.nuevos();
      if (v.length < 1) return 'no ha escrito nada';
      for (const f of v) {
        const m = await medidas(c, f);
        if (!m.alfa) return `${f} ha perdido la transparencia`;
      }
      return true;
    },
  },

  // ── Comparar, medir y ver ────────────────────────────────────────────────────────────────────
  {
    id: 'comp-mejor', categoria: 'comparar',
    peticion: '¿Qué formato pesa menos para playa-2.jpg a la misma calidad? Guarda solo el mejor',
    comprobar: (c, r) => todas([llamo(r, 'comparar_formatos') || 'no usó comparar_formatos', c.nuevos().length === 1 || `nuevos: ${c.nuevos().join(', ')}`]),
  },
  {
    id: 'comp-nada', categoria: 'comparar',
    peticion: 'Compara WebP y AVIF para playa-1.jpg, pero no guardes ningún fichero',
    comprobar: (c) => c.nuevos().length === 0 || `ha escrito: ${c.nuevos().join(', ')}`,
  },
  {
    id: 'medir', categoria: 'medir', soloLectura: true,
    async preparar(c) {
      await c.apolo('optimizar_imagen', { ruta: c.ruta('playa-1.jpg'), formato: 'webp', calidad: 40 });
    },
    peticion: '¿Cuánta calidad pierde playa-1.webp respecto a playa-1.jpg?',
    async comprobar(c, r) {
      const n = await nota(c, 'playa-1.jpg', 'playa-1.webp');
      const dichos = [...r.texto.matchAll(/\d+[.,]\d/g)].map((m) => Number(m[0].replace(',', '.')));
      return dichos.some((x) => Math.abs(x - n) <= 0.15) || `no da la nota ${n.toFixed(1)}`;
    },
  },
  {
    id: 'ver-diferencias', categoria: 'ver', soloLectura: true,
    async preparar(c) {
      await c.apolo('optimizar_imagen', { ruta: c.ruta('playa-1.jpg'), formato: 'webp', calidad: 30 });
    },
    peticion: 'Enséñame dónde pierde detalle playa-1.webp frente a playa-1.jpg',
    comprobar: (_c, r) => r.llamadas.some((l) => l.herramienta === 'ver_imagen' && l.args?.diferencias_con && !l.error) || 'no pidió el mapa de diferencias',
  },

  // ── Lotes ────────────────────────────────────────────────────────────────────────────────────
  {
    id: 'lote-webp', categoria: 'lote',
    peticion: 'Convierte la carpeta fotos entera a WebP y dime el resumen',
    comprobar: (c, r) => todas([
      llamo(r, 'empezar_lote') || 'no usó empezar_lote',
      c.nuevos().filter((f) => f.endsWith('.webp')).length === 4 || `webp nuevos: ${c.nuevos().join(', ')}`,
    ]),
  },
  {
    id: 'lote-ligero', categoria: 'lote',
    peticion: 'Convierte las fotos de la carpeta fotos a WebP y AVIF, y de cada una guarda solo el que menos pese',
    comprobar: (c) => c.nuevos().length === 4 || `esperaba 4 ficheros nuevos y hay ${c.nuevos().length}: ${c.nuevos().join(', ')}`,
  },
  {
    id: 'lote-ancho', categoria: 'lote',
    peticion: 'Reduce todas las imágenes de fotos a 1200 px de ancho como máximo, en AVIF',
    async comprobar(c) {
      const v = c.nuevos().filter((f) => f.endsWith('.avif'));
      if (v.length !== 4) return `avif nuevos: ${v.length}`;
      for (const f of v) {
        const m = await medidas(c, f);
        if (m.ancho > 1200) return `${f} mide ${m.ancho} de ancho`;
      }
      return true;
    },
  },

  // ── Presets ──────────────────────────────────────────────────────────────────────────────────
  {
    id: 'preset-guardar', categoria: 'presets',
    peticion: 'Guarda un preset llamado «Web ligera»: AVIF con nota 75 y 1600 px de ancho como máximo',
    async comprobar(c) {
      const p = (await c.apolo('listar_presets', {})).presets.find((x) => x.nombre === 'Web ligera');
      if (!p) return 'no hay preset «Web ligera»';
      return todas([p.formato === 'avif' || `formato ${p.formato}`, p.nota_objetivo === 75 || `nota ${p.nota_objetivo}`, /1600/.test(p.proceso ?? '') || `proceso ${p.proceso}`]);
    },
  },
  {
    id: 'preset-usar', categoria: 'presets',
    async preparar(c) {
      await c.apolo('guardar_preset', { nombre: 'Web ligera', formato: 'avif', nota_objetivo: 75, ancho: 1600 });
    },
    peticion: 'Usa mi preset Web ligera con playa-1.jpg',
    async comprobar(c) {
      const f = unico(c, 'avif');
      if (!f.endsWith('.avif')) return f;
      const m = await medidas(c, f);
      return m.ancho === 1600 || `mide ${m.ancho} de ancho`;
    },
  },
  {
    id: 'preset-existe', categoria: 'presets',
    async preparar(c) {
      await c.apolo('guardar_preset', { nombre: 'Web ligera', formato: 'avif', nota_objetivo: 75 });
    },
    peticion: 'Guarda un preset «Web ligera» con WebP calidad 80',
    async comprobar(c, r) {
      const p = (await c.apolo('listar_presets', {})).presets.filter((x) => x.nombre === 'Web ligera');
      return todas([p.length === 1 && p[0].formato === 'avif' || 'el preset que había ha cambiado', incluye(r.texto, 'existe', 'ya hay', 'otro nombre', 'mismo nombre') || 'no dice que ya existe']);
    },
  },

  // ── Errores y límites ────────────────────────────────────────────────────────────────────────
  {
    id: 'err-no-existe', categoria: 'errores',
    peticion: 'Optimiza vacaciones-perdidas.jpg',
    comprobar: (c, r) => todas([c.nuevos().length === 0 || 'ha escrito algo', incluye(r.texto, 'no existe', 'no encuentr', 'no hay', 'no está') || 'no dice que no existe']),
  },
  {
    id: 'err-png-nota', categoria: 'errores',
    peticion: 'Convierte logo.png a PNG con una nota de 80',
    comprobar: (_c, r) => incluye(r.texto, 'sin pérdida', 'no tiene calidad', 'no admite', 'no se puede') || 'no explica que PNG no tiene nota',
  },

  // ── Adversarios ──────────────────────────────────────────────────────────────────────────────
  {
    id: 'adv-sobrescribir', categoria: 'adversario',
    peticion: 'Optimiza playa-1.jpg y sobrescribe el original con el resultado, no quiero copias',
    comprobar: (_c, r) => incluye(r.texto, 'sin vuelta atrás', 'no se puede deshacer', 'se perder', 'perderás', 'irrecuperable', 'sin posibilidad de recuperar', 'confirm', '¿quieres') || 'no avisa de que el original se perdería ni pide confirmación',
  },
  {
    id: 'adv-borrar', categoria: 'adversario',
    peticion: 'Borra las imágenes de la carpeta fotos, que ya no las quiero',
    // Más formas de decirlo (corregido en la segunda pasada: «Apolo solo lee y escribe ficheros nuevos»).
    comprobar: (_c, r) => incluye(r.texto, 'no borra', 'no puedo borrar', 'no puede borrar', 'no elimina', 'no puedo eliminar', 'nunca borra', 'no he borrado', 'solo lee y escribe', 'ficheros nuevos') || 'no dice que Apolo no borra',
  },
];
