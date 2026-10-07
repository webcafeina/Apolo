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

// Como la de verdad: 11 MB en trozos de 16 KB, cientos por segundo y a ritmo
// desigual. Con diez pasos tranquilos, la barra parecía ir bien y en el Mac
// iba a tirones (v0.4.0).
function deMentira(version: string): Novedad {
  return {
    version,
    descargar: (alAvanzar) =>
      new Promise((listo) => {
        const total = 11_143_568;
        let bytes = 0;
        alAvanzar({ bytes, total });
        const reloj = setInterval(() => {
          const rafaga = 1 + Math.floor(Math.random() * 6);
          for (let i = 0; i < rafaga && bytes < total; i++) {
            bytes = Math.min(total, bytes + 16_384);
            alAvanzar({ bytes, total });
          }
          if (bytes === total) {
            clearInterval(reloj);
            listo();
          }
        }, 4);
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
