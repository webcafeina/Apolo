// La elección de cookies (ADR 0033): se guarda en este navegador (localStorage,
// no en una cookie) con su versión y su fecha, y se vuelve a preguntar a los 12
// meses o si cambia la versión. Aceptar la analítica actualiza Consent Mode y
// carga GTM; rechazarla la deniega y borra las cookies de Google Analytics.
export const CLAVE = "apolo-consentimiento";
export const VERSION = 1;
const DOCE_MESES = 365 * 864e5;

export interface Eleccion {
  v: number;
  analitica: boolean;
  fecha: number;
}

/** La elección guardada, si sigue valiendo. */
export function vigente(texto: string | null, ahora = Date.now()): Eleccion | null {
  try {
    const c = JSON.parse(texto ?? "null") as Eleccion | null;
    return c && c.v === VERSION && typeof c.analitica === "boolean" && ahora - c.fecha < DOCE_MESES ? c : null;
  } catch {
    return null;
  }
}

/** Los nombres de las cookies de Google Analytics que hay que borrar al retirar el permiso. */
export const deAnalitica = (cookies: string) =>
  cookies
    .split(";")
    .map((c) => c.trim().split("=")[0])
    .filter((n) => n === "_ga" || n.startsWith("_ga_") || n === "_gid");

declare global {
  interface Window {
    gtag?: (...a: unknown[]) => void;
    apoloCargarGtm?: () => void;
  }
}

export function iniciar() {
  const aviso = document.querySelector<HTMLElement>("[data-cookies]");
  if (!aviso) return;
  const raiz = document.documentElement;
  const casilla = aviso.querySelector<HTMLInputElement>("[data-analitica]")!;
  const configurar = aviso.querySelector<HTMLElement>("[data-configurar]")!;
  const botonGuardar = aviso.querySelector<HTMLElement>("[data-cookies-guardar]")!;
  const botonConfigurar = aviso.querySelector<HTMLElement>("[data-cookies-configurar]")!;
  let leida: string | null = null;
  try {
    leida = localStorage.getItem(CLAVE);
  } catch {
    // Sin almacenamiento: se pregunta en cada visita.
  }
  const actual = vigente(leida);

  const abrir = () => {
    casilla.checked = vigente(leidaAhora())?.analitica ?? false;
    aviso.hidden = false;
    aviso.querySelector<HTMLButtonElement>("button")?.focus();
  };
  const leidaAhora = () => {
    try {
      return localStorage.getItem(CLAVE);
    } catch {
      return null;
    }
  };
  const guardar = (analitica: boolean) => {
    const e: Eleccion = { v: VERSION, analitica, fecha: Date.now() };
    try {
      localStorage.setItem(CLAVE, JSON.stringify(e));
    } catch {
      // Sin almacenamiento, vale para esta visita.
    }
    window.gtag?.("consent", "update", { analytics_storage: analitica ? "granted" : "denied" });
    if (analitica) window.apoloCargarGtm?.();
    else
      for (const n of deAnalitica(document.cookie))
        for (const dominio of ["", location.hostname, `.${location.hostname.split(".").slice(-2).join(".")}`])
          document.cookie = `${n}=; Max-Age=0; path=/${dominio ? `; domain=${dominio}` : ""}`;
    (window.dataLayer ??= []).push({ event: "consentimiento_actualizado", analitica });
    raiz.setAttribute("data-consentimiento", "dado");
    aviso.hidden = true;
    configurar.hidden = true;
    botonGuardar.hidden = true;
    botonConfigurar.hidden = false;
  };

  aviso.querySelector("[data-cookies-aceptar]")!.addEventListener("click", () => guardar(true));
  aviso.querySelector("[data-cookies-rechazar]")!.addEventListener("click", () => guardar(false));
  botonConfigurar.addEventListener("click", () => {
    configurar.hidden = false;
    botonConfigurar.hidden = true;
    botonGuardar.hidden = false;
    casilla.focus();
  });
  botonGuardar.addEventListener("click", () => guardar(casilla.checked));
  aviso.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && raiz.hasAttribute("data-consentimiento")) aviso.hidden = true;
  });
  document.querySelectorAll("[data-preferencias-cookies]").forEach((b) => {
    (b as HTMLElement).hidden = false;
    b.addEventListener("click", abrir);
  });
  if (!actual) aviso.hidden = false;
}
