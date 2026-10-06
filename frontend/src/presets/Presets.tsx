// La sección Presets: los presets guardados, lo que hace cada uno y su orden.
//
// Antes vivían escondidos en Ajustes; el cliente pidió en la prueba guiada que
// tuvieran su sitio en la barra lateral, como Estudio y Lotes (v0.3.2).
// Desde aquí se aplican al Estudio, se renombran y se borran. Se crean en el
// Estudio, con «Guardar» en el panel de ajustes.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cabecera, Icono } from "../componentes";
import { EVENTO_APLICAR, EVENTO_PRESETS } from "../estudio/Estudio";
import type { OpcionesWebp } from "../estudio/opciones";
import * as puente from "../puente";

export function Presets({ carpeta, irAlEstudio }: { carpeta: string | null; irAlEstudio: () => void }) {
  const { t } = useTranslation();
  const [lista, setLista] = useState<puente.PresetGuardado[] | null>(null);
  const [fallo, setFallo] = useState<string | null>(null);

  const recargar = () => puente.presets().then(setLista, (e: puente.Fallo) => setFallo(e.mensaje));
  useEffect(() => {
    void recargar();
    window.addEventListener(EVENTO_PRESETS, recargar);
    return () => window.removeEventListener(EVENTO_PRESETS, recargar);
  }, []);

  const avisar = (nueva: puente.PresetGuardado[]) => {
    setLista(nueva);
    window.dispatchEvent(new Event(EVENTO_PRESETS));
  };

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.presets")} />
      <div className="contenido">
        <div className="panel-ajustes">
          <p className="apagado">{t("presets.entradilla")}</p>
          {fallo && <p className="error">{fallo}</p>}
          {lista && lista.length === 0 && (
            <section className="grupo vacio" data-prueba="presets-vacio">
              <Icono nombre="presets" lado={36} />
              <p className="zona-titulo">{t("presets.vacio")}</p>
              <p className="apagado">{t("presets.comoCrear")}</p>
              <button onClick={irAlEstudio}>{t("presets.irAlEstudio")}</button>
            </section>
          )}
          {lista?.map((p) => (
            <Ficha
              key={p.nombre}
              preset={p}
              usar={() => {
                window.dispatchEvent(new CustomEvent<OpcionesWebp>(EVENTO_APLICAR, { detail: p.webp }));
                irAlEstudio();
              }}
              renombrar={async (nuevo) => {
                await puente.guardarPreset({ ...p, nombre: nuevo });
                avisar(await puente.borrarPreset(p.nombre));
              }}
              borrar={async () => avisar(await puente.borrarPreset(p.nombre))}
            />
          ))}
          {carpeta && (
            <p className="apagado pie-presets">
              {t("presets.donde")} <code className="ruta seleccionable">{carpeta}</code>
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

function Ficha({
  preset,
  usar,
  renombrar,
  borrar,
}: {
  preset: puente.PresetGuardado;
  usar: () => void;
  renombrar: (nombre: string) => Promise<void>;
  borrar: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [orden, setOrden] = useState("");
  const [nombre, setNombre] = useState<string | null>(null);
  const [seguro, setSeguro] = useState(false);
  const w = preset.webp;

  useEffect(() => {
    puente.ordenOpciones(w).then(setOrden, () => {});
  }, [w]);

  // Lo que hace el preset en una línea: lo que más cambia el resultado.
  const rasgos = [
    "WebP",
    w.sin_perdida
      ? w.casi_sin_perdida < 100
        ? t("presets.casiSinPerdida", { valor: w.casi_sin_perdida })
        : t("presets.sinPerdida")
      : t("presets.calidad", { valor: w.calidad }),
    t("presets.metodo", { valor: w.metodo }),
    ...(w.preset ? [t(`presetCwebp.${w.preset}`)] : []),
    ...(w.redimension ? [t("presets.redimension", { ancho: w.redimension.ancho || "auto", alto: w.redimension.alto || "auto" })] : []),
    ...(w.enderezar ? [t("presets.endereza")] : []),
  ];

  return (
    <section className="grupo ficha-preset" data-prueba="preset">
      <div className="ficha-preset-cabecera">
        {nombre === null ? (
          <h2>{preset.nombre}</h2>
        ) : (
          <form
            className="guardar"
            onSubmit={async (e) => {
              e.preventDefault();
              if (nombre.trim() && nombre.trim() !== preset.nombre) await renombrar(nombre.trim());
              setNombre(null);
            }}
          >
            <input autoFocus value={nombre} onChange={(e) => setNombre(e.target.value)} aria-label={t("presets.nombre")} />
            <button type="submit">{t("presets.aceptar")}</button>
            <button type="button" onClick={() => setNombre(null)}>
              {t("orden.cancelar")}
            </button>
          </form>
        )}
      </div>
      <ul className="rasgos">
        {rasgos.map((r) => (
          <li key={r}>{r}</li>
        ))}
      </ul>
      <code className="orden-preset seleccionable">apolo webp -apolo_preset "{preset.nombre}"</code>
      <p className="apagado equivale">
        {t("presets.equivale")} <code className="seleccionable">{orden}</code>
      </p>
      <div className="acciones">
        <button className="principal" onClick={usar}>
          {t("presets.usar")}
        </button>
        <button onClick={() => setNombre(preset.nombre)}>{t("presets.renombrar")}</button>
        <span className="separador" />
        <button className={seguro ? "peligro" : ""} onClick={() => (seguro ? void borrar() : setSeguro(true))} onBlur={() => setSeguro(false)}>
          {seguro ? t("presets.seguro") : t("ajustes.borrar")}
        </button>
      </div>
    </section>
  );
}
