// El Estudio: una imagen, afinada viéndola (ADR 0004).
//
// Cada cambio de un control pide una vista previa nueva con una generación
// mayor, tras un pequeño retardo. Rust cancela la que esté en marcha con una
// generación anterior, y aquí se descarta cualquier respuesta que no sea la
// última pedida: lo que se ve siempre corresponde a los controles.

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cabecera, Icono, IconoApp, Marca } from "../componentes";
import * as puente from "../puente";
import { Comparador, type Modo } from "./Comparador";
import type { OpcionesWebp, Preset } from "./opciones";
import { Panel } from "./Panel";

const RETARDO_MS = 120;

/** Eventos de ventana entre la sección Presets y el Estudio. */
export const EVENTO_PRESETS = "apolo:presets";
export const EVENTO_APLICAR = "apolo:aplicar-preset";

export function formatoBytes(n: number): string {
  const f = (v: number, d: number) => v.toLocaleString("es", { maximumFractionDigits: d, minimumFractionDigits: d });
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${f(n / 1024, n < 10 * 1024 ? 1 : 0)} KB`;
  return `${f(n / 1024 / 1024, 1)} MB`;
}

export function Estudio({ inicio }: { inicio: puente.Inicio }) {
  const { t } = useTranslation();
  const [info, setInfo] = useState<puente.InfoImagen | null>(null);
  const [opciones, setOpciones] = useState<OpcionesWebp>(inicio.opciones);
  const [original, setOriginal] = useState<ImageData | null>(null);
  const [resultado, setResultado] = useState<ImageData | null>(null);
  const [vista, setVista] = useState<puente.Vista | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [aviso, setAviso] = useState<string | null>(null);
  const [modo, setModo] = useState<Modo>("deslizador");
  const [guardados, setGuardados] = useState<puente.PresetGuardado[]>([]);
  const [sobre, setSobre] = useState(false);
  const generacion = useRef(0);

  // La sección Presets avisa por eventos de ventana: cuando se aplica uno al
  // Estudio y cuando cambia la lista (guardar, renombrar, borrar).
  useEffect(() => {
    const recargar = () => puente.presets().then(setGuardados, () => {});
    const aplicar = (e: Event) => setOpciones((e as CustomEvent<OpcionesWebp>).detail);
    recargar();
    window.addEventListener(EVENTO_PRESETS, recargar);
    window.addEventListener(EVENTO_APLICAR, aplicar);
    return () => {
      window.removeEventListener(EVENTO_PRESETS, recargar);
      window.removeEventListener(EVENTO_APLICAR, aplicar);
    };
  }, []);

  const abrir = useCallback(
    async (fuente: string | File) => {
      setError(null);
      try {
        const nueva = typeof fuente === "string" ? await puente.abrirRuta(fuente) : await puente.abrirFichero(fuente);
        if (info) void puente.cerrar(info.id);
        setResultado(null);
        setVista(null);
        setInfo(nueva);
      } catch (e) {
        setError((e as puente.Fallo).mensaje);
      }
    },
    [info],
  );

  // Soltar ficheros sobre la ventana de la aplicación.
  useEffect(() => {
    let quitar = () => {};
    void puente.alSoltar((rutas) => rutas[0] && void abrir(rutas[0])).then((f) => (quitar = f));
    return () => quitar();
  }, [abrir]);

  // Los píxeles del original: al abrir, y al enderezar o dejar de hacerlo.
  useEffect(() => {
    if (!info) return;
    let vivo = true;
    puente.pixelesOriginal(info.id, opciones.enderezar).then(
      (p) => vivo && setOriginal(p),
      (e: puente.Fallo) => vivo && setError(e.mensaje),
    );
    return () => {
      vivo = false;
    };
  }, [info, opciones.enderezar]);

  // La vista previa en vivo.
  useEffect(() => {
    if (!info) return;
    const g = ++generacion.current;
    setOcupado(true);
    const reloj = window.setTimeout(async () => {
      try {
        const v = await puente.codificar(info.id, opciones, g);
        if (g !== generacion.current) return;
        const px = await puente.pixelesResultado(info.id);
        if (g !== generacion.current) return;
        setVista(v);
        setResultado(px);
        setError(null);
        setOcupado(false);
      } catch (e) {
        const f = e as puente.Fallo;
        if (g !== generacion.current || f.cancelado) return;
        setError(f.mensaje);
        setOcupado(false);
      }
    }, RETARDO_MS);
    return () => window.clearTimeout(reloj);
  }, [info, opciones]);

  const aplicarPreset = async (p: Preset) => setOpciones(await puente.aplicarPreset(opciones, p));
  const nivelSinPerdida = async (n: number) => setOpciones(await puente.nivelSinPerdida(opciones, n));
  const guardar = async (nombre: string) => {
    try {
      setGuardados(await puente.guardarPreset({ nombre, formato: "webp", webp: opciones }));
      window.dispatchEvent(new Event(EVENTO_PRESETS));
      setAviso(t("estudio.presetGuardado", { nombre }));
    } catch (e) {
      setError((e as puente.Fallo).mensaje);
    }
  };
  const borrar = async (nombre: string) => setGuardados(await puente.borrarPreset(nombre));

  const exportar = async () => {
    if (!info) return;
    try {
      const r = await puente.exportar(info.id, opciones);
      if (r) setAviso(t("estudio.exportado", { nombre: r }));
    } catch (e) {
      setError((e as puente.Fallo).mensaje);
    }
  };

  const elegir = async () => {
    const f = await puente.elegirImagen();
    if (f) void abrir(f);
  };

  if (!info) {
    return (
      <div className="seccion columna">
        <Cabecera titulo={t("nav.estudio")} />
        <div className="contenido bienvenida">
          <div className="bienvenida-cabecera">
            <IconoApp lado={88} clase="bienvenida-icono" />
            <h2>{t("estudio.bienvenida")}</h2>
            <p className="apagado">{t("app.lema")}</p>
          </div>
          <section
            className={`zona-soltar${sobre ? " sobre" : ""}`}
            data-prueba="zona"
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
            <p className="apagado">{t("estudio.soltarDetalle")}</p>
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

  const psnr = vista?.estadisticas.psnr[3];

  return (
    <div className="estudio">
      <div className="lienzo-y-barra">
        <Cabecera
          titulo={info.nombre}
          antetitulo={`${info.formato} · ${info.ancho} × ${info.alto}${info.alfa ? ` · ${t("estudio.conAlfa")}` : ""}`}
        >
          <div className="segmentado con-iconos" role="radiogroup" aria-label={t("comparador.modo")}>
            {(["deslizador", "ladoALado"] as const).map((m) => (
              <button key={m} role="radio" aria-checked={modo === m} onClick={() => setModo(m)}>
                <Icono nombre={m} lado={15} />
                {t(`comparador.${m}`)}
              </button>
            ))}
          </div>
          <button onClick={elegir}>
            <Icono nombre="abrir" />
            {t("estudio.abrirOtra")}
          </button>
        </Cabecera>

        {info.orientacion !== 1 && !opciones.enderezar && (
          <div className="aviso" role="status" data-prueba="aviso-orientacion">
            {t("estudio.avisoOrientacion")}
            <button onClick={() => setOpciones({ ...opciones, enderezar: true })}>{t("estudio.enderezar")}</button>
          </div>
        )}

        <Comparador original={original} resultado={resultado} modo={modo} ocupado={ocupado} />

        <footer className="barra-estado" data-prueba="barra-estado">
          <div className="fila-estado">
            <Pesos info={info} vista={vista} />
            <span className="separador" />
            {vista && (
              <div className="datos apagado">
                {vista.ancho} × {vista.alto} · {vista.milisegundos} ms
                {psnr !== undefined && !opciones.sin_perdida && ` · PSNR ${psnr.toLocaleString("es", { maximumFractionDigits: 1 })} dB`}
              </div>
            )}
            <button className="principal" onClick={exportar} disabled={!vista} data-prueba="exportar">
              <Icono nombre="exportar" />
              {t("estudio.exportar")}
            </button>
          </div>
          <OrdenCwebp vista={vista} formato={info.formato} cambiar={setOpciones} />
          {vista?.motivo && (
            <p className="motivo apagado" data-prueba="motivo">
              {t(`orden.motivo.${vista.motivo}`)}
            </p>
          )}
        </footer>
        {(error || aviso) && (
          <p className={error ? "error mensaje" : "exito mensaje"} role={error ? "alert" : "status"} onClick={() => { setError(null); setAviso(null); }}>
            {error ?? aviso}
          </p>
        )}
      </div>

      <Panel
        opciones={opciones}
        contexto={{ alfa: info.alfa, ancho: info.ancho, alto: info.alto }}
        presetsCwebp={inicio.presets_cwebp}
        guardados={guardados}
        cambiar={setOpciones}
        aplicarPreset={aplicarPreset}
        nivelSinPerdida={nivelSinPerdida}
        guardar={guardar}
        borrar={borrar}
      />
    </div>
  );
}

/**
 * Los dos pesos, con lo que son, el ahorro grande y una barra que los compara.
 * La v0.3.1 enseñaba «4,8 KB → 616 B −87 %» sin decir qué era cada número.
 */
function Pesos({ info, vista }: { info: puente.InfoImagen; vista: puente.Vista | null }) {
  const { t } = useTranslation();
  const ahorro = vista ? 1 - vista.bytes / info.bytes : 0;
  const crece = ahorro < 0;
  const proporcion = vista ? Math.min(1, vista.bytes / info.bytes) : 1;
  return (
    <div className="pesos" data-prueba="pesos">
      <div className="peso">
        <span className="etiqueta">{t("pesos.original", { formato: info.formato })}</span>
        <span className="valor">{formatoBytes(info.bytes)}</span>
      </div>
      <div className="peso">
        <span className="etiqueta">{t("pesos.resultado")}</span>
        <strong className="valor" data-prueba="peso-resultado">{vista ? formatoBytes(vista.bytes) : "…"}</strong>
      </div>
      {vista && (
        <div className={`ahorro ${crece ? "crece" : ""}`} data-prueba="ahorro">
          <span className="ahorro-cifra">
            {crece ? "+" : "−"}
            {Math.abs(ahorro * 100).toLocaleString("es", { maximumFractionDigits: 1 })} %
          </span>
          <span className="etiqueta">
            {crece
              ? t("pesos.mas", { bytes: formatoBytes(vista.bytes - info.bytes) })
              : t("pesos.menos", { bytes: formatoBytes(info.bytes - vista.bytes) })}
          </span>
        </div>
      )}
      <div
        className="barra-pesos"
        role="img"
        aria-label={vista ? t("pesos.barra", { porcentaje: Math.round(proporcion * 100) }) : ""}
      >
        <span className="barra-resultado" style={{ width: `${Math.max(proporcion * 100, 1.5)}%` }} />
      </div>
    </div>
  );
}

/** La orden cwebp equivalente: se copia, y se puede pegar otra para cargarla. */
function OrdenCwebp({
  vista,
  formato,
  cambiar,
}: {
  vista: puente.Vista | null;
  formato: string;
  cambiar: (o: OpcionesWebp) => void;
}) {
  const { t } = useTranslation();
  const [editando, setEditando] = useState(false);
  const [texto, setTexto] = useState("");
  const [fallo, setFallo] = useState<string | null>(null);
  const [copiado, setCopiado] = useState(false);

  if (editando) {
    return (
      <form
        className="orden editando"
        onSubmit={async (e) => {
          e.preventDefault();
          try {
            cambiar(await puente.leerOrden(texto));
            setEditando(false);
            setFallo(null);
          } catch (er) {
            setFallo((er as puente.Fallo).mensaje);
          }
        }}
      >
        <input autoFocus value={texto} onChange={(e) => setTexto(e.target.value)} placeholder={t("orden.pegar")} aria-label={t("orden.pegar")} data-prueba="orden-entrada" />
        <button type="submit">{t("orden.cargar")}</button>
        <button type="button" onClick={() => setEditando(false)}>{t("orden.cancelar")}</button>
        {fallo && <span className="error">{fallo}</span>}
      </form>
    );
  }
  return (
    <div className="orden">
      <code title={vista?.orden} data-prueba="orden">{vista?.orden ?? "cwebp …"}</code>
      {vista && vista.motivo && (
        <span className="aviso-orden" title={t(`orden.motivo.${vista.motivo}`)} data-prueba="no-equivalente">
          {/* Una frase corta según el caso; la explicación larga va debajo. */}
          {t(`orden.aviso.${vista.motivo}`, { formato })}
        </span>
      )}
      <button
        onClick={async () => {
          if (!vista) return;
          // Se copia con las rutas completas, para que funcione pegada en
          // cualquier carpeta; en pantalla se ve corta.
          await navigator.clipboard.writeText(vista.orden_completa);
          setCopiado(true);
          window.setTimeout(() => setCopiado(false), 1500);
        }}
        disabled={!vista}
      >
        <Icono nombre="copiar" />
        {copiado ? t("orden.copiada") : t("orden.copiar")}
      </button>
      <button onClick={() => { setTexto(vista?.orden ?? ""); setEditando(true); }}>
        <Icono nombre="pegar" />
        {t("orden.editar")}
      </button>
    </div>
  );
}
