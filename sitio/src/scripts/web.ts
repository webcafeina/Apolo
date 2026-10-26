// El comportamiento de la web (ADR 0033). Sin JavaScript la página se lee
// entera y las descargas funcionan (macOS, x64 y Ubuntu por defecto); esto
// añade lo que se elige, el comparador, el vídeo y los eventos de la medición.
// Lo que se elige se recuerda en este navegador (localStorage), sin cookies.

declare global {
  interface Window {
    dataLayer?: Record<string, unknown>[];
  }
  interface Navigator {
    userAgentData?: { platform?: string; getHighEntropyValues?: (h: string[]) => Promise<{ architecture?: string }> };
  }
}

const raiz = document.documentElement;
const guardar = (k: string, v: string) => {
  try {
    localStorage.setItem(k, v);
  } catch {
    // Sin almacenamiento, se elige igual; no se recordará.
  }
};
const leer = (k: string) => {
  try {
    return localStorage.getItem(k);
  } catch {
    return null;
  }
};

/** Un evento para la medición: GTM decide si cuenta, según el consentimiento (F7). */
export function evento(nombre: string, datos: Record<string, unknown> = {}) {
  (window.dataLayer ??= []).push({ event: nombre, ...datos });
}

// ── Lo que se elige: vista, pantalla, arquitectura y distribución ──────────
type Eleccion = "vista" | "pantalla" | "arquitectura" | "distro";
const actual = (e: Eleccion, grupo: Element) =>
  e === "pantalla" ? (grupo.closest("[data-pantalla]")?.getAttribute("data-pantalla") ?? "") : (raiz.getAttribute(`data-${e}`) ?? "");

function marcar() {
  document.querySelectorAll<HTMLElement>("[data-elige]").forEach((g) => {
    const v = actual(g.dataset.elige as Eleccion, g);
    g.querySelectorAll<HTMLButtonElement>("[data-valor]").forEach((b) => {
      const si = b.dataset.valor === v;
      b.setAttribute("aria-checked", String(si));
      b.tabIndex = si ? 0 : -1;
    });
  });
}

function elegir(g: HTMLElement, valor: string) {
  const e = g.dataset.elige as Eleccion;
  if (e === "pantalla") g.closest("[data-pantalla]")?.setAttribute("data-pantalla", valor);
  else {
    raiz.setAttribute(`data-${e}`, valor);
    guardar(`apolo-${e}`, valor);
    if (e === "arquitectura") raiz.setAttribute("data-arquitectura-segura", "");
    if (e === "vista") evento("cambiar_vista", { vista: valor });
  }
  marcar();
}

document.addEventListener("click", (ev) => {
  const b = (ev.target as Element).closest<HTMLButtonElement>("[data-elige] [data-valor]");
  if (b) elegir(b.closest<HTMLElement>("[data-elige]")!, b.dataset.valor!);
});
// Las flechas mueven la elección dentro de un grupo, como en un radiogroup.
document.addEventListener("keydown", (ev) => {
  const b = (ev.target as Element).closest<HTMLButtonElement>("[data-elige] [data-valor]");
  if (!b || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(ev.key)) return;
  ev.preventDefault();
  const g = b.closest<HTMLElement>("[data-elige]")!;
  const todos = [...g.querySelectorAll<HTMLButtonElement>("[data-valor]")];
  const paso = ev.key === "ArrowLeft" || ev.key === "ArrowUp" ? -1 : 1;
  const sig = todos[(todos.indexOf(b) + paso + todos.length) % todos.length];
  elegir(g, sig.dataset.valor!);
  sig.focus();
});
marcar();

// ── La arquitectura de verdad: Chromium la dice si se pregunta ─────────────
const plataforma = raiz.getAttribute("data-plataforma");
if (leer("apolo-arquitectura") || /ARM64|aarch64/i.test(navigator.userAgent)) raiz.setAttribute("data-arquitectura-segura", "");
else if ((plataforma === "windows" || plataforma === "linux") && navigator.userAgentData?.getHighEntropyValues) {
  navigator.userAgentData
    .getHighEntropyValues(["architecture", "bitness"])
    .then(({ architecture }) => {
      if (!architecture) return;
      raiz.setAttribute("data-arquitectura", architecture === "arm" ? "arm64" : "x64");
      raiz.setAttribute("data-arquitectura-segura", "");
      marcar();
    })
    .catch(() => {});
}

// ── Copiar órdenes y el enlace ─────────────────────────────────────────────
async function copiar(b: HTMLButtonElement, texto: string) {
  const antes = b.textContent;
  try {
    await navigator.clipboard.writeText(texto);
    b.textContent = b.dataset.hecho ?? antes;
    b.setAttribute("data-copiado", "");
    setTimeout(() => {
      b.textContent = antes;
      b.removeAttribute("data-copiado");
    }, 1800);
  } catch {
    // Sin portapapeles, se selecciona para copiarlo a mano.
    const pre = b.parentElement?.querySelector("pre");
    if (pre) getSelection()?.selectAllChildren(pre);
  }
}
document.addEventListener("click", (ev) => {
  const b = (ev.target as Element).closest<HTMLButtonElement>("[data-copiar]");
  if (!b) return;
  void copiar(b, b.dataset.copiar!);
  evento("copiar_comando", { comando: b.dataset.comando });
});
document.querySelector<HTMLButtonElement>("[data-compartir]")?.addEventListener("click", async () => {
  try {
    await navigator.share({ title: document.title, url: location.href });
  } catch {
    // Cancelado o sin compartir: queda el botón de copiar.
  }
});
document.querySelector<HTMLButtonElement>("[data-copiar-enlace]")?.addEventListener("click", (ev) => {
  void copiar(ev.currentTarget as HTMLButtonElement, location.href);
});

// ── Lo que se mide al pulsar ───────────────────────────────────────────────
document.addEventListener("click", (ev) => {
  const a = (ev.target as Element).closest<HTMLAnchorElement>("a");
  if (!a) return;
  if (a.dataset.descarga) {
    const [sistema, arquitectura, formato] = a.dataset.descarga.split("|");
    const origen = a.closest("section")?.id || "portada";
    evento("descarga_apolo", { sistema, arquitectura, formato, origen });
  } else if (a.hasAttribute("data-webcafeina")) evento("clic_webcafeina");
  else if (a.hreflang && a.closest(".idioma")) evento("cambiar_idioma", { idioma: a.hreflang });
});

// ── El comparador ──────────────────────────────────────────────────────────
document.querySelectorAll<HTMLElement>("[data-comparador]").forEach((c) => {
  const rango = c.querySelector<HTMLInputElement>('input[type="range"]')!;
  rango.addEventListener("input", () => c.style.setProperty("--corte", `${rango.value}%`));
  const jxl = c.querySelector<HTMLImageElement>("img[data-jxl]");
  const avisoJxl = c.querySelector<HTMLElement>(".sin-jxl");
  const formatos = c.querySelectorAll<HTMLButtonElement>(".formatos [data-formato]");
  formatos.forEach((b) =>
    b.addEventListener("click", () => {
      const f = b.dataset.formato!;
      c.dataset.formato = f;
      formatos.forEach((x) => x.setAttribute("aria-checked", String(x === b)));
      // Si el navegador no pinta JPEG XL, se ve el original: se dice.
      const sinJxl = f === "jxl" && jxl !== null && !jxl.currentSrc.endsWith(".jxl");
      if (avisoJxl) avisoJxl.hidden = !sinJxl;
    }),
  );
});

// ── El vídeo: arranca al verse, salvo que se pida menos movimiento ─────────
const quieto = matchMedia("(prefers-reduced-motion: reduce)").matches;
document.querySelectorAll<HTMLElement>("[data-video]").forEach((caja) => {
  const v = caja.querySelector("video")!;
  const boton = caja.querySelector<HTMLButtonElement>(".pausa")!;
  const pintar = () => {
    caja.toggleAttribute("data-reproduciendo", !v.paused);
    boton.setAttribute("aria-label", (v.paused ? boton.dataset.seguir : boton.dataset.pausa) ?? "");
  };
  v.addEventListener("play", pintar);
  v.addEventListener("pause", pintar);
  let parado = quieto;
  boton.addEventListener("click", () => {
    if (v.paused) {
      parado = false;
      void v.play();
    } else {
      parado = true;
      v.pause();
    }
  });
  new IntersectionObserver(([e]) => {
    if (e.isIntersecting && !parado) void v.play().catch(() => {});
    else if (!e.isIntersecting) v.pause();
  }, { threshold: 0.4 }).observe(v);
});

// ── Lo que entra en pantalla sube un poco (desde visible) ──────────────────
if (!quieto && "IntersectionObserver" in window) {
  const io = new IntersectionObserver(
    (es) =>
      es.forEach((e) => {
        if (e.isIntersecting) {
          (e.target as HTMLElement).dataset.aparece = "dentro";
          io.unobserve(e.target);
        }
      }),
    { rootMargin: "0px 0px -8% 0px" },
  );
  document.querySelectorAll<HTMLElement>("[data-aparece]").forEach((el) => {
    if (el.getBoundingClientRect().top > innerHeight) {
      el.dataset.aparece = "fuera";
      io.observe(el);
    }
  });
}
