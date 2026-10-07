// Actualizarse desde la propia aplicación (ADR 0018), como Esfinge: una vez al
// día se pregunta a GitHub si hay una versión nueva; si la hay, una banda lo
// dice, y en dos pasos —«Descargar» y «Instalar y reiniciar»— se instala sin
// volver a pasar por el instalador.
//
// Quien baja, comprueba la firma e instala es tauri-plugin-updater. Aquí solo
// está la puerta de las 24 horas (en Rust, `reservar_comprobacion`) y una capa
// fina que deja probar la banda en el navegador: con `?novedad=9.9.9` en la URL
// aparece una versión nueva de mentira que se «descarga» sola.

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { enTauri, reservarComprobacion } from "./puente";

export interface Avance {
  bytes: number;
  /** 0 si no se sabe el tamaño. */
  total: number;
}

export interface Novedad {
  version: string;
  descargar(alAvanzar: (a: Avance) => void): Promise<void>;
  /** Instala lo descargado y reinicia. En Windows el instalador cierra Apolo. */
  instalarYReiniciar(): Promise<void>;
}

/** Cada cuánto se asoma la aplicación; la puerta de Rust decide si pregunta. */
export const CADA_HORA = 60 * 60 * 1000;

const simulada = (): string | null => new URLSearchParams(window.location.search).get("novedad");

/** Si se puede buscar desde aquí: en la ventana, o en el navegador con la simulada. */
export const sePuedeBuscar = (): boolean => enTauri() || simulada() !== null;

function deTauri(u: Update): Novedad {
  return {
    version: u.version,
    async descargar(alAvanzar) {
      let bytes = 0;
      let total = 0;
      await u.download((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        else if (e.event === "Progress") bytes += e.data.chunkLength;
        alAvanzar({ bytes, total });
      });
    },
    async instalarYReiniciar() {
      await u.install();
      await relaunch();
    },
  };
}

function deMentira(version: string): Novedad {
  return {
    version,
    descargar: (alAvanzar) =>
      new Promise((listo) => {
        const total = 10;
        let bytes = 0;
        const reloj = setInterval(() => {
          bytes++;
          alAvanzar({ bytes, total });
          if (bytes === total) {
            clearInterval(reloj);
            listo();
          }
        }, 60);
      }),
    instalarYReiniciar: () => new Promise(() => {}),
  };
}

/**
 * Pregunta si hay una versión nueva. Sin `forzar`, solo si la casilla está
 * puesta y han pasado 24 horas; `null` si no toca o no la hay. Los errores de
 * red salen: la comprobación automática los calla y «Buscar ahora» los enseña.
 */
export async function buscar(forzar: boolean): Promise<Novedad | null> {
  if (!(await reservarComprobacion(forzar))) return null;
  const v = simulada();
  if (v !== null) return deMentira(v);
  if (!enTauri()) return null;
  const u = await check();
  return u ? deTauri(u) : null;
}
