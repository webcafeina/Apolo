// El panel de ajustes del lado que se edita: el lado, el formato, el preset de
// partida, los presets guardados, el proceso (para todos los formatos) y los
// controles del formato en tres niveles plegables. Los controles se pintan a
// partir de los esquemas de opciones.ts; aquí solo está cómo se ve cada tipo.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Icono } from "../componentes";
import * as puente from "../puente";
import {
  CONTROLES_PROCESO,
  FORMATOS,
  NIVELES,
  controlesDe,
  type Ajuste,
  type Contexto,
  type Control,
  type FormatoSalida,
  type Preset,
  type Proceso,
} from "./opciones";

export type Lado = 0 | 1;

interface Props {
  ajuste: Ajuste;
  contexto: Contexto;
  presetsCwebp: Preset[];
  guardados: puente.PresetGuardado[];
  cambiar: (a: Ajuste) => void;
  aplicarPreset: (p: Preset) => void;
  nivelSinPerdida: (n: number) => void;
  guardar: (nombre: string) => void;
  /** Qué lado se edita, y si el izquierdo es el original (sin formato). */
  lado: Lado;
  elegirLado: (l: Lado) => void;
  izquierdaOriginal: boolean;
  formatoIzquierda: FormatoSalida | null;
  formatoDerecha: FormatoSalida;
  usarFormatoIzquierda: (si: boolean) => void;
}

const CLAVE_PLEGADO = "apolo.niveles";
type Plegado = Record<string, boolean>;

function leerPlegado(): Plegado {
  const base: Plegado = { proceso: true, basico: true, avanzado: false, experto: false };
  try {
    const v = JSON.parse(localStorage.getItem(CLAVE_PLEGADO) ?? "null");
    if (v && typeof v === "object") return { ...base, ...v };
  } catch {
    /* sin almacenamiento: los valores por defecto */
  }
  return base;
}

/** Nombre corto de un formato de salida, para la interfaz. */
export function nombreFormato(f: FormatoSalida): string {
  return { webp: "WebP", jpeg: "JPEG", png: "PNG", qoi: "QOI", avif: "AVIF", jxl: "JPEG XL" }[f];
}

export function Panel(p: Props) {
  const { t } = useTranslation();
  const [abiertos, setAbiertos] = useState(leerPlegado);
  const [nombre, setNombre] = useState("");

  useEffect(() => {
    try {
      localStorage.setItem(CLAVE_PLEGADO, JSON.stringify(abiertos));
    } catch {
      /* nada */
    }
  }, [abiertos]);

  const a = p.ajuste;
  const elegirPartida = (v: string) => {
    if (v.startsWith("cwebp:")) p.aplicarPreset(v.slice(6) as Preset);
    else if (v.startsWith("apolo:")) {
      const g = p.guardados.find((x) => x.nombre === v.slice(6));
      if (g) {
        const { nombre: _, ...ajuste } = g;
        p.cambiar(ajuste);
      }
    }
  };

  const plegable = (clave: string, titulo: string, contenido: React.ReactNode) => (
    <details
      key={clave}
      open={abiertos[clave]}
      onToggle={(e) => {
        const abierto = (e.currentTarget as HTMLDetailsElement).open;
        setAbiertos((x) => (x[clave] === abierto ? x : { ...x, [clave]: abierto }));
      }}
      data-prueba={`nivel-${clave}`}
    >
      <summary>{titulo}</summary>
      <div className="controles">{contenido}</div>
    </details>
  );

  const selectorLado = (
    <div className="segmentado lados" role="radiogroup" aria-label={t("panel.lado.etiqueta")} data-prueba="lados">
      {([0, 1] as const).map((l) => (
        <button key={l} role="radio" aria-checked={p.lado === l} onClick={() => p.elegirLado(l)}>
          {t(l === 0 ? "panel.lado.izquierda" : "panel.lado.derecha")}
          <span className="apagado">
            {" · "}
            {l === 0
              ? p.formatoIzquierda
                ? nombreFormato(p.formatoIzquierda)
                : t("comparador.original")
              : nombreFormato(p.formatoDerecha)}
          </span>
        </button>
      ))}
    </div>
  );

  if (p.lado === 0 && p.izquierdaOriginal) {
    return (
      <aside className="panel" aria-label={t("panel.etiqueta")}>
        <div className="panel-cabeza">{selectorLado}</div>
        <div className="panel-original">
          <p>{t("panel.izquierdaOriginal")}</p>
          <button className="principal" onClick={() => p.usarFormatoIzquierda(true)} data-prueba="comparar-formato">
            {t("panel.compararFormato")}
          </button>
        </div>
      </aside>
    );
  }

  const controles = controlesDe(a.formato);

  return (
    <aside className="panel" aria-label={t("panel.etiqueta")}>
      <div className="panel-cabeza">
        {selectorLado}
        {p.lado === 0 && (
          <button className="volver-original" onClick={() => p.usarFormatoIzquierda(false)} data-prueba="volver-original">
            <Icono nombre="deslizador" lado={15} />
            {t("panel.volverOriginal")}
          </button>
        )}
        <label className="campo">
          <span>{t("panel.formato")}</span>
          <select
            value={a.formato}
            onChange={(e) => p.cambiar({ ...a, formato: e.target.value as FormatoSalida })}
            data-prueba="formato"
          >
            {FORMATOS.map((f) => (
              <option key={f} value={f}>
                {t(`formatoSalida.${f}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="campo">
          <span>{t("panel.partida")}</span>
          <select value="" onChange={(e) => elegirPartida(e.target.value)} data-prueba="partida">
            <option value="" disabled>
              {a.formato === "webp" && a.webp.preset ? t(`presetCwebp.${a.webp.preset}`) : t("panel.elegir")}
            </option>
            {a.formato === "webp" && (
              <optgroup label={t("panel.presetsCwebp")}>
                {p.presetsCwebp.map((x) => (
                  <option key={x} value={`cwebp:${x}`}>
                    {t(`presetCwebp.${x}`)}
                  </option>
                ))}
              </optgroup>
            )}
            {p.guardados.length > 0 && (
              <optgroup label={t("panel.presetsGuardados")}>
                {p.guardados.map((g) => (
                  <option key={g.nombre} value={`apolo:${g.nombre}`}>
                    {g.nombre} · {nombreFormato(g.formato)}
                  </option>
                ))}
              </optgroup>
            )}
          </select>
        </label>
        <form
          className="guardar"
          onSubmit={(e) => {
            e.preventDefault();
            if (nombre.trim()) {
              p.guardar(nombre.trim());
              setNombre("");
            }
          }}
        >
          <input
            value={nombre}
            onChange={(e) => setNombre(e.target.value)}
            placeholder={t("panel.nombrePreset")}
            aria-label={t("panel.nombrePreset")}
            data-prueba="nombre-preset"
          />
          <button type="submit" disabled={!nombre.trim()}>
            {t("panel.guardar")}
          </button>
        </form>
      </div>

      {plegable(
        "proceso",
        t("nivel.proceso"),
        CONTROLES_PROCESO.map((c) => <Fila key={c.clave} control={c} {...p} />),
      )}
      {controles.length === 0 ? (
        <p className="ayuda panel-sin-opciones">{t("panel.sinOpciones", { formato: nombreFormato(a.formato) })}</p>
      ) : (
        NIVELES.map((nivel) =>
          plegable(
            nivel,
            t(`nivel.${nivel}`),
            controles.filter((c) => c.nivel === nivel).map((c) => <Fila key={c.clave} control={c} {...p} />),
          ),
        )
      )}
    </aside>
  );
}

function Fila({ control: c, ajuste: a, contexto, cambiar, nivelSinPerdida }: Props & { control: Control }) {
  const { t } = useTranslation();
  const activo = !c.cuando || c.cuando(a, contexto);
  const obj = a[c.de] as unknown as Record<string, unknown>;
  const poner = (campo: string, v: unknown) => {
    const quitados = Object.fromEntries((c.limpiar ?? []).map((k) => [k, null]));
    cambiar({ ...a, [c.de]: { ...obj, ...quitados, [campo]: v } } as Ajuste);
  };
  const sinPerdidaWebp = c.de === "webp" && c.clave === "calidad" && a.webp.sin_perdida;
  const etiqueta = sinPerdidaWebp ? t("opcion.esfuerzo") : t(`opcion.${c.clave}`);
  const ayuda = sinPerdidaWebp ? t("opcion.esfuerzoAyuda") : t(`opcion.${c.clave}Ayuda`);
  const id = `control-${c.clave}`;

  let entrada: React.ReactNode;
  switch (c.tipo) {
    case "deslizador": {
      const v = (obj[c.campo] as number | null) ?? c.defecto ?? 0;
      entrada = (
        <div className="deslizador">
          <input
            id={id}
            type="range"
            min={c.min}
            max={c.max}
            step={c.paso}
            value={v}
            disabled={!activo}
            // Un campo que puede faltar (null) vuelve a faltar al llevarlo a 0.
            onChange={(e) => {
              const n = Number(e.target.value);
              const vuelveANulo = c.defecto === undefined && n === 0 && (obj[c.campo] === null || c.campo === "suavizado");
              poner(c.campo, vuelveANulo ? null : n);
            }}
          />
          <output htmlFor={id}>{v}</output>
        </div>
      );
      break;
    }
    case "numero":
      entrada = (
        <input
          id={id}
          type="number"
          min={c.min}
          max={c.max}
          value={(obj[c.campo] as number | null) ?? ""}
          disabled={!activo}
          onChange={(e) =>
            poner(c.campo, e.target.value === "" && obj[c.campo] !== 0 ? null : Math.max(c.min, Number(e.target.value) || 0))
          }
        />
      );
      break;
    case "interruptor":
      // El interruptor va en la misma línea que su etiqueta.
      entrada = null;
      break;
    case "lista": {
      const actual = obj[c.campo];
      const valor = actual === null && c.nulo ? c.nulo : String(actual);
      entrada = (
        <select
          id={id}
          value={valor}
          disabled={!activo}
          onChange={(e) => {
            const v = e.target.value;
            if (c.nulo && v === c.nulo) poner(c.campo, null);
            else poner(c.campo, c.numerica || typeof actual === "number" ? Number(v) : v);
          }}
        >
          {c.valores.map((v) => (
            <option key={v} value={v}>
              {t(`valor.${c.clave}.${v}`)}
            </option>
          ))}
        </select>
      );
      break;
    }
    case "especial":
      entrada = (
        <Especial c={c} a={a} activo={activo} poner={poner} cambiar={cambiar} nivelSinPerdida={nivelSinPerdida} contexto={contexto} id={id} />
      );
      break;
  }

  return (
    <div className={`fila-control${activo ? "" : " apagado"}`} data-prueba={`control-${c.clave}`}>
      <div className="cabecera-control">
        {c.tipo === "interruptor" && (
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={obj[c.campo] as boolean}
            disabled={!activo}
            onChange={(e) => poner(c.campo, e.target.checked)}
          />
        )}
        <label htmlFor={id}>{etiqueta}</label>
        {c.marca ? <code className="cwebp">{c.marca}</code> : <span className="marca-apolo">{t("panel.soloApolo")}</span>}
      </div>
      {entrada}
      <p className="ayuda">{activo ? ayuda : t(c.porQue ?? "")}</p>
    </div>
  );
}

function Especial({
  c,
  a,
  activo,
  poner,
  cambiar,
  nivelSinPerdida,
  contexto,
  id,
}: {
  c: Control & { tipo: "especial" };
  a: Ajuste;
  activo: boolean;
  poner: (campo: string, v: unknown) => void;
  cambiar: (a: Ajuste) => void;
  nivelSinPerdida: (n: number) => void;
  contexto: Contexto;
  id: string;
}) {
  const { t } = useTranslation();
  const num = (v: string) => Math.max(0, Math.round(Number(v) || 0));
  const o = a.webp;
  const proceso = (cambio: Partial<Proceso>) => cambiar({ ...a, proceso: { ...a.proceso, ...cambio } });
  switch (c.especial) {
    case "nivelSinPerdida":
      return (
        <select id={id} value="" disabled={!activo} onChange={(e) => nivelSinPerdida(Number(e.target.value))}>
          <option value="" disabled>
            {t("panel.elegirNivel")}
          </option>
          {Array.from({ length: 10 }, (_, n) => (
            <option key={n} value={n}>
              {n} · {t(n < 3 ? "valor.nivel.rapido" : n > 6 ? "valor.nivel.lento" : "valor.nivel.medio")}
            </option>
          ))}
        </select>
      );
    case "redimensionCwebp": {
      const r = o.redimension;
      return (
        <div className="pareja">
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={r !== null}
            onChange={(e) =>
              poner("redimension", e.target.checked ? { ancho: Math.round(contexto.ancho / 2), alto: 0 } : null)
            }
            aria-label={t("opcion.redimensionCwebp")}
          />
          {r && (
            <>
              <input type="number" min={0} value={r.ancho} aria-label={t("panel.ancho")} onChange={(e) => poner("redimension", { ...r, ancho: num(e.target.value) })} />
              <span>×</span>
              <input type="number" min={0} value={r.alto} aria-label={t("panel.alto")} onChange={(e) => poner("redimension", { ...r, alto: num(e.target.value) })} />
            </>
          )}
        </div>
      );
    }
    case "recorteCwebp": {
      const r = o.recorte;
      return (
        <div className="pareja cuadruple">
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={r !== null}
            onChange={(e) => poner("recorte", e.target.checked ? { x: 0, y: 0, ancho: contexto.ancho, alto: contexto.alto } : null)}
            aria-label={t("opcion.recorteCwebp")}
          />
          {r &&
            (["x", "y", "ancho", "alto"] as const).map((k) => (
              <input key={k} type="number" min={0} value={r[k]} aria-label={t(`panel.${k}`)} title={t(`panel.${k}`)} onChange={(e) => poner("recorte", { ...r, [k]: num(e.target.value) })} />
            ))}
        </div>
      );
    }
    case "rangoCalidad":
      return (
        <div className="pareja">
          <input id={id} type="number" min={0} max={100} value={o.calidad_minima} disabled={!activo} aria-label={t("panel.minimo")} onChange={(e) => poner("calidad_minima", Math.min(100, num(e.target.value)))} />
          <span>–</span>
          <input type="number" min={0} max={100} value={o.calidad_maxima} disabled={!activo} aria-label={t("panel.maximo")} onChange={(e) => poner("calidad_maxima", Math.min(100, num(e.target.value)))} />
        </div>
      );
    case "mezclarAlfa": {
      const m = o.mezclar_alfa;
      const hex = `#${(m ?? 0xffffff).toString(16).padStart(6, "0")}`;
      return (
        <div className="pareja">
          <input id={id} type="checkbox" role="switch" checked={m !== null} disabled={!activo} onChange={(e) => poner("mezclar_alfa", e.target.checked ? 0xffffff : null)} aria-label={t("opcion.mezclarAlfa")} />
          {m !== null && <input type="color" value={hex} disabled={!activo} aria-label={t("panel.colorFondo")} onChange={(e) => poner("mezclar_alfa", parseInt(e.target.value.slice(1), 16))} />}
        </div>
      );
    }
    case "metadatos":
      return (
        <div className="pareja" id={id}>
          {(["exif", "icc", "xmp"] as const).map((k) => (
            <label key={k} className="casilla">
              <input type="checkbox" checked={o.metadatos[k]} onChange={(e) => poner("metadatos", { ...o.metadatos, [k]: e.target.checked })} />
              {t(`panel.${k}`)}
            </label>
          ))}
        </div>
      );
    case "hilos":
      return <input id={id} type="checkbox" role="switch" checked={o.hilos > 0} onChange={(e) => poner("hilos", e.target.checked ? 1 : 0)} />;
    case "calidadJpeg": {
      const q = a.jpeg.calidad[0] ?? 75;
      return (
        <div className="deslizador">
          <input
            id={id}
            type="range"
            min={0}
            max={100}
            step={1}
            value={q}
            onChange={(e) => cambiar({ ...a, jpeg: { ...a.jpeg, calidad: [Number(e.target.value)] } })}
          />
          <output htmlFor={id}>{q}</output>
        </div>
      );
    }
    case "procesoRedimension": {
      const r = a.proceso.redimension;
      return (
        <div className="proceso-control">
          <div className="pareja">
            <input
              id={id}
              type="checkbox"
              role="switch"
              checked={r !== null}
              onChange={(e) =>
                proceso({
                  redimension: e.target.checked
                    ? { ancho: Math.max(1, Math.round(contexto.ancho / 2)), alto: null, filtro: "lanczos3", lineal: true }
                    : null,
                })
              }
              aria-label={t("opcion.procesoRedimension")}
            />
            {r && (
              <>
                <input
                  type="number"
                  min={1}
                  value={r.ancho ?? ""}
                  placeholder={t("panel.auto")}
                  aria-label={t("panel.ancho")}
                  onChange={(e) => proceso({ redimension: { ...r, ancho: e.target.value === "" ? null : Math.max(1, num(e.target.value)) } })}
                />
                <span>×</span>
                <input
                  type="number"
                  min={1}
                  value={r.alto ?? ""}
                  placeholder={t("panel.auto")}
                  aria-label={t("panel.alto")}
                  onChange={(e) => proceso({ redimension: { ...r, alto: e.target.value === "" ? null : Math.max(1, num(e.target.value)) } })}
                />
              </>
            )}
          </div>
          {r && (
            <div className="pareja">
              <select value={r.filtro} aria-label={t("panel.filtro")} onChange={(e) => proceso({ redimension: { ...r, filtro: e.target.value as never } })}>
                {(["lanczos3", "mitchell", "catmull_rom", "bilineal", "vecino"] as const).map((f) => (
                  <option key={f} value={f}>
                    {t(`valor.filtro.${f}`)}
                  </option>
                ))}
              </select>
              <label className="casilla">
                <input type="checkbox" checked={r.lineal} onChange={(e) => proceso({ redimension: { ...r, lineal: e.target.checked } })} />
                {t("panel.lineal")}
              </label>
            </div>
          )}
        </div>
      );
    }
    case "procesoRecorte": {
      const r = a.proceso.recorte;
      return (
        <div className="pareja cuadruple">
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={r !== null}
            onChange={(e) => proceso({ recorte: e.target.checked ? { x: 0, y: 0, ancho: contexto.ancho, alto: contexto.alto } : null })}
            aria-label={t("opcion.procesoRecorte")}
          />
          {r &&
            (["x", "y", "ancho", "alto"] as const).map((k) => (
              <input key={k} type="number" min={0} value={r[k]} aria-label={t(`panel.${k}`)} title={t(`panel.${k}`)} onChange={(e) => proceso({ recorte: { ...r, [k]: num(e.target.value) } })} />
            ))}
        </div>
      );
    }
    case "procesoPaleta": {
      const pal = a.proceso.paleta;
      return (
        <div className="proceso-control">
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={pal !== null}
            onChange={(e) => proceso({ paleta: e.target.checked ? { colores: 256, tramado: 1 } : null })}
            aria-label={t("opcion.procesoPaleta")}
          />
          {pal && (
            <>
              <label className="sub-control">
                <span>{t("panel.colores")}</span>
                <div className="deslizador">
                  <input type="range" min={2} max={256} step={1} value={pal.colores} onChange={(e) => proceso({ paleta: { ...pal, colores: Number(e.target.value) } })} />
                  <output>{pal.colores}</output>
                </div>
              </label>
              <label className="sub-control">
                <span>{t("panel.tramado")}</span>
                <div className="deslizador">
                  <input
                    type="range"
                    min={0}
                    max={100}
                    step={1}
                    value={Math.round(pal.tramado * 100)}
                    onChange={(e) => proceso({ paleta: { ...pal, tramado: Number(e.target.value) / 100 } })}
                  />
                  <output>{Math.round(pal.tramado * 100)} %</output>
                </div>
              </label>
            </>
          )}
        </div>
      );
    }
  }
}
