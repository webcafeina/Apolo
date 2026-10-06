// El comparador: original y resultado sobre el mismo lienzo, con deslizador
// o lado a lado, zoom con la rueda (alrededor del cursor) y arrastre para
// moverse. Los dos lados comparten la vista: lo que se acerca en uno se
// acerca en el otro.
//
// Pinta píxeles crudos (ImageData), no <img>: así ninguno de los dos lados
// pasa por la gestión de color del navegador y se compara lo que hay en los
// ficheros (ADR 0013). Al acercarse más del 100 % los píxeles se ven como
// cuadros, sin suavizar, que es lo que hace falta para juzgar artefactos.

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export type Modo = "deslizador" | "ladoALado";

interface Props {
  original: ImageData | null;
  resultado: ImageData | null;
  modo: Modo;
  ocupado: boolean;
}

interface Vista {
  /** Píxeles de pantalla (CSS) por píxel de imagen. */
  escala: number;
  /** Dónde cae la esquina de la imagen, en píxeles CSS del lienzo. */
  x: number;
  y: number;
}

function aBitmap(d: ImageData | null): Promise<ImageBitmap | null> {
  return d ? createImageBitmap(d) : Promise.resolve(null);
}

function color(nombre: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(nombre).trim() || "#ccc";
}

export function Comparador({ original, resultado, modo, ocupado }: Props) {
  const { t } = useTranslation();
  const caja = useRef<HTMLDivElement>(null);
  const lienzo = useRef<HTMLCanvasElement>(null);
  const [bitmaps, setBitmaps] = useState<[ImageBitmap | null, ImageBitmap | null]>([null, null]);
  const [tamano, setTamano] = useState({ w: 0, h: 0 });
  const [vista, setVista] = useState<Vista | null>(null);
  const [corte, setCorte] = useState(0.5);
  const arrastre = useRef<null | { tipo: "corte" } | { tipo: "mover"; x: number; y: number; v: Vista }>(null);

  // Las dimensiones de referencia son las del original: el resultado se
  // estira a ellas si tiene la misma proporción (redimensionar), y si no
  // (recortar), los dos van lado a lado.
  const ref = original ?? resultado;
  const mismaProporcion =
    !original || !resultado || Math.abs(original.width / original.height - resultado.width / resultado.height) < 0.01;
  const modoReal: Modo = mismaProporcion ? modo : "ladoALado";

  useEffect(() => {
    let vivo = true;
    void Promise.all([aBitmap(original), aBitmap(resultado)]).then((b) => vivo && setBitmaps(b as never));
    return () => {
      vivo = false;
    };
  }, [original, resultado]);

  useEffect(() => {
    const c = caja.current;
    if (!c) return;
    const o = new ResizeObserver(([e]) => setTamano({ w: e.contentRect.width, h: e.contentRect.height }));
    o.observe(c);
    return () => o.disconnect();
  }, []);

  const panel = useCallback(
    (lado: 0 | 1) => {
      // En lado a lado, cada mitad es un panel; en deslizador, uno solo.
      if (modoReal === "deslizador") return { x0: 0, w: tamano.w };
      const w = tamano.w / 2;
      return { x0: lado * w, w };
    },
    [modoReal, tamano.w],
  );

  const ajustar = useCallback((): Vista | null => {
    if (!ref || tamano.w === 0) return null;
    const { w } = panel(0);
    const margen = 24;
    // Ajusta a lo que cabe, también hacia arriba (hasta 8×): una imagen
    // pequeña se juzga mejor grande y con los píxeles a la vista.
    const escala = Math.min((w - margen * 2) / ref.width, (tamano.h - margen * 2) / ref.height, 8);
    return { escala, x: (w - ref.width * escala) / 2, y: (tamano.h - ref.height * escala) / 2 };
  }, [ref, tamano, panel]);

  // Al abrir otra imagen, o al cambiar el tamaño o el modo, se ajusta.
  const refTamano = ref ? `${ref.width}x${ref.height}` : "";
  useEffect(() => {
    setVista(ajustar());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refTamano, tamano.w, tamano.h, modoReal]);

  // Pintar.
  useEffect(() => {
    const c = lienzo.current;
    if (!c || !vista || !ref) return;
    const dpr = window.devicePixelRatio || 1;
    c.width = Math.round(tamano.w * dpr);
    c.height = Math.round(tamano.h * dpr);
    const g = c.getContext("2d")!;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, tamano.w, tamano.h);
    g.imageSmoothingEnabled = vista.escala < 1;
    g.imageSmoothingQuality = "high";

    const [a, b] = bitmaps;
    const damero = (x: number, y: number, w: number, h: number) => {
      const lado = 8;
      // Recortado al rectángulo de la imagen: el último cuadro de cada fila
      // se saldría y asomaría por el borde.
      g.save();
      g.beginPath();
      g.rect(x, y, w, h);
      g.clip();
      g.fillStyle = color("--damero-a");
      g.fillRect(x, y, w, h);
      g.fillStyle = color("--damero-b");
      for (let j = 0; j * lado < h; j++)
        for (let i = (j % 2); i * lado < w; i += 2) g.fillRect(x + i * lado, y + j * lado, lado, lado);
      g.restore();
    };
    const pintar = (bm: ImageBitmap | null, x0: number, w: number, desde: number, hasta: number) => {
      const ix = x0 + vista.x;
      const iw = ref.width * vista.escala;
      const ih = ref.height * vista.escala;
      g.save();
      g.beginPath();
      g.rect(Math.max(x0, desde), 0, Math.min(x0 + w, hasta) - Math.max(x0, desde), tamano.h);
      g.clip();
      // Si el resultado no tiene la proporción del original (recorte), se
      // encaja en su panel con su propia escala.
      if (bm && !mismaProporcion && bm !== a) {
        const e = Math.min((w - 48) / bm.width, (tamano.h - 48) / bm.height, 8) * (vista.escala / (ajustar()?.escala ?? vista.escala));
        const bx = x0 + (w - bm.width * e) / 2;
        const by = (tamano.h - bm.height * e) / 2;
        damero(bx, by, bm.width * e, bm.height * e);
        g.drawImage(bm, bx, by, bm.width * e, bm.height * e);
      } else {
        damero(ix, vista.y, iw, ih);
        if (bm) g.drawImage(bm, ix, vista.y, iw, ih);
      }
      g.restore();
    };

    if (modoReal === "deslizador") {
      const xc = corte * tamano.w;
      pintar(a, 0, tamano.w, 0, xc);
      pintar(b ?? a, 0, tamano.w, xc, tamano.w);
      // La línea del corte, blanca en los dos temas: va sobre la foto, no
      // sobre el cromo.
      g.fillStyle = "#ffffff";
      g.fillRect(xc - 1, 0, 2, tamano.h);
      g.fillStyle = color("--relleno");
      g.beginPath();
      g.arc(xc, tamano.h / 2, 12, 0, Math.PI * 2);
      g.fill();
      g.fillStyle = color("--sobre-acento");
      g.font = "12px system-ui";
      g.textAlign = "center";
      g.textBaseline = "middle";
      g.fillText("⇆", xc, tamano.h / 2 + 1);
    } else {
      const p0 = panel(0);
      const p1 = panel(1);
      pintar(a, p0.x0, p0.w, p0.x0, p0.x0 + p0.w);
      pintar(b ?? a, p1.x0, p1.w, p1.x0, p1.x0 + p1.w);
      g.fillStyle = color("--filete");
      g.fillRect(p1.x0 - 0.5, 0, 1, tamano.h);
    }
  }, [bitmaps, vista, corte, tamano, modoReal, ref, panel, mismaProporcion, ajustar]);

  const enCorte = (x: number) => modoReal === "deslizador" && Math.abs(x - corte * tamano.w) < 16;

  const posicion = (e: React.PointerEvent | React.WheelEvent) => {
    const r = lienzo.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const alPulsar = (e: React.PointerEvent) => {
    if (!vista) return;
    const p = posicion(e);
    (e.target as Element).setPointerCapture(e.pointerId);
    arrastre.current = enCorte(p.x) ? { tipo: "corte" } : { tipo: "mover", x: p.x, y: p.y, v: vista };
  };

  const alMover = (e: React.PointerEvent) => {
    const a = arrastre.current;
    const p = posicion(e);
    if (lienzo.current) lienzo.current.style.cursor = a?.tipo === "mover" ? "grabbing" : enCorte(p.x) ? "ew-resize" : "grab";
    if (!a) return;
    if (a.tipo === "corte") setCorte(Math.min(1, Math.max(0, p.x / tamano.w)));
    else setVista({ ...a.v, x: a.v.x + p.x - a.x, y: a.v.y + p.y - a.y });
  };

  const alSoltar = () => {
    arrastre.current = null;
  };

  const zoom = (factor: number, cx: number, cy: number) => {
    if (!vista) return;
    const escala = Math.min(64, Math.max(0.02, vista.escala * factor));
    const f = escala / vista.escala;
    const x0 = modoReal === "ladoALado" && cx > tamano.w / 2 ? tamano.w / 2 : 0;
    setVista({ escala, x: cx - x0 - (cx - x0 - vista.x) * f, y: cy - (cy - vista.y) * f });
  };

  // El gesto de zoom (rueda, o pellizco en el trackpad) es solo del lienzo.
  // Va con un oyente nativo **no pasivo**: el de React es pasivo, no puede
  // cancelar el evento, y entonces la ventana entera hacía su rebote elástico
  // a la vez que se acercaba la imagen (lo vio el cliente en la v0.3.1).
  //
  // El pellizco llega distinto según el motor: Chromium y WebView2 lo mandan
  // como rueda con Ctrl; WebKit (macOS y Linux), como `gesturechange` con la
  // escala acumulada desde que empezó.
  const zoomActual = useRef(zoom);
  zoomActual.current = zoom;
  const puntero = useRef({ x: 0, y: 0 });
  useEffect(() => {
    const c = lienzo.current;
    if (!c) return;
    const enLienzo = (e: { clientX: number; clientY: number }) => {
      const r = c.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    };
    const rueda = (e: WheelEvent) => {
      e.preventDefault();
      const p = enLienzo(e);
      puntero.current = p;
      const intensidad = e.ctrlKey ? 0.01 : 0.0015;
      zoomActual.current(Math.exp(-e.deltaY * intensidad), p.x, p.y);
    };
    let ultima = 1;
    const empieza = (e: Event) => {
      e.preventDefault();
      ultima = 1;
    };
    const cambia = (e: Event) => {
      e.preventDefault();
      const escala = (e as unknown as { scale: number }).scale;
      zoomActual.current(escala / ultima, puntero.current.x, puntero.current.y);
      ultima = escala;
    };
    const mueve = (e: PointerEvent) => (puntero.current = enLienzo(e));
    c.addEventListener("wheel", rueda, { passive: false });
    c.addEventListener("gesturestart", empieza, { passive: false } as AddEventListenerOptions);
    c.addEventListener("gesturechange", cambia, { passive: false } as AddEventListenerOptions);
    c.addEventListener("pointermove", mueve);
    return () => {
      c.removeEventListener("wheel", rueda);
      c.removeEventListener("gesturestart", empieza);
      c.removeEventListener("gesturechange", cambia);
      c.removeEventListener("pointermove", mueve);
    };
  }, []);

  const cienPorCien = () => {
    if (!vista || !ref) return;
    const { w } = panel(0);
    setVista({ escala: 1, x: (w - ref.width) / 2, y: (tamano.h - ref.height) / 2 });
  };

  return (
    <div className="comparador" ref={caja}>
      <canvas
        ref={lienzo}
        data-prueba="lienzo"
        style={{ width: tamano.w, height: tamano.h }}
        onPointerDown={alPulsar}
        onPointerMove={alMover}
        onPointerUp={alSoltar}
        onPointerCancel={alSoltar}
        onDoubleClick={() => setVista(ajustar())}
        aria-label={t("comparador.etiqueta")}
        role="img"
      />
      <span className="rotulo izquierda">{t("comparador.original")}</span>
      <span className="rotulo derecha">
        {t("comparador.resultado")}
        {ocupado && <span className="girando" aria-label={t("comparador.codificando")} />}
      </span>
      {!mismaProporcion && modo === "deslizador" && <span className="nota-comparador">{t("comparador.otraProporcion")}</span>}
      {/* Reducida, la vista promedia los píxeles y esconde los defectos de la
          compresión: con calidad 5 «apenas se veía diferencia» (v0.3.1). */}
      {vista && mismaProporcion && vista.escala < 0.995 && (
        <button className="nota-reducida" onClick={cienPorCien} data-prueba="nota-reducida">
          {t("comparador.reducida", { porcentaje: Math.round(vista.escala * 100) })}
        </button>
      )}
      <div className="zoom" role="group" aria-label={t("comparador.zoom")}>
        <button onClick={() => zoom(1 / 1.5, tamano.w / 2, tamano.h / 2)} aria-label={t("comparador.alejar")}>
          −
        </button>
        <button onClick={cienPorCien} data-prueba="zoom-100">
          {vista ? `${Math.round(vista.escala * 100)} %` : "—"}
        </button>
        <button onClick={() => zoom(1.5, tamano.w / 2, tamano.h / 2)} aria-label={t("comparador.acercar")}>
          +
        </button>
        <button onClick={() => setVista(ajustar())}>{t("comparador.ajustar")}</button>
      </div>
    </div>
  );
}
