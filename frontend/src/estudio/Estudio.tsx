// El Estudio: una imagen, afinada viéndola (ADR 0004).
//
// Cada cambio de un control pide una vista previa nueva con una generación
// mayor, tras un pequeño retardo. Rust cancela la que esté en marcha con una
// generación anterior, y aquí se descarta cualquier respuesta que no sea la
// última pedida: lo que se ve siempre corresponde a los controles.

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import * as puente from "../puente";
import { Comparador, type Modo } from "./Comparador";
import type { OpcionesWebp, Preset } from "./opciones";
import { Panel } from "./Panel";

const RETARDO_MS = 120;

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

  useEffect(() => {
    puente.presets().then(setGuardados, () => {});
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

  if (!info) {
    return (
      <section
        className={`zona${sobre ? " sobre" : ""}`}
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
        <p className="zona-titulo">{t("estudio.soltar")}</p>
        <p className="apagado">{t("estudio.soltarDetalle")}</p>
        <button className="principal" onClick={async () => {
          const f = await puente.elegirImagen();
          if (f) void abrir(f);
        }}>
          {t("estudio.abrir")}
        </button>
        {error && <p className="error" role="alert">{error}</p>}
      </section>
    );
  }

  const ahorro = vista ? 1 - vista.bytes / info.bytes : 0;
  const psnr = vista?.estadisticas.psnr[3];

  return (
    <div className="estudio">
      <div className="lienzo-y-barra">
        <div className="barra-superior">
          <span className="nombre-imagen" title={info.nombre}>{info.nombre}</span>
          <span className="apagado">
            {info.formato} · {info.ancho} × {info.alto}
            {info.alfa ? ` · ${t("estudio.conAlfa")}` : ""}
          </span>
          <span className="separador" />
          <div className="segmentado" role="radiogroup" aria-label={t("comparador.modo")}>
            {(["deslizador", "ladoALado"] as const).map((m) => (
              <button key={m} role="radio" aria-checked={modo === m} onClick={() => setModo(m)}>
                {t(`comparador.${m}`)}
              </button>
            ))}
          </div>
          <button onClick={async () => {
            const f = await puente.elegirImagen();
            if (f) void abrir(f);
          }}>
            {t("estudio.abrirOtra")}
          </button>
        </div>

        {info.orientacion !== 1 && !opciones.enderezar && (
          <div className="aviso" role="status" data-prueba="aviso-orientacion">
            {t("estudio.avisoOrientacion")}
            <button onClick={() => setOpciones({ ...opciones, enderezar: true })}>{t("estudio.enderezar")}</button>
          </div>
        )}

        <Comparador original={original} resultado={resultado} modo={modo} ocupado={ocupado} />

        <footer className="barra-estado" data-prueba="barra-estado">
          <div className="fila-estado">
          <div className="pesos">
            <span>{formatoBytes(info.bytes)}</span>
            <span className="flecha">→</span>
            <strong data-prueba="peso-resultado">{vista ? formatoBytes(vista.bytes) : "…"}</strong>
            {vista && (
              <span className={ahorro >= 0 ? "exito" : "error"} data-prueba="ahorro">
                {ahorro >= 0 ? "−" : "+"}
                {Math.abs(ahorro * 100).toLocaleString("es", { maximumFractionDigits: 1 })} %
              </span>
            )}
          </div>
          {vista && (
            <div className="datos apagado">
              {vista.ancho} × {vista.alto} · {vista.milisegundos} ms
              {psnr !== undefined && !opciones.sin_perdida && ` · PSNR ${psnr.toLocaleString("es", { maximumFractionDigits: 1 })} dB`}
            </div>
          )}
          <span className="separador" />
          <button className="principal" onClick={exportar} disabled={!vista} data-prueba="exportar">
            {t("estudio.exportar")}
          </button>
          </div>
          <OrdenCwebp vista={vista} cambiar={setOpciones} />
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

/** La orden cwebp equivalente: se copia, y se puede pegar otra para cargarla. */
function OrdenCwebp({ vista, cambiar }: { vista: puente.Vista | null; cambiar: (o: OpcionesWebp) => void }) {
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
      {vista && !vista.equivalente && (
        <span className="aviso-orden" title={t("orden.noEquivalenteAyuda")} data-prueba="no-equivalente">
          {t("orden.noEquivalente")}
        </span>
      )}
      <button
        onClick={async () => {
          if (!vista) return;
          await navigator.clipboard.writeText(vista.orden);
          setCopiado(true);
          window.setTimeout(() => setCopiado(false), 1500);
        }}
        disabled={!vista}
      >
        {copiado ? t("orden.copiada") : t("orden.copiar")}
      </button>
      <button onClick={() => { setTexto(vista?.orden ?? ""); setEditando(true); }}>{t("orden.editar")}</button>
    </div>
  );
}
