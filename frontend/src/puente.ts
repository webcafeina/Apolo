// El único sitio que habla con Rust (ADR 0013).
//
// Dentro de la aplicación, las órdenes van por Tauri (`invoke`) y los píxeles
// por el protocolo `apolo://`. Abierta en un navegador —`make dev-web`, o las
// pruebas de Playwright—, las mismas órdenes van por HTTP contra `apolo-dev`,
// que es el mismo servicio sin ventana. La interfaz no distingue los dos
// casos: llama a estas funciones y ya.

import { invoke } from "@tauri-apps/api/core";
import type { OpcionesWebp, Preset } from "./estudio/opciones";

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
  bytes: number;
  ancho: number;
  alto: number;
  milisegundos: number;
  estadisticas: Estadisticas;
  orden: string;
  /** La misma orden con las rutas completas: la que se copia. */
  orden_completa: string;
  equivalente: boolean;
  /** Por qué la orden cwebp no da este fichero, si no lo da. */
  motivo: "enderezada" | "formato_sin_cwebp" | null;
}

export interface PresetGuardado {
  nombre: string;
  formato: string;
  webp: OpcionesWebp;
}

export interface Inicio {
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
}

export const plataforma = (): Promise<Plataforma> =>
  enTauri() ? orden<Plataforma>("plataforma") : Promise.resolve({ sistema: "web", vidrio: false });

export const inicio = () => orden<Inicio>("inicio");
export const motores = () => orden<Motor[]>("motores");
export const cerrar = (id: number) => orden<void>("cerrar", { id });
export const codificar = (id: number, opciones: OpcionesWebp, generacion: number) =>
  orden<Vista>("codificar", { id, opciones, generacion });
export const aplicarPreset = (opciones: OpcionesWebp, preset: Preset) =>
  orden<OpcionesWebp>("aplicar_preset", { opciones, preset });
export const nivelSinPerdida = (opciones: OpcionesWebp, nivel: number) =>
  orden<OpcionesWebp>("nivel_sin_perdida", { opciones, nivel });
export const leerOrden = (texto: string) => orden<OpcionesWebp>("leer_orden", { texto });
export const presets = () => orden<PresetGuardado[]>("presets");
export const ordenOpciones = (opciones: OpcionesWebp) => orden<string>("orden_opciones", { opciones });
export const guardarPreset = (preset: PresetGuardado) => orden<PresetGuardado[]>("guardar_preset", { preset });
export const borrarPreset = (nombre: string) => orden<PresetGuardado[]>("borrar_preset", { nombre });

export const FORMATOS_ENTRADA = ["png", "jpg", "jpeg", "heic", "heif", "webp", "tif", "tiff", "gif", "bmp", "qoi", "ppm", "pgm", "pam", "pnm"];

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
export async function exportar(id: number, opciones: OpcionesWebp): Promise<string | null> {
  const nombre = await orden<string>("nombre_salida", { id });
  if (enTauri()) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const ruta = await save({ defaultPath: nombre, filters: [{ name: "WebP", extensions: ["webp"] }] });
    if (!ruta) return null;
    await orden<number>("exportar", { id, opciones, ruta });
    return ruta;
  }
  const r = await fetch("/api/exportar", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ id, opciones }),
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
export const pixelesResultado = (id: number) => pixeles(`resultado/${id}`);
