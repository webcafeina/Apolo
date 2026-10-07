// La sección Lotes (ADR 0019): muchas imágenes con un mismo preset.
//
// Cuatro momentos en la misma pantalla:
// - nada todavía: la zona para soltar carpetas o imágenes;
// - preparado: qué se va a convertir, con qué preset y adónde;
// - convirtiendo: el progreso, lo que va saliendo y «Cancelar»;
// - terminado: el resumen, con lo que menos ahorró y lo que falló.
//
// El trabajo va en Rust, en otro hilo. Aquí se pregunta cada cuarto de segundo
// cómo va y se pide solo lo nuevo (`estado_lote(id, desde)`).
//
// La sección no se desmonta al cambiar de sección (como el Estudio), para que
// un lote largo siga contando aunque se mire otra cosa.

import { useCallback, useEffect, useRef, useState } from "react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import { Cabecera, Icono } from "../componentes";
import { EVENTO_PRESETS, formatoBytes } from "../estudio/Estudio";
import type { OpcionesWebp, Preset } from "../estudio/opciones";
import * as puente from "../puente";

const CADA_MS = 250;
/** Las filas que se pintan, como mucho: con miles, la lista no se mira entera. */
const FILAS_VISIBLES = 200;

type Momento =
  | { es: "preparando" }
  | { es: "convirtiendo"; lote: puente.LoteEmpezado; estado: puente.EstadoLote | null; filas: puente.Fila[] }
  | { es: "terminado"; lote: puente.LoteEmpezado; estado: puente.EstadoLote; filas: puente.Fila[] };

export function ahorro(antes: number, despues: number): number {
  return antes > 0 ? Math.round((1 - despues / antes) * 100) : 0;
}

export function duracion(segundos: number, t: TFunction): string {
  if (segundos < 1) return t("lotes.tiempo.menosDeUno");
  if (segundos < 60) return t("lotes.tiempo.segundos", { s: segundos.toLocaleString("es", { maximumFractionDigits: 1 }) });
  const m = Math.floor(segundos / 60);
  const s = Math.round(segundos % 60);
  return s ? t("lotes.tiempo.minutosYSegundos", { m, s }) : t("lotes.tiempo.minutos", { m });
}

export function Lotes({ inicio, activo }: { inicio: puente.Inicio; activo: boolean }) {
  const { t } = useTranslation();
  const [entradas, setEntradas] = useState<string[]>([]);
  const [recogida, setRecogida] = useState<puente.Recogida | null>(null);
  const [salida, setSalida] = useState("");
  const [eleccion, setEleccion] = useState("defecto");
  const [opciones, setOpciones] = useState<OpcionesWebp>(inicio.opciones);
  const [orden, setOrden] = useState("");
  const [guardados, setGuardados] = useState<puente.PresetGuardado[]>([]);
  const [momento, setMomento] = useState<Momento>({ es: "preparando" });
  const [error, setError] = useState<string | null>(null);
  const [rutaDev, setRutaDev] = useState("");
  const [sobre, setSobre] = useState(false);
  // La salida la toca quien usa Apolo: entonces ya no se cambia sola.
  const salidaElegida = useRef(false);

  useEffect(() => {
    const cargar = () => puente.presets().then(setGuardados, () => {});
    void cargar();
    window.addEventListener(EVENTO_PRESETS, cargar);
    return () => window.removeEventListener(EVENTO_PRESETS, cargar);
  }, []);

  // Las opciones del preset elegido, y su orden cwebp para enseñarla.
  useEffect(() => {
    let vale = true;
    const base = inicio.opciones;
    const calcular: Promise<OpcionesWebp> = eleccion.startsWith("cwebp:")
      ? puente.aplicarPreset(base, eleccion.slice(6) as Preset)
      : eleccion.startsWith("apolo:")
        ? Promise.resolve(guardados.find((g) => g.nombre === eleccion.slice(6))?.webp ?? base)
        : Promise.resolve(base);
    calcular
      .then(async (o) => {
        // Sin opciones, `orden_opciones` da una frase («las opciones por
        // defecto»); aquí se enseña la orden tal cual: `cwebp` a secas.
        const args = await puente.ordenOpciones(o);
        const texto = args.startsWith("(") ? "cwebp" : `cwebp ${args}`;
        if (vale) {
          setOpciones(o);
          setOrden(texto);
        }
      })
      .catch(() => {});
    return () => {
      vale = false;
    };
  }, [eleccion, guardados, inicio.opciones]);

  // Lo que hay en lo elegido, cada vez que cambia.
  useEffect(() => {
    if (entradas.length === 0) {
      setRecogida(null);
      return;
    }
    let vale = true;
    puente.recogerLote(entradas).then(
      (r) => {
        if (!vale) return;
        setRecogida(r);
        if (!salidaElegida.current) setSalida(r.salida_sugerida ?? "");
      },
      (e: puente.Fallo) => vale && setError(e.mensaje),
    );
    return () => {
      vale = false;
    };
  }, [entradas]);

  const anadir = useCallback((rutas: string[]) => {
    if (rutas.length === 0) return;
    setError(null);
    setEntradas((ya) => [...ya, ...rutas.filter((r) => !ya.includes(r))]);
  }, []);

  // Soltar sobre la ventana: solo si Lotes es lo que se ve y no está convirtiendo.
  const aceptaSoltar = activo && momento.es === "preparando";
  const aceptaRef = useRef(aceptaSoltar);
  aceptaRef.current = aceptaSoltar;
  useEffect(() => {
    let quitar = () => {};
    void puente.alSoltar((rutas) => aceptaRef.current && anadir(rutas)).then((f) => (quitar = f));
    return () => quitar();
  }, [anadir]);

  // Preguntar cómo va mientras convierte.
  const loteId = momento.es === "convirtiendo" ? momento.lote.id : null;
  useEffect(() => {
    if (loteId === null) return;
    let desde = 0;
    let parado = false;
    const mirar = async () => {
      if (parado) return;
      try {
        const e = await puente.estadoLote(loteId, desde);
        desde += e.nuevas.length;
        setMomento((m) => {
          if (m.es !== "convirtiendo" || m.lote.id !== loteId) return m;
          const filas = e.nuevas.length ? [...e.nuevas.slice().reverse(), ...m.filas].slice(0, FILAS_VISIBLES) : m.filas;
          return e.terminado ? { es: "terminado", lote: m.lote, estado: e, filas } : { ...m, estado: e, filas };
        });
        if (!e.terminado) setTimeout(mirar, CADA_MS);
      } catch (e) {
        setError((e as puente.Fallo).mensaje);
      }
    };
    void mirar();
    return () => {
      parado = true;
    };
  }, [loteId]);

  async function convertir() {
    setError(null);
    try {
      const lote = await puente.empezarLote(entradas, opciones, salida);
      setMomento({ es: "convirtiendo", lote, estado: null, filas: [] });
    } catch (e) {
      setError((e as puente.Fallo).mensaje);
    }
  }

  function otroLote() {
    setEntradas([]);
    setSalida("");
    salidaElegida.current = false;
    setMomento({ es: "preparando" });
  }

  // ------------------------------------------------------------------ vistas

  if (momento.es !== "preparando") {
    return <Progreso momento={momento} alCancelar={() => puente.cancelarLote(momento.lote.id)} alOtro={otroLote} />;
  }

  const zona = (
    <section
      className={`zona-soltar${sobre ? " sobre" : ""}${entradas.length ? " compacta" : ""}`}
      data-prueba="zona-lotes"
      onDragOver={(e) => {
        e.preventDefault();
        setSobre(true);
      }}
      onDragLeave={() => setSobre(false)}
      onDrop={(e) => {
        e.preventDefault();
        setSobre(false);
        // En la ventana llegan por `alSoltar`, con sus rutas; el navegador no las da.
        if (!puente.enTauri()) setError(t("lotes.sinRutasNavegador"));
      }}
    >
      {!entradas.length && <Icono nombre="lotes" lado={40} />}
      <p className="zona-titulo">{t(entradas.length ? "lotes.anadirMas" : "lotes.soltar")}</p>
      {!entradas.length && <p className="apagado">{t("lotes.soltarDetalle")}</p>}
      <div className="pareja centrada">
        <button className={entradas.length ? undefined : "principal"} onClick={async () => anadir(await puente.elegirEntradas(true))}>
          <Icono nombre="abrir" />
          {t("lotes.elegirCarpetas")}
        </button>
        <button onClick={async () => anadir(await puente.elegirEntradas(false))}>{t("lotes.elegirImagenes")}</button>
      </div>
      {!puente.enTauri() && (
        // Solo en el navegador de desarrollo, que no sabe las rutas de lo que se suelta.
        <form
          className="pareja centrada"
          onSubmit={(e) => {
            e.preventDefault();
            if (rutaDev.trim()) anadir([rutaDev.trim()]);
            setRutaDev("");
          }}
        >
          <input
            value={rutaDev}
            onChange={(e) => setRutaDev(e.target.value)}
            placeholder={t("lotes.rutaDev")}
            aria-label={t("lotes.rutaDev")}
            data-prueba="ruta-dev"
          />
          <button type="submit">{t("lotes.anadirRuta")}</button>
        </form>
      )}
    </section>
  );

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.lotes")} />
      <div className="contenido">
        <div className="panel-ajustes lotes">
          {zona}
          {error && (
            <p className="error" role="alert">
              {error}
            </p>
          )}
          {entradas.length > 0 && (
            <>
              <section className="grupo" data-prueba="entradas">
                <h2>{t("lotes.que")}</h2>
                <ul className="lista-entradas">
                  {entradas.map((r) => (
                    <li key={r}>
                      <code title={r}>{r}</code>
                      <button
                        className="discreto"
                        aria-label={t("lotes.quitar", { ruta: r })}
                        onClick={() => setEntradas((ya) => ya.filter((x) => x !== r))}
                      >
                        ×
                      </button>
                    </li>
                  ))}
                </ul>
                {recogida && (
                  <p data-prueba="recogida">
                    {recogida.imagenes === 0
                      ? t("lotes.ninguna")
                      : t("lotes.cuantas", { count: recogida.imagenes, bytes: formatoBytes(recogida.bytes) })}
                  </p>
                )}
              </section>

              <section className="grupo">
                <h2>{t("lotes.como")}</h2>
                {/* El mismo que el del panel del Estudio: hoy solo WebP, y los
                    demás se ven como «pronto» (entrega 5). Lo pidió el cliente al
                    probar la v0.4.0, para que se vea que el formato se elige. */}
                <label className="campo">
                  <span>{t("panel.formato")}</span>
                  <select value="webp" onChange={() => {}} data-prueba="formato-lote">
                    <option value="webp">WebP</option>
                    {["avif", "jxl", "mozjpeg", "oxipng", "qoi"].map((f) => (
                      <option key={f} value={f} disabled>
                        {t(`formato.${f}`)} · {t("panel.pronto")}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="campo">
                  <span>{t("lotes.preset")}</span>
                  <select value={eleccion} onChange={(e) => setEleccion(e.target.value)} data-prueba="preset-lote">
                    <option value="defecto">{t("lotes.porDefecto")}</option>
                    {guardados.length > 0 && (
                      <optgroup label={t("panel.presetsGuardados")}>
                        {guardados.map((g) => (
                          <option key={g.nombre} value={`apolo:${g.nombre}`}>
                            {g.nombre}
                          </option>
                        ))}
                      </optgroup>
                    )}
                    <optgroup label={t("panel.presetsCwebp")}>
                      {inicio.presets_cwebp.map((p) => (
                        <option key={p} value={`cwebp:${p}`}>
                          {t(`presetCwebp.${p}`)}
                        </option>
                      ))}
                    </optgroup>
                  </select>
                </label>
                <p className="apagado">
                  {t("lotes.formato")} <code data-prueba="orden-lote">{orden}</code>
                </p>
              </section>

              <section className="grupo">
                <h2>{t("lotes.donde")}</h2>
                <div className="pareja">
                  <input
                    className="ruta-salida"
                    value={salida}
                    onChange={(e) => {
                      salidaElegida.current = true;
                      setSalida(e.target.value);
                    }}
                    aria-label={t("lotes.salida")}
                    data-prueba="salida"
                  />
                  {puente.enTauri() && (
                    <button
                      onClick={async () => {
                        const r = await puente.elegirSalida(salida || null);
                        if (r) {
                          salidaElegida.current = true;
                          setSalida(r);
                        }
                      }}
                    >
                      {t("lotes.cambiar")}
                    </button>
                  )}
                </div>
                <p className="apagado">{t("lotes.nuncaPisa")}</p>
              </section>

              <div className="pareja">
                <button
                  className="principal"
                  disabled={!recogida || recogida.imagenes === 0 || !salida.trim()}
                  onClick={convertir}
                >
                  {t("lotes.convertir", { count: recogida?.imagenes ?? 0 })}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function Progreso({
  momento,
  alCancelar,
  alOtro,
}: {
  momento: Exclude<Momento, { es: "preparando" }>;
  alCancelar: () => void;
  alOtro: () => void;
}) {
  const { t } = useTranslation();
  const e = momento.estado;
  const total = momento.lote.total;
  const hechas = e?.hechas ?? 0;
  const r = momento.es === "terminado" ? momento.estado.resumen : null;

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.lotes")} />
      <div className="contenido">
        <div className="panel-ajustes lotes">
          {momento.es === "convirtiendo" ? (
            <section className="grupo" data-prueba="progreso">
              <p className="zona-titulo">{t("lotes.convirtiendo", { hechas, total })}</p>
              <div className="barra-progreso ancha">
                <i style={{ width: `${total ? (hechas / total) * 100 : 0}%` }} />
              </div>
              <div className="pareja">
                <span className="apagado">{e ? duracion(e.segundos, t) : ""}</span>
                <span className="relleno-flex" />
                <button onClick={alCancelar} disabled={e?.cancelado}>
                  {e?.cancelado ? t("lotes.cancelando") : t("lotes.cancelar")}
                </button>
              </div>
            </section>
          ) : (
            r && (
              <section className="grupo" data-prueba="resumen">
                <p className="zona-titulo">
                  {momento.estado.cancelado
                    ? t("lotes.canceladoTitulo", { hechas: r.convertidas, total })
                    : t("lotes.hechoTitulo", { count: r.convertidas, tiempo: duracion(momento.estado.segundos, t) })}
                </p>
                {r.convertidas > 0 && (
                  <p className="ahorro-lote">
                    {formatoBytes(r.bytes_entrada)} → <strong>{formatoBytes(r.bytes_salida)}</strong>
                    <span className="apagado"> · </span>
                    <strong>{t("lotes.ahorro", { porcentaje: ahorro(r.bytes_entrada, r.bytes_salida) })}</strong>
                  </p>
                )}
                {r.mayores > 0 && <p className="aviso-texto">{t("lotes.mayores", { count: r.mayores })}</p>}
                {r.fallidas > 0 && <p className="error">{t("lotes.fallidas", { count: r.fallidas })}</p>}
                <p className="apagado">{t("lotes.dondeQuedo", { salida: momento.lote.salida })}</p>
                <div className="pareja">
                  {puente.enTauri() && momento.filas.find((f) => f.ruta) && (
                    <button onClick={() => puente.mostrarEnCarpeta(momento.filas.find((f) => f.ruta)!.ruta!)}>
                      {t("lotes.mostrar")}
                    </button>
                  )}
                  <button className="principal" onClick={alOtro}>
                    {t("lotes.otro")}
                  </button>
                </div>
              </section>
            )
          )}

          {/* Con cinco o menos, «Todas» ya es la misma lista. */}
          {r && r.convertidas > r.peores.length && (
            <section className="grupo" data-prueba="peores">
              <h2>{t("lotes.peores")}</h2>
              <p className="apagado">{t("lotes.peoresDetalle")}</p>
              <ul className="filas-lote">
                {r.peores.map((p) => (
                  <FilaLote
                    key={p.relativa}
                    fila={{ relativa: p.relativa, bytes_entrada: p.bytes_entrada, bytes_salida: p.bytes_salida, ruta: null, error: null }}
                  />
                ))}
              </ul>
            </section>
          )}

          {momento.filas.length > 0 && (
            <section className="grupo" data-prueba="filas">
              <h2>{momento.es === "terminado" ? t("lotes.todas") : t("lotes.ultimas")}</h2>
              <ul className="filas-lote">
                {(momento.es === "terminado"
                  ? [...momento.filas].sort((a, b) => Number(!!b.error) - Number(!!a.error))
                  : momento.filas
                ).map((f, i) => (
                  <FilaLote key={`${f.relativa}-${i}`} fila={f} />
                ))}
              </ul>
              {momento.es === "terminado" && hechas > FILAS_VISIBLES && (
                <p className="apagado">{t("lotes.soloUltimas", { count: FILAS_VISIBLES })}</p>
              )}
            </section>
          )}
        </div>
      </div>
    </div>
  );
}

function FilaLote({ fila }: { fila: puente.Fila }) {
  const { t } = useTranslation();
  const crece = fila.bytes_salida !== null && fila.bytes_salida > fila.bytes_entrada;
  return (
    <li className={fila.error ? "con-error" : undefined}>
      <span className="nombre" title={fila.relativa}>
        {fila.relativa}
      </span>
      {fila.error ? (
        <span className="error">{fila.error}</span>
      ) : (
        <>
          <span className="apagado">
            {formatoBytes(fila.bytes_entrada)} → {formatoBytes(fila.bytes_salida ?? 0)}
          </span>
          <span className={crece ? "aviso-texto cifra" : "cifra"}>
            {crece
              ? t("lotes.crece", { porcentaje: -ahorro(fila.bytes_entrada, fila.bytes_salida ?? 0) })
              : `−${ahorro(fila.bytes_entrada, fila.bytes_salida ?? 0)} %`}
          </span>
        </>
      )}
    </li>
  );
}
