// El único sitio que habla con Rust. Dentro de la aplicación va por Tauri;
// abierta en un navegador (pnpm dev, Playwright) no hay Rust detrás y cada
// llamada contesta lo que tenga sentido sin él.
//
// En la entrega 2 se le añade el segundo camino, por HTTP contra un servidor
// de desarrollo, como el puente de Esfinge (su ADR 0009).
import { invoke } from "@tauri-apps/api/core";

export interface Motor {
  nombre: string;
  version: string;
}

export const enTauri = (): boolean => "__TAURI_INTERNALS__" in window;

export async function motores(): Promise<Motor[]> {
  if (!enTauri()) return [];
  return invoke<Motor[]>("motores");
}
