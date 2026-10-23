// La barra de encima de la imagen (ADR 0032): Recortar y Redimensionar, a la
// vista y comunes a los dos lados, porque son de la imagen y no del formato.
// Mientras se recorta, la barra es la del recorte: Listo, Quitar, la
// proporción y las medidas del marco (el camino con el teclado; el ratón va
// sobre la imagen).

import { useTranslation } from "react-i18next";
import { Icono } from "../componentes";
import type { Recorte, Tamano } from "./opciones";
import { dentro, PROPORCIONES, type Proporcion } from "./recorte";
import { SelectorTamano } from "./VentanaTamano";

export function BarraImagen({
  recorte,
  editando,
  proporcion,
  ancho,
  alto,
  empezar,
  acabar,
  quitar,
  elegirProporcion,
  ponerRecorte,
  redimension,
  cambiarRedimension,
}: {
  recorte: Recorte | null;
  editando: boolean;
  proporcion: Proporcion;
  /** Lo que mide la imagen ya enderezada: el marco va en sus píxeles. */
  ancho: number;
  alto: number;
  empezar: () => void;
  acabar: () => void;
  quitar: () => void;
  elegirProporcion: (p: Proporcion) => void;
  ponerRecorte: (r: Recorte) => void;
  redimension: Tamano | null;
  cambiarRedimension: (r: Tamano | null) => void;
}) {
  const { t } = useTranslation();

  if (editando && recorte) {
    const campo = (k: keyof Recorte) => (
      <label key={k} className="campo-recorte">
        <span>{t(`barraImagen.${k}`)}</span>
        <input
          type="number"
          min={0}
          value={recorte[k]}
          onChange={(e) => {
            const r = dentro({ ...recorte, [k]: Math.max(0, Math.round(Number(e.target.value) || 0)) }, ancho, alto);
            if (r) ponerRecorte(r);
          }}
          data-prueba={`recorte-${k}`}
        />
      </label>
    );
    return (
      <div className="barra-imagen recortando" role="toolbar" aria-label={t("barraImagen.etiqueta")} data-prueba="barra-imagen">
        <button className="principal" onClick={acabar} data-prueba="recorte-listo">
          <Icono nombre="recortar" />
          {t("barraImagen.listo")}
        </button>
        <button onClick={quitar} data-prueba="quitar-recorte">
          {t("barraImagen.quitarRecorte")}
        </button>
        <select
          value={proporcion}
          onChange={(e) => elegirProporcion(e.target.value as Proporcion)}
          aria-label={t("barraImagen.proporcion")}
          title={t("barraImagen.proporcion")}
          data-prueba="proporcion"
        >
          {PROPORCIONES.map((p) => (
            <option key={p} value={p}>
              {p === "libre" || p === "original" ? t(`barraImagen.proporciones.${p}`) : p}
            </option>
          ))}
        </select>
        <span className="campos-recorte">{(["ancho", "alto", "x", "y"] as const).map(campo)}</span>
      </div>
    );
  }

  // Lo que mide la imagen antes de redimensionar: la recortada, si lo está.
  const medidas: [number, number] = recorte
    ? [Math.min(recorte.ancho, ancho - recorte.x), Math.min(recorte.alto, alto - recorte.y)]
    : [ancho, alto];
  return (
    <div className="barra-imagen" role="toolbar" aria-label={t("barraImagen.etiqueta")} data-prueba="barra-imagen">
      <span className="grupo-barra">
        <button onClick={empezar} data-prueba="recortar">
          <Icono nombre="recortar" />
          <span className="etiqueta">
            {recorte ? t("barraImagen.recortada", { ancho: medidas[0], alto: medidas[1] }) : t("barraImagen.recortar")}
          </span>
        </button>
        {recorte && (
          <button className="solo-icono" onClick={quitar} aria-label={t("barraImagen.quitarRecorte")} title={t("barraImagen.quitarRecorte")} data-prueba="quitar-recorte">
            <Icono nombre="quitar" />
          </button>
        )}
      </span>
      <SelectorTamano valor={redimension} cambiar={cambiarRedimension} medidas={medidas} vacio={t("tamano.boton")} prueba="redimensionar" />
    </div>
  );
}
