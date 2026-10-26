// Qué ordenador tiene quien visita la web, para ofrecerle su descarga (ADR
// 0033). Una función pura, que se prueba con navegadores de verdad
// (pruebas/plataforma.test.ts). La distribución de Linux no la dice ningún
// navegador: se elige en la tarjeta.

export type Plataforma = "macos" | "windows" | "linux" | "movil" | "chromeos" | "otro";
export type Arquitectura = "x64" | "arm64";

export interface Datos {
  ua: string;
  /** `navigator.userAgentData.platform`, si lo hay (Chromium). */
  plataforma?: string;
  /** `navigator.userAgentData.mobile`. */
  movil?: boolean;
  /** `navigator.maxTouchPoints`: el iPad se presenta como un Mac. */
  toques?: number;
  /** `getHighEntropyValues(["architecture"])`, solo en Chromium: «x86» o «arm». */
  arquitectura?: string;
  bitness?: string;
}

export interface Detectada {
  plataforma: Plataforma;
  arquitectura: Arquitectura;
  /** Falso si la arquitectura es una suposición (x64, la más común). */
  segura: boolean;
}

export function detectar(d: Datos): Detectada {
  const { ua } = d;
  const plat = d.plataforma ?? "";
  const arm = d.arquitectura === "arm";
  const suposicion = (plataforma: Plataforma): Detectada => ({ plataforma, arquitectura: "x64", segura: false });

  if (d.movil || /Android|iPhone|iPod/.test(ua)) return suposicion("movil");
  if (/Macintosh/.test(ua) && (d.toques ?? 0) > 1) return suposicion("movil");
  if (/CrOS/.test(ua) || plat === "Chrome OS") return suposicion("chromeos");
  if (plat === "macOS" || /Macintosh|Mac OS X/.test(ua)) return { plataforma: "macos", arquitectura: "x64", segura: true };
  if (plat === "Windows" || /Windows/.test(ua)) {
    if (d.arquitectura) return { plataforma: "windows", arquitectura: arm ? "arm64" : "x64", segura: true };
    if (/ARM64|aarch64/i.test(ua)) return { plataforma: "windows", arquitectura: "arm64", segura: true };
    return suposicion("windows");
  }
  if (plat === "Linux" || /Linux|X11/.test(ua)) {
    if (d.arquitectura) return { plataforma: "linux", arquitectura: arm ? "arm64" : "x64", segura: true };
    if (/aarch64|arm64/i.test(ua)) return { plataforma: "linux", arquitectura: "arm64", segura: true };
    return suposicion("linux");
  }
  return suposicion("otro");
}
