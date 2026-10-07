// El único sitio que habla con Rust (ADR 0013).
//
// Dentro de la aplicación, las órdenes van por Tauri (`invoke`) y los píxeles
// por el protocolo `apolo://`. Abierta en un navegador —`make dev-web`, o las
// pruebas de Playwright—, las mismas órdenes van por HTTP contra `apolo-dev`,
// que es el mismo servicio sin ventana. La interfaz no distingue los dos
// casos: llama a estas funciones y ya.

import { invoke } from "@tauri-apps/api/core";
import type { Ajuste, FormatoSalida, OpcionesWebp, Preset } from "./estudio/opciones";

export interface Motor {
  nombre: string;
  version: string;
}

export interface InfoImagen {
  id: number;
  nombre: string;
  formato: string;
  ancho: number;
  alto: number;
  bytes: number;
  alfa: boolean;
  exif: number | null;
  icc: number | null;
  xmp: number | null;
  orientacion: number;
}

export interface Estadisticas {
  bytes: number;
  psnr: [number, number, number, number, number];
  bloques: [number, number, number];
  bytes_alfa: number;
  tamano_paleta: number;
}

export interface Vista {
  generacion: number;
  /** El lado del comparador: 0 izquierdo, 1 derecho. */
  lado: 0 | 1;
  formato: FormatoSalida;
  /** cwebp, cjpeg, oxipng o qoiconv. */
  herramienta: string;
  bytes: number;
  ancho: number;
  alto: number;
  milisegundos: number;
  /** Solo WebP las da. */
  estadisticas: Estadisticas | null;
  orden: string;
  /** La misma orden con las rutas completas: la que se copia. */
  orden_completa: string;
  equivalente: boolean;
  /** Por qué la orden de la herramienta no da este fichero, si no lo da. */
  motivo: "enderezada" | "procesada" | "formato_sin_herramienta" | null;
}

/** Un preset guardado: un nombre y un ajuste entero (ADR 0020). */
export type PresetGuardado = { nombre: string } & Ajuste;

export interface Inicio {
  /** El ajuste por defecto: WebP, sin proceso, con todas las opciones. */
  ajuste: Ajuste;
  opciones: OpcionesWebp;
  presets_cwebp: Preset[];
  carpeta_presets: string;
  version: string;
}

/** Un error de Rust, con su texto ya en español. */
export interface Fallo {
  mensaje: string;
  cancelado: boolean;
}

export const enTauri = (): boolean => "__TAURI_INTERNALS__" in window;

function comoFallo(e: unknown): Fallo {
  if (e && typeof e === "object" && "mensaje" in e) return e as Fallo;
  return { mensaje: String(e), cancelado: false };
}

async function orden<T>(nombre: string, args: Record<string, unknown> = {}): Promise<T> {
  try {
    if (enTauri()) return await invoke<T>(nombre, args);
    const r = await fetch(`/api/${nombre}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(args),
    });
    if (!r.ok) throw await r.json();
    return (await r.json()) as T;
  } catch (e) {
    throw comoFallo(e);
  }
}

export interface Plataforma {
  /** "macos", "windows", "linux", o "web" en el navegador de desarrollo. */
  sistema: string;
  /** Si la ventana tiene vidrio detrás (ADR 0016): solo macOS, por ahora. */
  vidrio: boolean;
  /** macOS la ejecuta desde una copia de solo lectura: no se puede actualizar sola. */
  traslocada: boolean;
}

export const plataforma = (): Promise<Plataforma> =>
  enTauri()
    ? orden<Plataforma>("plataforma")
    : Promise.resolve({ sistema: "web", vidrio: false, traslocada: false });

/** Lo que se guarda entre sesiones (ADR 0018). */
export interface Ajustes {
  buscar_actualizaciones: boolean;
  /** Segundos Unix. */
  ultima_comprobacion: number | null;
}

export const ajustes = () => orden<Ajustes>("ajustes");
export const buscarActualizaciones = (si: boolean) => orden<Ajustes>("buscar_actualizaciones", { si });
/** Si toca preguntar a GitHub: una vez al día, o ya si `forzar` («Buscar ahora»). */
export const reservarComprobacion = (forzar: boolean) => orden<boolean>("reservar_comprobacion", { forzar });

export const inicio = () => orden<Inicio>("inicio");
export const motores = () => orden<Motor[]>("motores");
export const cerrar = (id: number) => orden<void>("cerrar", { id });
export const codificar = (id: number, ajuste: Ajuste, lado: 0 | 1, generacion: number) =>
  orden<Vista>("codificar", { id, ajuste, lado, generacion });
export const aplicarPreset = (opciones: OpcionesWebp, preset: Preset) =>
  orden<OpcionesWebp>("aplicar_preset", { opciones, preset });
export const nivelSinPerdida = (opciones: OpcionesWebp, nivel: number) =>
  orden<OpcionesWebp>("nivel_sin_perdida", { opciones, nivel });
/** Una orden pegada (de cwebp, cjpeg, oxipng o qoiconv), sobre el ajuste actual. */
export const leerOrden = (texto: string, base: Ajuste) => orden<Ajuste>("leer_orden", { texto, base });
export const presets = () => orden<PresetGuardado[]>("presets");
/** La orden de la herramienta que da un ajuste, sin ficheros. */
export const ordenOpciones = (ajuste: Ajuste) => orden<string>("orden_opciones", { ajuste });
export const guardarPreset = (preset: PresetGuardado) => orden<PresetGuardado[]>("guardar_preset", { preset });
export const borrarPreset = (nombre: string) => orden<PresetGuardado[]>("borrar_preset", { nombre });

export const FORMATOS_ENTRADA = [
  "png", "jpg", "jpeg", "heic", "heif", "avif", "jxl", "webp", "tif", "tiff", "gif", "bmp", "qoi", "ppm", "pgm", "pam", "pnm",
];

/** Abre una imagen de disco (en la aplicación). */
export const abrirRuta = (ruta: string) => orden<InfoImagen>("abrir", { ruta });

/** Abre un fichero que llega del navegador (en desarrollo). */
export async function abrirFichero(f: File): Promise<InfoImagen> {
  const r = await fetch("/api/abrir", {
    method: "POST",
    headers: { "x-nombre": encodeURIComponent(f.name) },
    body: f,
  });
  if (!r.ok) throw comoFallo(await r.json());
  return (await r.json()) as InfoImagen;
}

/** Pide al sistema una imagen. `null` si se cancela. En el navegador, `File`. */
export async function elegirImagen(): Promise<string | File | null> {
  if (enTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const r = await open({ multiple: false, filters: [{ name: "Imágenes", extensions: FORMATOS_ENTRADA }] });
    return typeof r === "string" ? r : null;
  }
  return new Promise((resolver) => {
    const i = document.createElement("input");
    i.type = "file";
    i.accept = FORMATOS_ENTRADA.map((e) => `.${e}`).join(",");
    i.onchange = () => resolver(i.files?.[0] ?? null);
    i.click();
  });
}

/** Lo que llega al soltar ficheros sobre la ventana de la aplicación. */
export async function alSoltar(f: (rutas: string[]) => void): Promise<() => void> {
  if (!enTauri()) return () => {};
  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  return getCurrentWebview().onDragDropEvent((e) => {
    if (e.payload.type === "drop") f(e.payload.paths);
  });
}

/**
 * Exporta el resultado. En la aplicación pregunta dónde con el diálogo del
 * sistema; en el navegador lo descarga. Devuelve el nombre o `null` si se
 * canceló.
 */
const NOMBRE_FORMATO: Record<FormatoSalida, string> = {
  webp: "WebP",
  jpeg: "JPEG",
  png: "PNG",
  qoi: "QOI",
  avif: "AVIF",
  jxl: "JPEG XL",
};
const EXTENSIONES: Record<FormatoSalida, string[]> = {
  webp: ["webp"],
  jpeg: ["jpg", "jpeg"],
  png: ["png"],
  qoi: ["qoi"],
  avif: ["avif"],
  jxl: ["jxl"],
};

export async function exportar(id: number, ajuste: Ajuste, lado: 0 | 1): Promise<string | null> {
  const nombre = await orden<string>("nombre_salida", { id, formato: ajuste.formato });
  if (enTauri()) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const ruta = await save({
      defaultPath: nombre,
      filters: [{ name: NOMBRE_FORMATO[ajuste.formato], extensions: EXTENSIONES[ajuste.formato] }],
    });
    if (!ruta) return null;
    await orden<number>("exportar", { id, ajuste, lado, ruta });
    return ruta;
  }
  const r = await fetch("/api/exportar", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ id, ajuste, lado }),
  });
  if (!r.ok) throw comoFallo(await r.json());
  const url = URL.createObjectURL(await r.blob());
  const a = document.createElement("a");
  a.href = url;
  a.download = nombre;
  a.click();
  URL.revokeObjectURL(url);
  return nombre;
}

function urlPixeles(ruta: string): string {
  if (!enTauri()) return `/pixeles/${ruta}`;
  // WebView2 (Windows) no admite esquemas propios: Tauri los sirve por http.
  return navigator.userAgent.includes("Windows") ? `http://apolo.localhost/${ruta}` : `apolo://localhost/${ruta}`;
}

/** Píxeles crudos: ancho y alto en u32 little-endian, y el RGBA detrás. */
async function pixeles(ruta: string): Promise<ImageData> {
  const r = await fetch(urlPixeles(ruta), { cache: "no-store" });
  if (!r.ok) throw comoFallo(await r.text());
  const b = await r.arrayBuffer();
  const v = new DataView(b);
  const ancho = v.getUint32(0, true);
  const alto = v.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(b, 8, ancho * alto * 4), ancho, alto);
}

export const pixelesOriginal = (id: number, enderezar: boolean) =>
  pixeles(`original/${id}${enderezar ? "?enderezar=1" : ""}`);
export const pixelesResultado = (id: number, lado: 0 | 1) => pixeles(`resultado/${id}?lado=${lado}`);

// ---------------------------------------------------------------- lotes (ADR 0019)

/** Lo que hay en lo soltado, antes de convertir. */
export interface Recogida {
  imagenes: number;
  bytes: number;
  salida_sugerida: string | null;
  muestra: string[];
}

export interface LoteEmpezado {
  id: number;
  total: number;
  salida: string;
}

export interface FilaSalida {
  formato: FormatoSalida;
  bytes: number;
  ruta: string;
}

export interface Fila {
  relativa: string;
  bytes_entrada: number;
  /** Los ficheros que dejó, el más ligero primero. */
  salidas: FilaSalida[];
  error: string | null;
}

export interface Destacada {
  relativa: string;
  formato: FormatoSalida;
  bytes_entrada: number;
  bytes_salida: number;
}

export interface PorFormato {
  formato: FormatoSalida;
  ficheros: number;
  bytes_entrada: number;
  bytes_salida: number;
}

export interface Resumen {
  convertidas: number;
  fallidas: number;
  mayores: number;
  bytes_entrada: number;
  bytes_salida: number;
  por_formato: PorFormato[];
  peores: Destacada[];
}

export interface EstadoLote {
  total: number;
  hechas: number;
  nuevas: Fila[];
  terminado: boolean;
  cancelado: boolean;
  segundos: number;
  resumen: Resumen | null;
}

export const recogerLote = (entradas: string[]) => orden<Recogida>("recoger_lote", { entradas });
export const empezarLote = (entradas: string[], ajustes: Ajuste[], soloMasLigero: boolean, salida: string) =>
  orden<LoteEmpezado>("empezar_lote", { entradas, ajustes, soloMasLigero, salida });
export const estadoLote = (id: number, desde: number) => orden<EstadoLote>("estado_lote", { id, desde });
export const cancelarLote = (id: number) => orden<void>("cancelar_lote", { id });

/** Pide carpetas o imágenes al sistema. En el navegador no hay rutas: `[]`. */
export async function elegirEntradas(carpetas: boolean): Promise<string[]> {
  if (!enTauri()) return [];
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open(
    carpetas
      ? { multiple: true, directory: true }
      : { multiple: true, filters: [{ name: "Imágenes", extensions: FORMATOS_ENTRADA }] },
  );
  return r === null ? [] : Array.isArray(r) ? r : [r];
}

/** Pide la carpeta de salida. `null` si se cancela o en el navegador. */
export async function elegirSalida(propuesta: string | null): Promise<string | null> {
  if (!enTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({ directory: true, multiple: false, defaultPath: propuesta ?? undefined });
  return typeof r === "string" ? r : null;
}

/** Enseña un fichero en el Finder o el explorador. */
export async function mostrarEnCarpeta(ruta: string): Promise<void> {
  if (!enTauri()) return;
  const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
  await revealItemInDir(ruta);
}
