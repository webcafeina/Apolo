// Vectorizar (ADR 0039): una imagen pasada a dibujo vectorial, viéndola.
//
// Como el Estudio: cada cambio de una opción del dibujo pide uno nuevo con una generación mayor, tras un
// retardo; Rust cancela entre prueba y prueba lo que tenga una anterior, y aquí se descarta cualquier
// respuesta que no sea la última. Las opciones de exportar (formato, estilo, grosor, DXF) no vuelven a
// vectorizar: solo cambian el fichero que se guarda.
//
// A la izquierda, lo que recibió el motor (la imagen enderezada, reducida si pasaba de 2 Mpx y sobre blanco);
// a la derecha, el dibujo pintado del mismo tamaño. Así el deslizador, el lado a lado y las diferencias son
// los del Estudio, y «Contornos» enseña los trazados y sus nodos sobre el dibujo apagado.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cabecera, Icono, IconoApp, Interruptor, lleno, Marca } from "../componentes";
import { Comparador } from "../estudio/Comparador";
import { formatoBytes } from "../estudio/Estudio";
import { banda } from "../estudio/Panel";
import { lengua } from "../i18n";
import * as puente from "../puente";
import {
  AJUSTE_INICIAL,
  ESTILOS,
  FORMATOS_VECTOR,
  leerPaleta,
  NIVELES_DXF,
  parteDibujo,
  TIPOS,
  type AjusteVector,
  type FormatoVector,
} from "./opciones";

const RETARDO_MS = 250;

type ModoVector = "deslizador" | "ladoALado" | "diferencias" | "contornos";
const MODOS: ModoVector[] = ["deslizador", "ladoALado", "diferencias", "contornos"];

const cifra = (n: number, d: number) => n.toLocaleString(lengua(), { minimumFractionDigits: d, maximumFractionDigits: d });

export function Vectorizar({ activo }: { activo: boolean }) {
  const { t } = useTranslation();
  const [info, setInfo] = useState<puente.InfoImagen | null>(null);
  const [ajuste, setAjuste] = useState<AjusteVector>(AJUSTE_INICIAL);
  const [formato, setFormato] = useState<FormatoVector>("svg");
  const [vista, setVista] = useState<puente.VistaVector | null>(null);
  const [dibujo, setDibujo] = useState<ImageData | null>(null);
  const [referencia, setReferencia] = useState<ImageData | null>(null);
  const [extra, setExtra] = useState<ImageData | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const [modo, setModo] = useState<ModoVector>("deslizador");
  const [tipoMapa, setTipoMapa] = useState<puente.TipoMapa>("diferencia");
  const [error, setError] = useState<string | null>(null);
  const [aviso, setAviso] = useState<string | null>(null);
  const [sobre, setSobre] = useState(false);
  const generacion = useRef(0);

  const abrir = useCallback(
    async (fuente: string | File) => {
      setError(null);
      try {
        const nueva = typeof fuente === "string" ? await puente.abrirRuta(fuente) : await puente.abrirFichero(fuente);
        if (info) void puente.cerrar(info.id);
        setVista(null);
        setDibujo(null);
        setReferencia(null);
        setExtra(null);
        setInfo(nueva);
      } catch (e) {
        setError((e as puente.Fallo).mensaje);
      }
    },
    [info],
  );

  // Soltar ficheros sobre la ventana: solo si esta sección es la que se ve.
  const activoRef = useRef(activo);
  activoRef.current = activo;
  useEffect(() => {
    let quitar = () => {};
    void puente
      .alSoltar((rutas) => activoRef.current && rutas[0] && void abrir(rutas[0]))
      .then((f) => (quitar = f));
    return () => quitar();
  }, [abrir]);

  // Lo que cambia el dibujo: memorizado por su contenido, para no vectorizar al cambiar solo lo de exportar.
  const dibujoPedido = JSON.stringify(parteDibujo(ajuste));
  useEffect(() => {
    if (!info) return;
    const g = ++generacion.current;
    setOcupado(true);
    const pedido: AjusteVector = { ...AJUSTE_INICIAL, ...JSON.parse(dibujoPedido) };
    const reloj = window.setTimeout(async () => {
      try {
        const v = await puente.vectorizar(info.id, pedido, g);
        if (g !== generacion.current) return;
        const [d, r] = await Promise.all([puente.pixelesVector(info.id), puente.pixelesReferenciaVector(info.id)]);
        if (g !== generacion.current) return;
        setVista(v);
        setDibujo(d);
        setReferencia(r);
        setOcupado(false);
        setError(null);
      } catch (e) {
        const f = e as puente.Fallo;
        if (g !== generacion.current || f.cancelado) return;
        setError(f.mensaje);
        setOcupado(false);
      }
    }, RETARDO_MS);
    return () => window.clearTimeout(reloj);
  }, [info, dibujoPedido]);

  // El mapa de diferencias o los contornos, cuando se miran.
  useEffect(() => {
    if (!info || !vista || (modo !== "diferencias" && modo !== "contornos")) {
      setExtra(null);
      return;
    }
    let vivo = true;
    setExtra(null);
    const pedir = modo === "contornos" ? puente.pixelesContornosVector(info.id) : puente.pixelesMapaVector(info.id, tipoMapa);
    pedir.then(
      (p) => vivo && setExtra(p),
      (e: puente.Fallo) => vivo && !e.cancelado && setError(e.mensaje),
    );
    return () => {
      vivo = false;
    };
  }, [info, vista, modo, tipoMapa]);

  const elegir = async () => {
    const f = await puente.elegirImagen();
    if (f) void abrir(f);
  };

  const exportar = async () => {
    if (!info) return;
    try {
      const r = await puente.exportarVector(info.id, formato, ajuste);
      if (r) setAviso(t("estudio.exportado", { nombre: r }));
    } catch (e) {
      setError((e as puente.Fallo).mensaje);
    }
  };

  const cambiar = (c: Partial<AjusteVector>) => setAjuste((a) => ({ ...a, ...c }));

  if (!info) {
    return (
      <div className="seccion columna">
        <Cabecera titulo={t("nav.vectorizar")} />
        <div className="contenido bienvenida">
          <div className="bienvenida-cabecera">
            <IconoApp lado={88} clase="bienvenida-icono" />
            <h2>{t("vectorizar.bienvenida")}</h2>
            <p className="apagado">{t("vectorizar.lema")}</p>
          </div>
          <section
            className={`zona-soltar${sobre ? " sobre" : ""}`}
            data-prueba="zona-vector"
            onDragOver={(e) => {
              e.preventDefault();
              setSobre(true);
            }}
            onDragLeave={() => setSobre(false)}
            onDrop={(e) => {
              e.preventDefault();
              setSobre(false);
              const f = e.dataTransfer.files[0];
              if (f && !puente.enTauri()) void abrir(f);
            }}
          >
            <Marca lado={52} clase="zona-marca" />
            <p className="zona-titulo">{t("estudio.soltar")}</p>
            <p className="apagado">{t("vectorizar.soltarDetalle")}</p>
            <button className="principal" onClick={elegir}>
              <Icono nombre="abrir" />
              {t("estudio.abrir")}
            </button>
            {error && (
              <p className="error" role="alert">
                {error}
              </p>
            )}
          </section>
        </div>
      </div>
    );
  }

  const nota = vista?.nota ?? null;
  const rotuloDibujo = vista
    ? `${t("vectorizar.dibujo")} · SVG ${formatoBytes(vista.bytes_svg)}${nota !== null ? ` · ${cifra(nota, 1)}` : ""}`
    : t("vectorizar.dibujo");
  const izquierda = modo === "diferencias" || modo === "contornos" ? null : referencia;
  const derecha = modo === "diferencias" || modo === "contornos" ? extra : dibujo;
  const rotulos: [string, string] =
    modo === "diferencias"
      ? ["", `${t(`comparador.mapa.${tipoMapa}`)} · ${rotuloDibujo}`]
      : modo === "contornos"
        ? ["", `${t("vectorizar.contornos")} · ${rotuloDibujo}`]
        : [t("comparador.original"), rotuloDibujo];

  return (
    <div className="estudio vectorizar">
      <div className="lienzo-y-barra">
        <Cabecera
          titulo={info.nombre}
          antetitulo={`${info.formato} · ${info.ancho} × ${info.alto}${info.alfa ? ` · ${t("estudio.conAlfa")}` : ""}`}
        >
          <div className="segmentado con-iconos" role="radiogroup" aria-label={t("comparador.modo")}>
            {MODOS.map((m) => (
              <button
                key={m}
                role="radio"
                aria-checked={modo === m}
                aria-label={m === "contornos" ? t("vectorizar.contornos") : t(`comparador.${m}`)}
                // Sin sitio solo queda el icono: el nombre sale al dejar el puntero.
                title={m === "contornos" ? t("vectorizar.contornos") : t(`comparador.${m}`)}
                onClick={() => setModo(m)}
              >
                <Icono nombre={m} lado={15} />
                <span className="etiqueta">{m === "contornos" ? t("vectorizar.contornos") : t(`comparador.${m}`)}</span>
              </button>
            ))}
          </div>
          <button onClick={elegir}>
            <Icono nombre="abrir" />
            {t("estudio.abrirOtra")}
          </button>
        </Cabecera>

        {vista?.reducida_desde && (
          <div className="aviso" role="status" data-prueba="aviso-reducida">
            {t("vectorizar.avisoReducida", {
              ancho: vista.reducida_desde[0],
              alto: vista.reducida_desde[1],
              w: vista.ancho,
              h: vista.alto,
            })}
          </div>
        )}
        {vista?.tipo === "foto" && (
          <div className="aviso" role="status" data-prueba="aviso-foto">
            {t("vectorizar.avisoFoto")}
          </div>
        )}

        <Comparador
          izquierda={izquierda}
          derecha={derecha}
          rotulos={rotulos}
          ocupados={
            modo === "diferencias" || modo === "contornos"
              ? [false, ocupado || (!!vista && !extra)]
              : [false, ocupado]
          }
          modo={modo === "contornos" ? "diferencias" : modo}
          extra={
            modo === "diferencias" ? (
              <select
                value={tipoMapa}
                onChange={(e) => setTipoMapa(e.target.value as puente.TipoMapa)}
                aria-label={t("comparador.tipoMapa")}
              >
                <option value="diferencia">{t("comparador.mapa.diferencia")}</option>
                <option value="estructura">{t("comparador.mapa.estructura")}</option>
              </select>
            ) : undefined
          }
        />

        <footer className="barra-estado" data-prueba="barra-estado-vector">
          <div className="fila-estado">
            <PesosVector info={info} vista={vista} />
            <div className="derecha-estado">
              <MedicionVector vista={vista} ocupado={ocupado} objetivo={ajuste.objetivo} />
              <button className="principal" onClick={exportar} disabled={!vista} data-prueba="exportar-vector">
                <Icono nombre="exportar" />
                {t("vectorizar.exportar", { formato: formato.toUpperCase() })}
              </button>
            </div>
          </div>
        </footer>
        {(error || aviso) && (
          <p
            className={error ? "error mensaje" : "exito mensaje"}
            role={error ? "alert" : "status"}
            onClick={() => {
              setError(null);
              setAviso(null);
            }}
          >
            {error ?? aviso}
          </p>
        )}
      </div>

      <PanelVector ajuste={ajuste} cambiar={cambiar} formato={formato} elegirFormato={setFormato} />
    </div>
  );
}

/** El peso del original y el del dibujo en SVG, con el ahorro, como en el Estudio. */
function PesosVector({ info, vista }: { info: puente.InfoImagen; vista: puente.VistaVector | null }) {
  const { t } = useTranslation();
  const bytes = vista?.bytes_svg ?? 0;
  const ahorro = vista ? 1 - bytes / info.bytes : 0;
  const crece = ahorro < 0;
  const proporcion = vista ? Math.min(1, bytes / info.bytes) : 1;
  return (
    <div className="pesos" data-prueba="pesos-vector">
      <div className="peso">
        <span className="etiqueta">{t("pesos.original", { formato: info.formato })}</span>
        <span className="valor">{formatoBytes(info.bytes)}</span>
      </div>
      <div className="peso">
        <span className="etiqueta">{t("pesos.resultado", { formato: "SVG" })}</span>
        <strong className="valor" data-prueba="peso-vector">
          {vista ? formatoBytes(bytes) : "…"}
        </strong>
      </div>
      {vista && (
        <div className={`ahorro ${crece ? "crece" : ""}`}>
          <span className="ahorro-cifra">
            {crece ? "+" : "−"}
            {Math.abs(ahorro * 100).toLocaleString(lengua(), { maximumFractionDigits: 1 })} %
          </span>
          <span className="etiqueta">
            {crece ? t("pesos.mas", { bytes: formatoBytes(bytes - info.bytes) }) : t("pesos.menos", { bytes: formatoBytes(info.bytes - bytes) })}
          </span>
        </div>
      )}
      <div className="barra-pesos" role="img" aria-label={vista ? t("pesos.barra", { porcentaje: Math.round(proporcion * 100) }) : ""}>
        <span className="barra-resultado" style={{ width: `${Math.max(proporcion * 100, 1.5)}%` }} />
      </div>
    </div>
  );
}

/** La nota del dibujo, lo que tiene (formas, nodos, colores) y, con objetivo, lo que encontró la búsqueda. */
function MedicionVector({ vista, ocupado, objetivo }: { vista: puente.VistaVector | null; ocupado: boolean; objetivo: number | null }) {
  const { t } = useTranslation();
  if (ocupado) {
    return (
      <div className="medicion" data-prueba="medidas-vector">
        <span className="trabajando" role="status" data-prueba="vectorizando">
          <span className="girando" aria-hidden="true" />
          {objetivo !== null ? t("vectorizar.buscando", { nota: objetivo }) : t("vectorizar.trabajando")}
        </span>
      </div>
    );
  }
  if (!vista) return null;
  const e = vista.estadisticas;
  const h = vista.hallado;
  return (
    <div className="medicion" data-prueba="medidas-vector">
      {vista.nota !== null && (
        <span className="nota" title={t("medidas.ayudaNota")}>
          <span className="etiqueta entera">{t("medidas.nota")}</span>
          <strong className="entera" data-prueba="nota-vector">
            {cifra(vista.nota, 1)}
          </strong>
          <span className="apagado entera">{t(`nota.${banda(vista.nota)}`)}</span>
        </span>
      )}
      {h && (
        <span className={h.alcanzada ? "apagado" : "aviso-texto"} data-prueba="hallado">
          {h.alcanzada
            ? t("vectorizar.hallado", { detalle: h.detalle, pruebas: h.pruebas })
            : t("vectorizar.noAlcanzado", { detalle: h.detalle })}
        </span>
      )}
      <span className="apagado" data-prueba="estadisticas-vector">
        <span className="entera">{t(`vectorizar.tipos.${vista.tipo}`)}</span>
        {" · "}
        <span className="entera">{t("vectorizar.formas", { n: e.formas })}</span>
        {" · "}
        <span className="entera">{t("vectorizar.nodos", { n: e.nodos })}</span>
        {" · "}
        <span className="entera">{t("vectorizar.colores_n", { n: e.colores })}</span>
        {" · "}
        <span className="entera">
          {vista.ancho} × {vista.alto}
        </span>
        {" · "}
        <span className="entera">{vista.milisegundos} ms</span>
      </span>
    </div>
  );
}

const CLAVE_PLEGADO = "apolo.vectorizar.plegado";

function leerPlegado(): Record<string, boolean> {
  const base = { dibujo: true, avanzado: false, exportar: true };
  try {
    const v = JSON.parse(localStorage.getItem(CLAVE_PLEGADO) ?? "null");
    if (v && typeof v === "object") return { ...base, ...v };
  } catch {
    /* sin almacenamiento: los valores por defecto */
  }
  return base;
}

/** El panel: el dibujo (tipo, nota o detalle, colores, degradados), lo avanzado y cómo se exporta. */
function PanelVector({
  ajuste: a,
  cambiar,
  formato,
  elegirFormato,
}: {
  ajuste: AjusteVector;
  cambiar: (c: Partial<AjusteVector>) => void;
  formato: FormatoVector;
  elegirFormato: (f: FormatoVector) => void;
}) {
  const { t } = useTranslation();
  const [abiertos, setAbiertos] = useState(leerPlegado);
  useEffect(() => {
    try {
      localStorage.setItem(CLAVE_PLEGADO, JSON.stringify(abiertos));
    } catch {
      /* sin almacenamiento */
    }
  }, [abiertos]);
  const [paletaTexto, setPaletaTexto] = useState(a.paleta.join(", "));
  const paleta = useMemo(() => leerPaleta(paletaTexto), [paletaTexto]);

  const plegable = (clave: string, titulo: string, contenido: React.ReactNode) => (
    <details
      key={clave}
      open={abiertos[clave]}
      onToggle={(e) => {
        const abierto = (e.currentTarget as HTMLDetailsElement).open;
        setAbiertos((x) => (x[clave] === abierto ? x : { ...x, [clave]: abierto }));
      }}
      data-prueba={`vector-${clave}`}
    >
      <summary>{titulo}</summary>
      <div className="controles">{contenido}</div>
    </details>
  );

  const fila = (clave: string, etiqueta: string, entrada: React.ReactNode, ayuda: string, interruptor?: React.ReactNode, apagado = false) => (
    <div className={`fila-control${apagado ? " apagado" : ""}`} data-prueba={`vector-control-${clave}`}>
      <div className="cabecera-control">
        {interruptor}
        {/* La etiqueta va al deslizador o al campo, si lo hay; si no, al interruptor. */}
        <label htmlFor={entrada ? `vector-${clave}` : `vector-${clave}-activo`}>{etiqueta}</label>
      </div>
      {entrada}
      <p className="ayuda">{ayuda}</p>
    </div>
  );

  const deslizador = (clave: string, valor: number, min: number, max: number, paso: number, poner: (n: number) => void, desactivado = false) => (
    <div className="deslizador">
      <input
        id={`vector-${clave}`}
        type="range"
        min={min}
        max={max}
        step={paso}
        value={valor}
        style={lleno(valor, min, max)}
        disabled={desactivado}
        onChange={(e) => poner(Number(e.target.value))}
      />
      <output htmlFor={`vector-${clave}`}>{valor}</output>
    </div>
  );

  const conObjetivo = a.objetivo !== null;
  const trazo = a.estilo !== "rellenar";

  return (
    <aside className="panel" aria-label={t("vectorizar.panel")}>
      <div className="panel-cabeza">
        <label className="campo">
          <span>{t("vectorizar.tipo")}</span>
          <select value={a.tipo} onChange={(e) => cambiar({ tipo: e.target.value as AjusteVector["tipo"] })} data-prueba="vector-tipo">
            {TIPOS.map((x) => (
              <option key={x} value={x}>
                {t(`vectorizar.tipos.${x}`)}
              </option>
            ))}
          </select>
        </label>
      </div>
      {plegable(
        "dibujo",
        t("vectorizar.grupoDibujo"),
        <>
          {fila(
            "objetivo",
            t("vectorizar.objetivo"),
            conObjetivo ? deslizador("objetivo", a.objetivo!, 50, 95, 1, (n) => cambiar({ objetivo: n })) : null,
            t("vectorizar.objetivoAyuda"),
            <Interruptor
              id="vector-objetivo-activo"
              checked={conObjetivo}
              onChange={(e) => cambiar({ objetivo: e.target.checked ? 75 : null, detalle: e.target.checked ? 0 : 75 })}
            />,
          )}
          {fila(
            "detalle",
            conObjetivo ? t("vectorizar.detalleMinimo") : t("vectorizar.detalle"),
            deslizador("detalle", a.detalle, 0, 100, 1, (n) => cambiar({ detalle: n })),
            conObjetivo ? t("vectorizar.detalleMinimoAyuda") : t("vectorizar.detalleAyuda"),
          )}
          {fila(
            "colores",
            t("vectorizar.colores"),
            a.colores !== null ? deslizador("colores", a.colores, 2, 64, 1, (n) => cambiar({ colores: n })) : null,
            a.colores !== null ? t("vectorizar.coloresAyuda") : t("vectorizar.coloresTodos"),
            <Interruptor id="vector-colores-activo" checked={a.colores !== null} onChange={(e) => cambiar({ colores: e.target.checked ? 8 : null })} />,
          )}
          {fila(
            "degradados",
            t("vectorizar.degradados"),
            null,
            a.colores !== null || a.paleta.length > 0 ? t("vectorizar.degradadosNo") : t("vectorizar.degradadosAyuda"),
            <Interruptor
              id="vector-degradados-activo"
              checked={a.degradados && a.colores === null && a.paleta.length === 0}
              disabled={a.colores !== null || a.paleta.length > 0}
              onChange={(e) => cambiar({ degradados: e.target.checked })}
            />,
            a.colores !== null || a.paleta.length > 0,
          )}
        </>,
      )}
      {plegable(
        "avanzado",
        t("vectorizar.grupoAvanzado"),
        <>
          {fila(
            "paleta",
            t("vectorizar.paleta"),
            <input
              id="vector-paleta"
              type="text"
              value={paletaTexto}
              placeholder="#1e46c8, #ffffff"
              spellCheck={false}
              onChange={(e) => setPaletaTexto(e.target.value)}
              onBlur={() => cambiar({ paleta: paleta.colores })}
              onKeyDown={(e) => e.key === "Enter" && cambiar({ paleta: paleta.colores })}
            />,
            paleta.malos.length > 0 ? t("vectorizar.paletaMala", { malos: paleta.malos.join(", ") }) : t("vectorizar.paletaAyuda"),
          )}
          {fila(
            "area",
            t("vectorizar.areaMinima"),
            deslizador("area", a.area_minima, 0, 50, 0.1, (n) => cambiar({ area_minima: n })),
            t("vectorizar.areaMinimaAyuda"),
          )}
        </>,
      )}
      {plegable(
        "exportar",
        t("vectorizar.grupoExportar"),
        <>
          {fila(
            "formato",
            t("vectorizar.formato"),
            <select id="vector-formato" value={formato} onChange={(e) => elegirFormato(e.target.value as FormatoVector)} data-prueba="vector-formato">
              {FORMATOS_VECTOR.map((f) => (
                <option key={f} value={f}>
                  {t(`vectorizar.formatos.${f}`)}
                </option>
              ))}
            </select>,
            t("vectorizar.formatoAyuda"),
          )}
          {fila(
            "estilo",
            t("vectorizar.estilo"),
            <select id="vector-estilo" value={a.estilo} onChange={(e) => cambiar({ estilo: e.target.value as AjusteVector["estilo"] })} data-prueba="vector-estilo">
              {ESTILOS.map((x) => (
                <option key={x} value={x}>
                  {t(`vectorizar.estilos.${x}`)}
                </option>
              ))}
            </select>,
            t("vectorizar.estiloAyuda"),
          )}
          {trazo &&
            fila(
              "grosor",
              t("vectorizar.grosor"),
              deslizador("grosor", a.grosor, 0.1, 10, 0.1, (n) => cambiar({ grosor: n })),
              t("vectorizar.grosorAyuda"),
            )}
          {trazo &&
            fila(
              "color-trazo",
              t("vectorizar.colorTrazo"),
              a.color_trazo !== null ? (
                <input id="vector-color-trazo" type="color" value={a.color_trazo} onChange={(e) => cambiar({ color_trazo: e.target.value })} />
              ) : null,
              a.color_trazo !== null ? t("vectorizar.colorTrazoAyuda") : t("vectorizar.colorTrazoCapa"),
              <Interruptor id="vector-color-trazo-activo" checked={a.color_trazo !== null} onChange={(e) => cambiar({ color_trazo: e.target.checked ? "#000000" : null })} />,
            )}
          {formato === "dxf" &&
            fila(
              "dxf",
              t("vectorizar.nivelDxf"),
              <select id="vector-dxf" value={a.nivel_dxf} onChange={(e) => cambiar({ nivel_dxf: e.target.value as AjusteVector["nivel_dxf"] })} data-prueba="vector-dxf">
                {NIVELES_DXF.map((x) => (
                  <option key={x} value={x}>
                    {t(`vectorizar.niveles.${x}`)}
                  </option>
                ))}
              </select>,
              t("vectorizar.nivelDxfAyuda"),
            )}
        </>,
      )}
    </aside>
  );
}
