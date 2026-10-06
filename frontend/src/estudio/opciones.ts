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

export type Nivel = "basico" | "avanzado" | "experto";
export const NIVELES: Nivel[] = ["basico", "avanzado", "experto"];

/** Lo que hace falta saber de la imagen para decidir qué controles tienen sentido. */
export interface Contexto {
  alfa: boolean;
  ancho: number;
  alto: number;
}

type Cuando = (o: OpcionesWebp, c: Contexto) => boolean;

interface Base {
  clave: string;
  nivel: Nivel;
  /** La opción de cwebp, para enseñarla junto a la etiqueta. */
  cwebp: string;
  /** Si no se cumple, el control se ve apagado y explica por qué. */
  cuando?: Cuando;
  /** Clave i18n de por qué está apagado. */
  porQue?: string;
}

export type Control =
  | (Base & {
      tipo: "deslizador";
      campo: keyof OpcionesWebp;
      min: number;
      max: number;
      paso: number;
    })
  | (Base & { tipo: "interruptor"; campo: keyof OpcionesWebp })
  | (Base & { tipo: "lista"; campo: keyof OpcionesWebp; valores: string[] })
  | (Base & { tipo: "numero"; campo: keyof OpcionesWebp; min: number; max: number })
  | (Base & {
      tipo: "especial";
      especial: "nivelSinPerdida" | "recorte" | "redimension" | "rangoCalidad" | "mezclarAlfa" | "metadatos" | "hilos";
    });

const conPerdida: Cuando = (o) => !o.sin_perdida;
const sinPerdida: Cuando = (o) => o.sin_perdida;
const conAlfa: Cuando = (o, c) => c.alfa && !o.sin_alfa;

export const CONTROLES: Control[] = [
  // Básico
  { clave: "calidad", nivel: "basico", cwebp: "-q", tipo: "deslizador", campo: "calidad", min: 0, max: 100, paso: 1 },
  { clave: "sinPerdida", nivel: "basico", cwebp: "-lossless", tipo: "interruptor", campo: "sin_perdida" },
  { clave: "redimension", nivel: "basico", cwebp: "-resize", tipo: "especial", especial: "redimension" },
  { clave: "recorte", nivel: "basico", cwebp: "-crop", tipo: "especial", especial: "recorte" },
  { clave: "enderezar", nivel: "basico", cwebp: "", tipo: "interruptor", campo: "enderezar" },

  // Avanzado
  {
    clave: "nivelSinPerdida", nivel: "avanzado", cwebp: "-z", tipo: "especial", especial: "nivelSinPerdida",
    cuando: sinPerdida, porQue: "porQue.soloSinPerdida",
  },
  { clave: "metodo", nivel: "avanzado", cwebp: "-m", tipo: "deslizador", campo: "metodo", min: 0, max: 6, paso: 1 },
  {
    clave: "casiSinPerdida", nivel: "avanzado", cwebp: "-near_lossless", tipo: "deslizador", campo: "casi_sin_perdida",
    min: 0, max: 100, paso: 1, cuando: sinPerdida, porQue: "porQue.soloSinPerdida",
  },
  {
    clave: "tamanoObjetivo", nivel: "avanzado", cwebp: "-size", tipo: "numero", campo: "tamano_objetivo", min: 0, max: 100_000_000,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "yuvNitido", nivel: "avanzado", cwebp: "-sharp_yuv", tipo: "interruptor", campo: "yuv_nitido",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "autofiltro", nivel: "avanzado", cwebp: "-af", tipo: "interruptor", campo: "autofiltro",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "calidadAlfa", nivel: "avanzado", cwebp: "-alpha_q", tipo: "deslizador", campo: "calidad_alfa",
    min: 0, max: 100, paso: 1, cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  { clave: "exacto", nivel: "avanzado", cwebp: "-exact", tipo: "interruptor", campo: "exacto", cuando: conAlfa, porQue: "porQue.sinAlfa" },
  { clave: "mezclarAlfa", nivel: "avanzado", cwebp: "-blend_alpha", tipo: "especial", especial: "mezclarAlfa", cuando: conAlfa, porQue: "porQue.sinAlfa" },
  { clave: "sinAlfa", nivel: "avanzado", cwebp: "-noalpha", tipo: "interruptor", campo: "sin_alfa", cuando: (_o, c) => c.alfa, porQue: "porQue.sinAlfa" },
  {
    clave: "modoRedimension", nivel: "avanzado", cwebp: "-resize_mode", tipo: "lista", campo: "modo_redimension",
    valores: ["siempre", "solo_reducir", "solo_ampliar"], cuando: (o) => o.redimension !== null, porQue: "porQue.sinRedimension",
  },
  { clave: "metadatos", nivel: "avanzado", cwebp: "-metadata", tipo: "especial", especial: "metadatos" },

  // Experto
  {
    clave: "segmentos", nivel: "experto", cwebp: "-segments", tipo: "deslizador", campo: "segmentos", min: 1, max: 4, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "sns", nivel: "experto", cwebp: "-sns", tipo: "deslizador", campo: "sns", min: 0, max: 100, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "fuerzaFiltro", nivel: "experto", cwebp: "-f", tipo: "deslizador", campo: "fuerza_filtro", min: 0, max: 100, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "nitidezFiltro", nivel: "experto", cwebp: "-sharpness", tipo: "deslizador", campo: "nitidez_filtro", min: 0, max: 7, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "filtroFuerte", nivel: "experto", cwebp: "-strong", tipo: "interruptor", campo: "filtro_fuerte",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "psnrObjetivo", nivel: "experto", cwebp: "-psnr", tipo: "numero", campo: "psnr_objetivo", min: 0, max: 99,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "pasadas", nivel: "experto", cwebp: "-pass", tipo: "deslizador", campo: "pasadas", min: 1, max: 10, paso: 1,
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "rangoCalidad", nivel: "experto", cwebp: "-qrange", tipo: "especial", especial: "rangoCalidad", cuando: conPerdida, porQue: "porQue.soloConPerdida" },
  {
    clave: "limiteParticion", nivel: "experto", cwebp: "-partition_limit", tipo: "deslizador", campo: "limite_particion",
    min: 0, max: 100, paso: 1, cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "pista", nivel: "experto", cwebp: "-hint", tipo: "lista", campo: "pista", valores: ["ninguna", "photo", "picture", "graph"] },
  {
    clave: "compresionAlfa", nivel: "experto", cwebp: "-alpha_method", tipo: "lista", campo: "compresion_alfa", valores: ["1", "0"],
    cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  {
    clave: "filtradoAlfa", nivel: "experto", cwebp: "-alpha_filter", tipo: "lista", campo: "filtrado_alfa", valores: ["fast", "best", "none"],
    cuando: conAlfa, porQue: "porQue.sinAlfa",
  },
  {
    clave: "emularJpeg", nivel: "experto", cwebp: "-jpeg_like", tipo: "interruptor", campo: "emular_jpeg",
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  {
    clave: "preprocesado", nivel: "experto", cwebp: "-pre", tipo: "lista", campo: "preprocesado", valores: ["0", "1", "2", "3"],
    cuando: conPerdida, porQue: "porQue.soloConPerdida",
  },
  { clave: "hilos", nivel: "experto", cwebp: "-mt", tipo: "especial", especial: "hilos" },
  { clave: "pocaMemoria", nivel: "experto", cwebp: "-low_memory", tipo: "interruptor", campo: "poca_memoria" },
];

/** Las opciones de cwebp que el panel cubre, para la prueba de cobertura. */
export const OPCIONES_CUBIERTAS = new Set(CONTROLES.map((c) => c.cwebp).filter(Boolean));
