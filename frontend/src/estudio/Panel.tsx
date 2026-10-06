// El panel de ajustes: formato, preset de partida, presets guardados y los
// controles en tres niveles plegables. Los controles se pintan a partir de
// CONTROLES (opciones.ts); aquí solo está cómo se ve cada tipo.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import * as puente from "../puente";
import {
  CONTROLES,
  NIVELES,
  type Contexto,
  type Control,
  type Nivel,
  type OpcionesWebp,
  type Preset,
} from "./opciones";

interface Props {
  opciones: OpcionesWebp;
  contexto: Contexto;
  presetsCwebp: Preset[];
  guardados: puente.PresetGuardado[];
  cambiar: (o: OpcionesWebp) => void;
  aplicarPreset: (p: Preset) => void;
  nivelSinPerdida: (n: number) => void;
  guardar: (nombre: string) => void;
  borrar: (nombre: string) => void;
}

const CLAVE_PLEGADO = "apolo.niveles";

function leerPlegado(): Record<Nivel, boolean> {
  try {
    const v = JSON.parse(localStorage.getItem(CLAVE_PLEGADO) ?? "null");
    if (v && typeof v === "object") return { basico: true, avanzado: false, experto: false, ...v };
  } catch {
    /* sin almacenamiento: los valores por defecto */
  }
  return { basico: true, avanzado: false, experto: false };
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

  const elegirPartida = (v: string) => {
    if (v.startsWith("cwebp:")) p.aplicarPreset(v.slice(6) as Preset);
    else if (v.startsWith("apolo:")) {
      const g = p.guardados.find((x) => x.nombre === v.slice(6));
      if (g) p.cambiar(g.webp);
    }
  };

  return (
    <aside className="panel" aria-label={t("panel.etiqueta")}>
      <div className="grupo">
        <label className="campo">
          <span>{t("panel.formato")}</span>
          <select value="webp" onChange={() => {}} data-prueba="formato">
            <option value="webp">WebP</option>
            {["avif", "jxl", "mozjpeg", "oxipng", "qoi"].map((f) => (
              <option key={f} value={f} disabled>
                {t(`formato.${f}`)} · {t("panel.pronto")}
              </option>
            ))}
          </select>
        </label>
        <label className="campo">
          <span>{t("panel.partida")}</span>
          <select value="" onChange={(e) => elegirPartida(e.target.value)} data-prueba="partida">
            <option value="" disabled>
              {p.opciones.preset ? t(`presetCwebp.${p.opciones.preset}`) : t("panel.elegir")}
            </option>
            <optgroup label={t("panel.presetsCwebp")}>
              {p.presetsCwebp.map((x) => (
                <option key={x} value={`cwebp:${x}`}>
                  {t(`presetCwebp.${x}`)}
                </option>
              ))}
            </optgroup>
            {p.guardados.length > 0 && (
              <optgroup label={t("panel.presetsGuardados")}>
                {p.guardados.map((g) => (
                  <option key={g.nombre} value={`apolo:${g.nombre}`}>
                    {g.nombre}
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

      {NIVELES.map((nivel) => (
        <details
          key={nivel}
          open={abiertos[nivel]}
          onToggle={(e) => {
            const abierto = (e.currentTarget as HTMLDetailsElement).open;
            setAbiertos((a) => (a[nivel] === abierto ? a : { ...a, [nivel]: abierto }));
          }}
          data-prueba={`nivel-${nivel}`}
        >
          <summary>{t(`nivel.${nivel}`)}</summary>
          <div className="controles">
            {CONTROLES.filter((c) => c.nivel === nivel).map((c) => (
              <Fila key={c.clave} control={c} {...p} />
            ))}
          </div>
        </details>
      ))}
    </aside>
  );
}

function Fila({ control: c, opciones: o, contexto, cambiar, nivelSinPerdida }: Props & { control: Control }) {
  const { t } = useTranslation();
  const activo = !c.cuando || c.cuando(o, contexto);
  const poner = (campo: keyof OpcionesWebp, v: unknown) => cambiar({ ...o, [campo]: v, preset: o.preset } as OpcionesWebp);
  const etiqueta = c.clave === "calidad" && o.sin_perdida ? t("opcion.esfuerzo") : t(`opcion.${c.clave}`);
  const ayuda = c.clave === "calidad" && o.sin_perdida ? t("opcion.esfuerzoAyuda") : t(`opcion.${c.clave}Ayuda`);
  const id = `control-${c.clave}`;

  let entrada: React.ReactNode;
  switch (c.tipo) {
    case "deslizador": {
      const v = o[c.campo] as number;
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
            onChange={(e) => poner(c.campo, Number(e.target.value))}
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
          value={o[c.campo] as number}
          disabled={!activo}
          onChange={(e) => poner(c.campo, Math.max(c.min, Number(e.target.value) || 0))}
        />
      );
      break;
    case "interruptor":
      // El interruptor va en la misma línea que su etiqueta.
      entrada = null;
      break;
    case "lista": {
      const actual = o[c.campo];
      entrada = (
        <select
          id={id}
          value={String(actual)}
          disabled={!activo}
          onChange={(e) => poner(c.campo, typeof actual === "number" ? Number(e.target.value) : e.target.value)}
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
      entrada = <Especial c={c} o={o} activo={activo} poner={poner} nivelSinPerdida={nivelSinPerdida} contexto={contexto} id={id} />;
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
            checked={o[c.campo] as boolean}
            disabled={!activo}
            onChange={(e) => poner(c.campo, e.target.checked)}
          />
        )}
        <label htmlFor={id}>{etiqueta}</label>
        {c.cwebp ? <code className="cwebp">{c.cwebp}</code> : <span className="marca-apolo">{t("panel.soloApolo")}</span>}
      </div>
      {entrada}
      <p className="ayuda">{activo ? ayuda : t(c.porQue ?? "")}</p>
    </div>
  );
}

function Especial({
  c,
  o,
  activo,
  poner,
  nivelSinPerdida,
  contexto,
  id,
}: {
  c: Control & { tipo: "especial" };
  o: OpcionesWebp;
  activo: boolean;
  poner: (campo: keyof OpcionesWebp, v: unknown) => void;
  nivelSinPerdida: (n: number) => void;
  contexto: Contexto;
  id: string;
}) {
  const { t } = useTranslation();
  const num = (v: string) => Math.max(0, Math.round(Number(v) || 0));
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
    case "redimension": {
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
            aria-label={t("opcion.redimension")}
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
    case "recorte": {
      const r = o.recorte;
      return (
        <div className="pareja cuadruple">
          <input
            id={id}
            type="checkbox"
            role="switch"
            checked={r !== null}
            onChange={(e) => poner("recorte", e.target.checked ? { x: 0, y: 0, ancho: contexto.ancho, alto: contexto.alto } : null)}
            aria-label={t("opcion.recorte")}
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
      return (
        <input id={id} type="checkbox" role="switch" checked={o.hilos > 0} onChange={(e) => poner("hilos", e.target.checked ? 1 : 0)} />
      );
  }
}
