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
import { FORMATOS, type Ajuste, type FormatoSalida, type Preset } from "../estudio/opciones";
import { banda, nombreFormato } from "../estudio/Panel";
import * as puente from "../puente";

const CADA_MS = 250;
/** Las filas que se pintan, como mucho: con miles, la lista no se mira entera. */
const FILAS_VISIBLES = 200;

type Momento =
  | { es: "preparando" }
  | { es: "convirtiendo"; lote: puente.LoteEmpezado; estado: puente.EstadoLote | null; filas: puente.Fila[] }
  | { es: "terminado"; lote: puente.LoteEmpezado; estado: puente.EstadoLote; filas: puente.Fila[] };

/** Una salida del lote: un formato y su preset («defecto», «cwebp:photo» o «apolo:Nombre»). */
interface SalidaLote {
  formato: FormatoSalida;
  eleccion: string;
}

const EXTENSION: Record<FormatoSalida, string> = { webp: "webp", jpeg: "jpg", png: "png", qoi: "qoi", avif: "avif", jxl: "jxl" };
const HERRAMIENTA: Record<FormatoSalida, string> = {
  webp: "cwebp",
  jpeg: "cjpeg",
  png: "oxipng",
  qoi: "qoiconv",
  avif: "avifenc",
  jxl: "cjxl",
};

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
  // Una salida por formato pedido (ADR 0020): su formato y su preset.
  const [salidas, setSalidas] = useState<SalidaLote[]>([{ formato: "webp", eleccion: "defecto" }]);
  const [ajustes, setAjustes] = useState<Ajuste[]>([inicio.ajuste]);
  const [ordenes, setOrdenes] = useState<string[]>([]);
  const [soloMasLigero, setSoloMasLigero] = useState(false);
  // Medir la calidad y buscar una nota (ADR 0022): apagados, que alargan el lote.
  const [medir, setMedir] = useState(false);
  const [objetivo, setObjetivo] = useState<number | null>(null);
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

  // El ajuste de cada salida (su formato con su preset), y la orden de su
  // herramienta para enseñarla.
  useEffect(() => {
    let vale = true;
    const base = inicio.ajuste;
    Promise.all(
      salidas.map(async (s): Promise<Ajuste> => {
        if (s.eleccion.startsWith("cwebp:"))
          return { ...base, formato: "webp", webp: await puente.aplicarPreset(base.webp, s.eleccion.slice(6) as Preset) };
        if (s.eleccion.startsWith("apolo:")) {
          const g = guardados.find((x) => x.nombre === s.eleccion.slice(6));
          if (g) {
            const { nombre: _, ...a } = g;
            return a;
          }
        }
        return { ...base, formato: s.formato };
      }),
    )
      .then(async (a) => {
        const o = await Promise.all(a.map((x) => puente.ordenOpciones(x)));
        if (vale) {
          setAjustes(a);
          setOrdenes(o);
        }
      })
      .catch(() => {});
    return () => {
      vale = false;
    };
  }, [salidas, guardados, inicio.ajuste]);

  // La carpeta propuesta lleva el sufijo de lo que se pide: «-webp», «-jpg»…
  // o «-apolo» si son varios formatos.
  const formatosPedidos = [...new Set(salidas.map((s) => s.formato))];
  const sufijo = formatosPedidos.length === 1 ? EXTENSION[formatosPedidos[0]] : "apolo";
  useEffect(() => {
    if (!salidaElegida.current && recogida?.salida_sugerida) setSalida(`${recogida.salida_sugerida}-${sufijo}`);
  }, [sufijo, recogida]);

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
      // La nota objetivo vale para todas las salidas, encima de su preset.
      const conObjetivo = objetivo === null ? ajustes : ajustes.map((a) => ({ ...a, objetivo }));
      const lote = await puente.empezarLote(entradas, conObjetivo, soloMasLigero && ajustes.length > 1, medir, salida);
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
                {salidas.map((s, i) => (
                  <FilaSalida
                    key={i}
                    indice={i}
                    salida={s}
                    orden={ordenes[i] ?? ""}
                    guardados={guardados}
                    presetsCwebp={inicio.presets_cwebp}
                    cambiar={(n) => setSalidas((x) => x.map((y, k) => (k === i ? n : y)))}
                    quitar={salidas.length > 1 ? () => setSalidas((x) => x.filter((_, k) => k !== i)) : undefined}
                  />
                ))}
                <div className="pareja">
                  <button
                    onClick={() => {
                      // El siguiente formato que no esté ya.
                      const libre = FORMATOS.find((f) => !salidas.some((s) => s.formato === f)) ?? "webp";
                      setSalidas((x) => [...x, { formato: libre, eleccion: "defecto" }]);
                    }}
                    data-prueba="anadir-formato"
                  >
                    {t("lotes.anadirFormato")}
                  </button>
                </div>
                {salidas.length > 1 && (
                  <label className="casilla" data-prueba="mas-ligero">
                    <input type="checkbox" checked={soloMasLigero} onChange={(e) => setSoloMasLigero(e.target.checked)} />
                    {t("lotes.masLigero")}
                  </label>
                )}
                {salidas.length > 1 && (
                  <p className="apagado">{t(soloMasLigero ? "lotes.masLigeroDetalle" : "lotes.todosDetalle")}</p>
                )}
              </section>

              <section className="grupo">
                <h2>{t("lotes.calidad")}</h2>
                <label className="casilla" data-prueba="medir-lote">
                  <input type="checkbox" checked={medir} onChange={(e) => setMedir(e.target.checked)} />
                  {t("lotes.medir")}
                </label>
                <p className="apagado">{t("lotes.medirDetalle")}</p>
                <label className="casilla" data-prueba="objetivo-lote">
                  <input type="checkbox" checked={objetivo !== null} onChange={(e) => setObjetivo(e.target.checked ? 80 : null)} />
                  {t("lotes.objetivo")}
                </label>
                {objetivo !== null && (
                  <div className="deslizador objetivo-lote">
                    <input
                      type="range"
                      min={30}
                      max={95}
                      step={1}
                      value={objetivo}
                      onChange={(e) => setObjetivo(Number(e.target.value))}
                      aria-label={t("panel.nota")}
                      data-prueba="nota-objetivo-lote"
                    />
                    <output>{objetivo}</output>
                    <span className="apagado">{t(`nota.${banda(objetivo)}`)}</span>
                  </div>
                )}
                <p className="apagado">{t("lotes.objetivoDetalle")}</p>
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
                <button className="peligro" onClick={alCancelar} disabled={e?.cancelado}>
                  {e?.cancelado ? t("lotes.cancelando") : t("lotes.cancelar")}
                </button>
              </div>
            </section>
          ) : (
            r && (
              <section className="grupo" data-prueba="resumen">
                <p className={momento.estado.cancelado ? "zona-titulo cancelado" : "zona-titulo"} data-prueba="titulo-resumen">
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
                {/* Con varios formatos: cuántos ficheros de cada uno y cuánto pesan
                    (todos guardados) o cuántas imágenes ganó cada uno (el más ligero). */}
                {r.por_formato.length > 1 && (
                  <ul className="por-formato" data-prueba="por-formato">
                    {r.por_formato.map((f) => (
                      <li key={f.formato}>
                        <strong>{nombreFormato(f.formato)}</strong>
                        <span className="apagado">
                          {" · "}
                          {t("lotes.ficheros", { count: f.ficheros })}
                          {" · "}
                          {formatoBytes(f.bytes_entrada)} → {formatoBytes(f.bytes_salida)}
                        </span>
                        <span className="cifra">{t("lotes.ahorro", { porcentaje: ahorro(f.bytes_entrada, f.bytes_salida) })}</span>
                      </li>
                    ))}
                  </ul>
                )}
                {r.nota_media !== null && (
                  <p data-prueba="nota-media">
                    {t("lotes.notaMedia")} <strong>{r.nota_media.toLocaleString("es", { maximumFractionDigits: 1, minimumFractionDigits: 1 })}</strong>
                    <span className="apagado"> · {t(`nota.${banda(r.nota_media)}`)}</span>
                  </p>
                )}
                {r.mayores > 0 && <p className="aviso-texto">{t("lotes.mayores", { count: r.mayores })}</p>}
                {r.fallidas > 0 && <p className="error">{t("lotes.fallidas", { count: r.fallidas })}</p>}
                <p className="apagado">{t("lotes.dondeQuedo", { salida: momento.lote.salida })}</p>
                <div className="pareja">
                  {/* La carpeta de salida entera, seleccionada en el Finder. Antes
                      señalaba la última imagen, y abría la subcarpeta en la que
                      cayera (lo vio el cliente con la v0.4.0). */}
                  {puente.enTauri() && r.convertidas > 0 && (
                    <button onClick={() => puente.mostrarEnCarpeta(momento.lote.salida)}>{t("lotes.mostrar")}</button>
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
                    fila={{
                      relativa: p.relativa,
                      bytes_entrada: p.bytes_entrada,
                      salidas: [{ formato: p.formato, bytes: p.bytes_salida, ruta: "", nota: null, calidad: null }],
                      error: null,
                    }}
                  />
                ))}
              </ul>
            </section>
          )}

          {r && r.peores_notas.length > 0 && r.convertidas > r.peores_notas.length && (
            <section className="grupo" data-prueba="peores-notas">
              <h2>{t("lotes.peoresNotas")}</h2>
              <p className="apagado">{t("lotes.peoresNotasDetalle")}</p>
              <ul className="filas-lote">
                {r.peores_notas.map((p) => (
                  <li key={`${p.relativa}-${p.formato}`}>
                    <span className="nombre" title={p.relativa}>
                      {p.relativa}
                    </span>
                    <span className="apagado">{nombreFormato(p.formato)}</span>
                    <span className="cifra">{p.nota.toLocaleString("es", { maximumFractionDigits: 1, minimumFractionDigits: 1 })}</span>
                  </li>
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
  // La más ligera de las que dejó; con varias, las demás en el título.
  const principal = fila.salidas[0];
  const bytesSalida = principal?.bytes ?? 0;
  const crece = principal !== undefined && bytesSalida > fila.bytes_entrada;
  const resto = fila.salidas
    .slice(1)
    .map((s) => `${nombreFormato(s.formato)} ${formatoBytes(s.bytes)}`)
    .join(" · ");
  return (
    <li className={fila.error ? "con-error" : undefined}>
      <span className="nombre" title={fila.relativa}>
        {fila.relativa}
      </span>
      {fila.error ? (
        <span className="error">{fila.error}</span>
      ) : (
        <>
          <span className="apagado" title={resto || undefined}>
            {formatoBytes(fila.bytes_entrada)} → {principal && `${nombreFormato(principal.formato)} `}
            {formatoBytes(bytesSalida)}
            {fila.salidas.length > 1 && " …"}
            {principal?.calidad != null && ` · ${t("lotes.calidadHallada", { calidad: principal.calidad })}`}
            {principal?.nota != null && (
              <>
                {" · "}
                <span data-prueba="nota-fila">
                  {t("lotes.nota", { nota: principal.nota.toLocaleString("es", { maximumFractionDigits: 1, minimumFractionDigits: 1 }) })}
                </span>
              </>
            )}
          </span>
          <span className={crece ? "aviso-texto cifra" : "cifra"}>
            {crece
              ? t("lotes.crece", { porcentaje: -ahorro(fila.bytes_entrada, bytesSalida) })
              : `−${ahorro(fila.bytes_entrada, bytesSalida)} %`}
          </span>
        </>
      )}
    </li>
  );
}

/** Una salida pedida: formato, preset y la orden de su herramienta. */
function FilaSalida({
  indice,
  salida: s,
  orden,
  guardados,
  presetsCwebp,
  cambiar,
  quitar,
}: {
  indice: number;
  salida: SalidaLote;
  orden: string;
  guardados: puente.PresetGuardado[];
  presetsCwebp: Preset[];
  cambiar: (s: SalidaLote) => void;
  quitar?: () => void;
}) {
  const { t } = useTranslation();
  // La primera conserva los nombres de prueba de siempre.
  const prueba = (n: string) => (indice === 0 ? n : `${n}-${indice}`);
  const suyos = guardados.filter((g) => g.formato === s.formato);
  return (
    <div className="salida-lote" data-prueba={prueba("salida-lote")}>
      <div className="pareja">
        <label className="campo">
          <span>{t("panel.formato")}</span>
          <select
            value={s.formato}
            onChange={(e) => cambiar({ formato: e.target.value as FormatoSalida, eleccion: "defecto" })}
            data-prueba={prueba("formato-lote")}
          >
            {FORMATOS.map((f) => (
              <option key={f} value={f}>
                {t(`formatoSalida.${f}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="campo crece">
          <span>{t("lotes.preset")}</span>
          <select
            value={s.eleccion}
            onChange={(e) => cambiar({ ...s, eleccion: e.target.value })}
            data-prueba={prueba("preset-lote")}
            disabled={s.formato === "qoi"}
          >
            <option value="defecto">
              {s.formato === "qoi" ? t("lotes.sinOpciones") : t("lotes.porDefecto", { herramienta: HERRAMIENTA[s.formato] })}
            </option>
            {suyos.length > 0 && (
              <optgroup label={t("panel.presetsGuardados")}>
                {suyos.map((g) => (
                  <option key={g.nombre} value={`apolo:${g.nombre}`}>
                    {g.nombre}
                  </option>
                ))}
              </optgroup>
            )}
            {s.formato === "webp" && (
              <optgroup label={t("panel.presetsCwebp")}>
                {presetsCwebp.map((p) => (
                  <option key={p} value={`cwebp:${p}`}>
                    {t(`presetCwebp.${p}`)}
                  </option>
                ))}
              </optgroup>
            )}
          </select>
        </label>
        {quitar && (
          <button className="discreto quitar-salida" onClick={quitar} aria-label={t("lotes.quitarFormato", { formato: nombreFormato(s.formato) })}>
            ×
          </button>
        )}
      </div>
      <p className="apagado">
        {t("lotes.formato", { formato: nombreFormato(s.formato) })} <code data-prueba={prueba("orden-lote")}>{orden}</code>
      </p>
    </div>
  );
}
