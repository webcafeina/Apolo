// Las opciones de WebP tal como las serializa el núcleo (OpcionesWebp, en
// crates/nucleo/src/webp/opciones.rs), y el esquema de controles del panel.
//
// El panel no tiene un componente por opción: se pinta a partir de CONTROLES.
// Cada control dice qué campo toca, de qué tipo es, su rango y su nivel
// (Básico, Avanzado o Experto, los de docs/cobertura-cwebp.md). La etiqueta y
// la explicación salen de i18n con la clave `opcion.<clave>` y
// `opcion.<clave>Ayuda`.

export type Preset = "default" | "photo" | "picture" | "drawing" | "icon" | "text";
export type Pista = "ninguna" | "picture" | "photo" | "graph";
export type FiltradoAlfa = "none" | "fast" | "best";
export type ModoRedimension = "solo_reducir" | "solo_ampliar" | "siempre";

export interface Recorte {
  x: number;
  y: number;
  ancho: number;
  alto: number;
}

export interface Redimension {
  ancho: number;
  alto: number;
}

export interface Conservar {
  exif: boolean;
  icc: boolean;
  xmp: boolean;
}

export interface OpcionesWebp {
  preset: Preset | null;
  sin_perdida: boolean;
  calidad: number;
  metodo: number;
  pista: Pista;
  tamano_objetivo: number;
  psnr_objetivo: number;
  segmentos: number;
  sns: number;
  fuerza_filtro: number;
  nitidez_filtro: number;
  filtro_fuerte: boolean;
  autofiltro: boolean;
  compresion_alfa: number;
  filtrado_alfa: FiltradoAlfa;
  calidad_alfa: number;
  pasadas: number;
  preprocesado: number;
  limite_particion: number;
  emular_jpeg: boolean;
  hilos: number;
  poca_memoria: boolean;
  casi_sin_perdida: number;
  exacto: boolean;
  yuv_nitido: boolean;
  calidad_minima: number;
  calidad_maxima: number;
  recorte: Recorte | null;
  redimension: Redimension | null;
  modo_redimension: ModoRedimension;
  mezclar_alfa: number | null;
  sin_alfa: boolean;
  metadatos: Conservar;
  enderezar: boolean;
}

// --------------------------------------------------------- los demás formatos

export type FormatoSalida = "webp" | "jpeg" | "png" | "qoi";
export const FORMATOS: FormatoSalida[] = ["webp", "jpeg", "png", "qoi"];

/** Las opciones de cjpeg (OpcionesJpeg en crates/nucleo/src/jpeg/opciones.rs). */
export interface OpcionesJpeg {
  /** -quality N[,N…]; vacío, lo de cjpeg (75). */
  calidad: number[];
  revertir: boolean;
  color: "auto" | "gris" | "rgb";
  escaneo: "por_defecto" | "progresivo" | "secuencial";
  tablas_baseline: boolean;
  optimizar: boolean;
  dct: "int" | "fast" | "float" | null;
  rapido: boolean;
  dc_scan_opt: number | null;
  sin_trellis: boolean;
  trellis_dc: boolean | null;
  peso_dc: number | null;
  afinado: "psnr" | "hvs_psnr" | "ssim" | "ms_ssim" | null;
  tabla: number | null;
  lambda1: number | null;
  lambda2: number | null;
  sin_overshoot: boolean;
  sin_jfif: boolean;
  reinicio: { en: "filas" | "bloques"; n: number } | null;
  suavizado: number | null;
  muestreo: string | null;
  ranuras: string | null;
  enderezar: boolean;
}

/** Las opciones de oxipng (OpcionesPng en crates/nucleo/src/formatos/png.rs). */
export interface OpcionesPng {
  nivel: number;
  alfa: boolean;
  entrelazado: "no" | "si" | "mantener";
  quitar: "nada" | "seguro" | "todo";
  escala16: boolean;
  sin_bits: boolean;
  sin_color: boolean;
  sin_paleta: boolean;
  sin_gris: boolean;
  sin_reducciones: boolean;
  sin_recodificar: boolean;
  filtros: string | null;
  rapido: boolean;
  compresion: number | null;
  zopfli: boolean;
  iteraciones: number | null;
  sin_mejora: number | null;
  bruta_nivel: number | null;
  bruta_lineas: number | null;
  forzar: boolean;
  arreglar: boolean;
  enderezar: boolean;
}

export type Filtro = "lanczos3" | "mitchell" | "catmull_rom" | "bilineal" | "vecino";

/** El proceso antes de codificar (Proceso en crates/nucleo/src/proceso.rs). */
export interface Proceso {
  enderezar: boolean;
  recorte: Recorte | null;
  redimension: { ancho: number | null; alto: number | null; filtro: Filtro; lineal: boolean } | null;
  paleta: { colores: number; tramado: number } | null;
}

/** Todo lo que decide el fichero de salida (Ajuste en crates/nucleo/src/salida.rs). */
export interface Ajuste {
  formato: FormatoSalida;
  webp: OpcionesWebp;
  jpeg: OpcionesJpeg;
  png: OpcionesPng;
  proceso: Proceso;
}

// ------------------------------------------------------------- los controles

export type Nivel = "basico" | "avanzado" | "experto";
export const NIVELES: Nivel[] = ["basico", "avanzado", "experto"];

/** Lo que hace falta saber de la imagen para decidir qué controles tienen sentido. */
export interface Contexto {
  alfa: boolean;
  ancho: number;
  alto: number;
}

type Cuando = (a: Ajuste, c: Contexto) => boolean;

/** Dónde vive el campo que toca el control. */
export type De = "webp" | "jpeg" | "png" | "proceso";

interface Base {
  clave: string;
  /** Sin nivel: va en el grupo Proceso. */
  nivel?: Nivel;
  de: De;
  /** La opción de la herramienta, para enseñarla junto a la etiqueta (vacía: de Apolo). */
  marca: string;
  /** Si no se cumple, el control se ve apagado y explica por qué. */
  cuando?: Cuando;
  /** Clave i18n de por qué está apagado. */
  porQue?: string;
}

export type Control =
  | (Base & { tipo: "deslizador"; campo: string; min: number; max: number; paso: number })
  | (Base & { tipo: "interruptor"; campo: string })
  /** `nulo`: el valor que en la lista significa «sin poner» (null). */
  | (Base & { tipo: "lista"; campo: string; valores: string[]; nulo?: string; numerica?: boolean })
  | (Base & { tipo: "numero"; campo: string; min: number; max: number })
  | (Base & {
      tipo: "especial";
      especial:
        | "nivelSinPerdida"
        | "recorteCwebp"
        | "redimensionCwebp"
        | "rangoCalidad"
        | "mezclarAlfa"
        | "metadatos"
        | "hilos"
        | "calidadJpeg"
        | "procesoRecorte"
        | "procesoRedimension"
        | "procesoPaleta";
    });

const conPerdida: Cuando = (a) => !a.webp.sin_perdida;
const sinPerdida: Cuando = (a) => a.webp.sin_perdida;
const conAlfa: Cuando = (a, c) => c.alfa && !a.webp.sin_alfa;

export const CONTROLES_WEBP: Control[] = [
  // Básico
  { clave: "calidad", nivel: "basico", de: "webp", marca: "-q", tipo: "deslizador", campo: "calidad", min: 0, max: 100, paso: 1 },
  { clave: "sinPerdida", nivel: "basico", de: "webp", marca: "-lossless", tipo: "interruptor", campo: "sin_perdida" },

  // Avanzado
  {
    clave: "nivelSinPerdida", nivel: "avanzado", de: "webp", marca: "-z", tipo: "especial", especial: "nivelSinPerdida",
    cuando: sinPerdida, porQue: "porQue.soloSinPerdida",
  },
  { clave: "metodo", nivel: "avanzado", de: "webp", marca: "-m", tipo: "deslizador", campo: "metodo", min: 0, max: 6, paso: 1 },
  {
    clave: "casiSinPerdida", nivel: "avanzado", de: "webp", marca: "-near_lossless", tipo: "deslizador", campo: "casi_sin_perdida",
    min: 0, max: 100, paso: 1, cuando: sinPerdida, porQue: "porQue.soloSinPerdida",
  },
  {
    clave: "tamanoObjetivo", nivel: "avanzado", de: "webp", marca: "-size", tipo: "numero", campo: "tamano_objetivo", min: 0, max: 100_000_000,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "yuvNitido", nivel: "avanzado", de: "webp", marca: "-sharp_yuv", tipo: "interruptor", campo: "yuv_nitido",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "autofiltro", nivel: "avanzado", de: "webp", marca: "-af", tipo: "interruptor", campo: "autofiltro",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "calidadAlfa", nivel: "avanzado", de: "webp", marca: "-alpha_q", tipo: "deslizador", campo: "calidad_alfa",
    min: 0, max: 100, paso: 1, cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  { clave: "exacto", nivel: "avanzado", de: "webp", marca: "-exact", tipo: "interruptor", campo: "exacto", cuando: conAlfa, porQue: "porQue.sinAlfa" },
  { clave: "mezclarAlfa", nivel: "avanzado", de: "webp", marca: "-blend_alpha", tipo: "especial", especial: "mezclarAlfa", cuando: conAlfa, porQue: "porQue.sinAlfa" },
  { clave: "sinAlfa", nivel: "avanzado", de: "webp", marca: "-noalpha", tipo: "interruptor", campo: "sin_alfa", cuando: (_o, c) => c.alfa, porQue: "porQue.sinAlfa" },
  {
    clave: "modoRedimension", nivel: "avanzado", de: "webp", marca: "-resize_mode", tipo: "lista", campo: "modo_redimension",
    valores: ["siempre", "solo_reducir", "solo_ampliar"], cuando: (a) => a.webp.redimension !== null, porQue: "porQue.sinRedimension",
  },
  { clave: "metadatos", nivel: "avanzado", de: "webp", marca: "-metadata", tipo: "especial", especial: "metadatos" },

  // Experto
  {
    clave: "segmentos", nivel: "experto", de: "webp", marca: "-segments", tipo: "deslizador", campo: "segmentos", min: 1, max: 4, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "sns", nivel: "experto", de: "webp", marca: "-sns", tipo: "deslizador", campo: "sns", min: 0, max: 100, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "fuerzaFiltro", nivel: "experto", de: "webp", marca: "-f", tipo: "deslizador", campo: "fuerza_filtro", min: 0, max: 100, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "nitidezFiltro", nivel: "experto", de: "webp", marca: "-sharpness", tipo: "deslizador", campo: "nitidez_filtro", min: 0, max: 7, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "filtroFuerte", nivel: "experto", de: "webp", marca: "-strong", tipo: "interruptor", campo: "filtro_fuerte",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "psnrObjetivo", nivel: "experto", de: "webp", marca: "-psnr", tipo: "numero", campo: "psnr_objetivo", min: 0, max: 99,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "pasadas", nivel: "experto", de: "webp", marca: "-pass", tipo: "deslizador", campo: "pasadas", min: 1, max: 10, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "rangoCalidad", nivel: "experto", de: "webp", marca: "-qrange", tipo: "especial", especial: "rangoCalidad", cuando: conPerdida, porQue: "porQue.soloConPerdida" },
  {
    clave: "limiteParticion", nivel: "experto", de: "webp", marca: "-partition_limit", tipo: "deslizador", campo: "limite_particion",
    min: 0, max: 100, paso: 1, cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "pista", nivel: "experto", de: "webp", marca: "-hint", tipo: "lista", campo: "pista", valores: ["ninguna", "photo", "picture", "graph"] },
  {
    clave: "compresionAlfa", nivel: "experto", de: "webp", marca: "-alpha_method", tipo: "lista", campo: "compresion_alfa", valores: ["1", "0"],
    cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  {
    clave: "filtradoAlfa", nivel: "experto", de: "webp", marca: "-alpha_filter", tipo: "lista", campo: "filtrado_alfa", valores: ["fast", "best", "none"],
    cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  {
    clave: "emularJpeg", nivel: "experto", de: "webp", marca: "-jpeg_like", tipo: "interruptor", campo: "emular_jpeg",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "preprocesado", nivel: "experto", de: "webp", marca: "-pre", tipo: "lista", campo: "preprocesado", valores: ["0", "1", "2", "3"],
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "hilos", nivel: "experto", de: "webp", marca: "-mt", tipo: "especial", especial: "hilos" },
  { clave: "pocaMemoria", nivel: "experto", de: "webp", marca: "-low_memory", tipo: "interruptor", campo: "poca_memoria" },
  // cwebp redimensiona y recorta a su manera; lo normal es el proceso, que
  // vale para todos los formatos. Estos son para repetir una orden de cwebp.
  { clave: "redimensionCwebp", nivel: "experto", de: "webp", marca: "-resize", tipo: "especial", especial: "redimensionCwebp" },
  { clave: "recorteCwebp", nivel: "experto", de: "webp", marca: "-crop", tipo: "especial", especial: "recorteCwebp" },
];

export const CONTROLES_JPEG: Control[] = [
  // Básico
  { clave: "calidadJpeg", nivel: "basico", de: "jpeg", marca: "-quality", tipo: "especial", especial: "calidadJpeg" },
  { clave: "colorJpeg", nivel: "basico", de: "jpeg", marca: "-grayscale", tipo: "lista", campo: "color", valores: ["auto", "gris", "rgb"] },
  {
    clave: "escaneoJpeg", nivel: "basico", de: "jpeg", marca: "-baseline", tipo: "lista", campo: "escaneo",
    valores: ["por_defecto", "progresivo", "secuencial"],
  },
  // Avanzado
  {
    clave: "afinadoJpeg", nivel: "avanzado", de: "jpeg", marca: "-tune-*", tipo: "lista", campo: "afinado",
    valores: ["defecto", "hvs_psnr", "psnr", "ssim", "ms_ssim"], nulo: "defecto",
  },
  {
    clave: "muestreoJpeg", nivel: "avanzado", de: "jpeg", marca: "-sample", tipo: "lista", campo: "muestreo",
    valores: ["auto", "1x1", "2x1", "2x2"], nulo: "auto",
  },
  {
    clave: "tablaJpeg", nivel: "avanzado", de: "jpeg", marca: "-quant-table", tipo: "lista", campo: "tabla",
    valores: ["defecto", "0", "1", "2", "3", "4", "5", "6", "7", "8"], nulo: "defecto", numerica: true,
  },
  { clave: "suavizadoJpeg", nivel: "avanzado", de: "jpeg", marca: "-smooth", tipo: "deslizador", campo: "suavizado", min: 0, max: 100, paso: 1 },
  { clave: "sinTrellisJpeg", nivel: "avanzado", de: "jpeg", marca: "-notrellis", tipo: "interruptor", campo: "sin_trellis" },
  { clave: "revertirJpeg", nivel: "avanzado", de: "jpeg", marca: "-revert", tipo: "interruptor", campo: "revertir" },
  // Experto
  { clave: "rapidoJpeg", nivel: "experto", de: "jpeg", marca: "-fastcrush", tipo: "interruptor", campo: "rapido" },
  {
    clave: "dcScanJpeg", nivel: "experto", de: "jpeg", marca: "-dc-scan-opt", tipo: "lista", campo: "dc_scan_opt",
    valores: ["defecto", "0", "1", "2"], nulo: "defecto", numerica: true,
  },
  {
    clave: "dctJpeg", nivel: "experto", de: "jpeg", marca: "-dct", tipo: "lista", campo: "dct",
    valores: ["defecto", "int", "fast", "float"], nulo: "defecto",
  },
  { clave: "tablasBaselineJpeg", nivel: "experto", de: "jpeg", marca: "-quant-baseline", tipo: "interruptor", campo: "tablas_baseline" },
  { clave: "sinOvershootJpeg", nivel: "experto", de: "jpeg", marca: "-noovershoot", tipo: "interruptor", campo: "sin_overshoot" },
  { clave: "sinJfifJpeg", nivel: "experto", de: "jpeg", marca: "-nojfif", tipo: "interruptor", campo: "sin_jfif" },
  { clave: "optimizarJpeg", nivel: "experto", de: "jpeg", marca: "-optimize", tipo: "interruptor", campo: "optimizar" },
];

const conZopfli: Cuando = (a) => a.png.zopfli;
const sinZopfli: Cuando = (a) => !a.png.zopfli;

export const CONTROLES_PNG: Control[] = [
  // Básico
  { clave: "nivelPng", nivel: "basico", de: "png", marca: "-o", tipo: "deslizador", campo: "nivel", min: 0, max: 6, paso: 1 },
  { clave: "alfaPng", nivel: "basico", de: "png", marca: "-a", tipo: "interruptor", campo: "alfa" },
  { clave: "quitarPng", nivel: "basico", de: "png", marca: "--strip", tipo: "lista", campo: "quitar", valores: ["nada", "seguro", "todo"] },
  // Avanzado
  { clave: "entrelazadoPng", nivel: "avanzado", de: "png", marca: "-i", tipo: "lista", campo: "entrelazado", valores: ["no", "si", "mantener"] },
  { clave: "zopfliPng", nivel: "avanzado", de: "png", marca: "-z", tipo: "interruptor", campo: "zopfli" },
  {
    clave: "iteracionesPng", nivel: "avanzado", de: "png", marca: "--zi", tipo: "numero", campo: "iteraciones", min: 1, max: 1000,
    cuando: conZopfli, porQue: "porQue.soloZopfli",
  },
  {
    clave: "compresionPng", nivel: "avanzado", de: "png", marca: "--zc", tipo: "lista", campo: "compresion",
    valores: ["defecto", "0", "4", "8", "10", "11", "12"], nulo: "defecto", numerica: true, cuando: sinZopfli, porQue: "porQue.conZopfli",
  },
  { clave: "rapidoPng", nivel: "avanzado", de: "png", marca: "--fast", tipo: "interruptor", campo: "rapido" },
  // Experto
  { clave: "sinReduccionesPng", nivel: "experto", de: "png", marca: "--nx", tipo: "interruptor", campo: "sin_reducciones" },
  { clave: "sinBitsPng", nivel: "experto", de: "png", marca: "--nb", tipo: "interruptor", campo: "sin_bits" },
  { clave: "sinColorPng", nivel: "experto", de: "png", marca: "--nc", tipo: "interruptor", campo: "sin_color" },
  { clave: "sinPaletaPng", nivel: "experto", de: "png", marca: "--np", tipo: "interruptor", campo: "sin_paleta" },
  { clave: "sinGrisPng", nivel: "experto", de: "png", marca: "--ng", tipo: "interruptor", campo: "sin_gris" },
  { clave: "sinRecodificarPng", nivel: "experto", de: "png", marca: "--nz", tipo: "interruptor", campo: "sin_recodificar" },
  { clave: "escala16Png", nivel: "experto", de: "png", marca: "--scale16", tipo: "interruptor", campo: "escala16" },
  { clave: "forzarPng", nivel: "experto", de: "png", marca: "--force", tipo: "interruptor", campo: "forzar" },
];

/** El proceso: para todos los formatos, y de Apolo (sin opción de la herramienta). */
export const CONTROLES_PROCESO: Control[] = [
  { clave: "enderezar", de: "proceso", marca: "", tipo: "interruptor", campo: "enderezar" },
  { clave: "procesoRedimension", de: "proceso", marca: "", tipo: "especial", especial: "procesoRedimension" },
  { clave: "procesoRecorte", de: "proceso", marca: "", tipo: "especial", especial: "procesoRecorte" },
  { clave: "procesoPaleta", de: "proceso", marca: "", tipo: "especial", especial: "procesoPaleta" },
];

export function controlesDe(f: FormatoSalida): Control[] {
  return f === "webp" ? CONTROLES_WEBP : f === "jpeg" ? CONTROLES_JPEG : f === "png" ? CONTROLES_PNG : [];
}

/** Todos, para las pruebas (i18n y cobertura). */
export const CONTROLES: Control[] = [...CONTROLES_WEBP, ...CONTROLES_JPEG, ...CONTROLES_PNG, ...CONTROLES_PROCESO];

/** Las opciones de cwebp que el panel cubre, para la prueba de cobertura. */
export const OPCIONES_CUBIERTAS = new Set(CONTROLES_WEBP.map((c) => c.marca).filter(Boolean));
