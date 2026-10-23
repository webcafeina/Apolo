// Redimensionar (ADR 0032): un botón que dice el tamaño, y su ventanita con
// el lado mayor, el porcentaje o el ancho y el alto. Lo usan la barra de
// encima de la imagen del Estudio y Lotes, para todas las salidas.

import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Emergente } from "../Emergente";
import { Icono } from "../componentes";
import { lengua } from "../i18n";
import type { Filtro, Tamano } from "./opciones";
import { ATAJOS_LADO, ATAJOS_PORCENTAJE, FILTROS, MODOS, medidasTras, numero, nueva, textoTamano, vacia } from "./tamano";

export function SelectorTamano({
  valor,
  cambiar,
  medidas,
  vacio,
  prueba,
}: {
  valor: Tamano | null;
  cambiar: (r: Tamano | null) => void;
  /** Lo que mide la imagen antes de redimensionar (ya recortada), para
   *  decir cómo quedará. En Lotes no se sabe. */
  medidas?: [number, number];
  /** El botón sin tamaño: «Redimensionar» en el Estudio, «El de cada preset» en Lotes. */
  vacio: string;
  prueba?: string;
}) {
  const { t } = useTranslation();
  const [clave, datos] = valor ? textoTamano(valor, lengua()) : ["", {}];
  return (
    <Emergente
      clase="selector-tamano"
      titulo={t("tamano.titulo")}
      prueba={prueba}
      boton={
        <>
          <Icono nombre="redimensionar" />
          <span className="etiqueta">{valor ? t(clave, datos) : vacio}</span>
          <Icono nombre="desplegar" lado={12} />
        </>
      }
    >
      {(cerrar) => <Ventana valor={valor} cambiar={cambiar} medidas={medidas} cerrar={cerrar} />}
    </Emergente>
  );
}

function Ventana({
  valor,
  cambiar,
  medidas,
  cerrar,
}: {
  valor: Tamano | null;
  cambiar: (r: Tamano | null) => void;
  medidas?: [number, number];
  cerrar: () => void;
}) {
  const { t } = useTranslation();
  // Un borrador: el modo elegido sin su número todavía no llega al núcleo,
  // que no sabría qué hacer con él.
  const [r, setR] = useState<Tamano>(valor ?? nueva());
  useEffect(() => {
    if (valor) setR(valor);
  }, [valor]);
  const poner = (cambio: Partial<Tamano>) => {
    const n = { ...r, ...cambio };
    setR(n);
    cambiar(vacia(n) ? null : n);
  };
  const bloqueado = !(r.ancho && r.alto);

  let resumen: string | null = null;
  if (medidas && !vacia(r)) {
    const [w, h] = medidas;
    const [nw, nh] = medidasTras(r, w, h);
    const [aw, ah] = medidasTras({ ...r, solo_reducir: false }, w, h);
    resumen =
      r.solo_reducir && (aw > w || ah > h)
        ? t("tamano.noAgranda", { ancho: w, alto: h })
        : t("tamano.quedara", { ancho: nw, alto: nh });
  }

  return (
    <div className="ventana-tamano" data-prueba="ventana-tamano">
      <div className="segmentado" role="radiogroup" aria-label={t("tamano.modo")}>
        {MODOS.map((m) => (
          <button key={m} role="radio" aria-checked={r.modo === m} onClick={() => poner({ modo: m })} data-prueba={`modo-${m}`}>
            {t(`tamano.modos.${m}`)}
          </button>
        ))}
      </div>

      {r.modo === "lado_mayor" && (
        <>
          <div className="atajos" role="group" aria-label={t("tamano.atajos")}>
            {ATAJOS_LADO.map((n) => (
              <button key={n} aria-pressed={r.lado_mayor === n} onClick={() => poner({ lado_mayor: n })}>
                {n}
              </button>
            ))}
          </div>
          <Numero etiqueta={t("tamano.ladoMayor")} sufijo="px" valor={r.lado_mayor} poner={(n) => poner({ lado_mayor: n })} prueba="lado-mayor" />
        </>
      )}
      {r.modo === "porcentaje" && (
        <>
          <div className="atajos" role="group" aria-label={t("tamano.atajos")}>
            {ATAJOS_PORCENTAJE.map((n) => (
              <button key={n} aria-pressed={r.porcentaje === n} onClick={() => poner({ porcentaje: n })}>
                {numero(n, lengua())} %
              </button>
            ))}
          </div>
          <Numero
            etiqueta={t("tamano.porcentaje")}
            sufijo="%"
            decimales
            valor={r.porcentaje}
            poner={(n) => poner({ porcentaje: n })}
            prueba="porcentaje"
          />
        </>
      )}
      {r.modo === "medidas" && (
        <div className="medidas-tamano">
          <Numero
            etiqueta={t("tamano.ancho")}
            sufijo="px"
            valor={r.ancho}
            // Con el candado, el otro lado va en proporción.
            poner={(n) => poner(bloqueado ? { ancho: n, alto: null } : { ancho: n })}
            prueba="ancho"
          />
          <button
            className="candado"
            aria-pressed={bloqueado}
            aria-label={t("tamano.candado")}
            title={t("tamano.candado")}
            onClick={() => {
              if (!bloqueado) poner({ alto: null });
              else if (r.ancho || r.alto) {
                const [w, h] = medidas ? medidasTras(r, medidas[0], medidas[1]) : [r.ancho, r.alto];
                poner({ ancho: w, alto: h });
              }
            }}
            data-prueba="candado"
          >
            <Icono nombre={bloqueado ? "candado" : "candadoAbierto"} />
          </button>
          <Numero
            etiqueta={t("tamano.alto")}
            sufijo="px"
            valor={r.alto}
            poner={(n) => poner(bloqueado ? { alto: n, ancho: null } : { alto: n })}
            prueba="alto"
          />
        </div>
      )}

      {resumen && (
        <p className="apagado resumen-tamano" role="status" data-prueba="resumen-tamano">
          {resumen}
        </p>
      )}

      <label className="casilla">
        <input type="checkbox" checked={!r.solo_reducir} onChange={(e) => poner({ solo_reducir: !e.target.checked })} data-prueba="agrandar" />
        {t("tamano.permitirAgrandar")}
      </label>

      <details className="mas-tamano">
        <summary>{t("tamano.mas")}</summary>
        <div className="pareja">
          <select value={r.filtro} aria-label={t("panel.filtro")} onChange={(e) => poner({ filtro: e.target.value as Filtro })}>
            {FILTROS.map((f) => (
              <option key={f} value={f}>
                {t(`valor.filtro.${f}`)}
              </option>
            ))}
          </select>
          <label className="casilla">
            <input type="checkbox" checked={r.lineal} onChange={(e) => poner({ lineal: e.target.checked })} />
            {t("panel.lineal")}
          </label>
        </div>
      </details>

      <div className="pie-tamano">
        <button
          onClick={() => {
            cambiar(null);
            cerrar();
          }}
          disabled={!valor}
          data-prueba="quitar-tamano"
        >
          {t("tamano.quitar")}
        </button>
        <button className="principal" onClick={cerrar} data-prueba="listo-tamano">
          {t("tamano.listo")}
        </button>
      </div>
    </div>
  );
}

/** Un número que se aplica al pulsar Intro, al salir del campo o tras un
 *  momento sin escribir: no se codifica a cada tecla. */
function Numero({
  etiqueta,
  sufijo,
  valor,
  poner,
  decimales = false,
  prueba,
}: {
  etiqueta: string;
  sufijo: string;
  valor: number | null;
  poner: (n: number | null) => void;
  decimales?: boolean;
  prueba: string;
}) {
  const [texto, setTexto] = useState(valor === null ? "" : String(valor));
  const reloj = useRef<number | undefined>(undefined);
  const campo = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (document.activeElement !== campo.current) setTexto(valor === null ? "" : String(valor));
  }, [valor]);
  useEffect(() => () => window.clearTimeout(reloj.current), []);
  const aplicar = (v: string) => {
    window.clearTimeout(reloj.current);
    const n = Number(v.replace(",", "."));
    if (v.trim() === "") poner(null);
    else if (Number.isFinite(n) && n > 0) poner(decimales ? Math.round(n * 10) / 10 : Math.round(n));
  };
  return (
    <label className="numero-tamano">
      <span>{etiqueta}</span>
      <span className="con-sufijo">
        <input
          ref={campo}
          type="number"
          min={1}
          step={decimales ? 0.1 : 1}
          inputMode={decimales ? "decimal" : "numeric"}
          value={texto}
          onChange={(e) => {
            setTexto(e.target.value);
            window.clearTimeout(reloj.current);
            const v = e.target.value;
            reloj.current = window.setTimeout(() => aplicar(v), 400);
          }}
          onBlur={(e) => aplicar(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") aplicar((e.target as HTMLInputElement).value);
          }}
          data-prueba={prueba}
        />
        <span className="sufijo">{sufijo}</span>
      </span>
    </label>
  );
}
