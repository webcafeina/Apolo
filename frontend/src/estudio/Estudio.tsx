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
import type { Ajuste, Preset } from "./opciones";
import { nombreFormato, Panel, type Lado } from "./Panel";

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

/** Un lado del comparador: su ajuste (null en el izquierdo: el original) y lo que dio. */
interface EstadoLado {
  ajuste: Ajuste | null;
  vista: puente.Vista | null;
  pixeles: ImageData | null;
  ocupado: boolean;
}

const ladoVacio = (ajuste: Ajuste | null): EstadoLado => ({ ajuste, vista: null, pixeles: null, ocupado: false });

export function Estudio({ inicio, activo }: { inicio: puente.Inicio; activo: boolean }) {
  const { t } = useTranslation();
  const [info, setInfo] = useState<puente.InfoImagen | null>(null);
  // El izquierdo empieza siendo el original; el derecho, WebP.
  const [lados, setLados] = useState<[EstadoLado, EstadoLado]>([ladoVacio(null), ladoVacio(inicio.ajuste)]);
  const [editando, setEditando] = useState<Lado>(1);
  const [original, setOriginal] = useState<ImageData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [aviso, setAviso] = useState<string | null>(null);
  const [modo, setModo] = useState<Modo>("deslizador");
  const [guardados, setGuardados] = useState<puente.PresetGuardado[]>([]);
  const [sobre, setSobre] = useState(false);
  const generaciones = useRef<[number, number]>([0, 0]);

  const ajusteDe = (l: Lado) => lados[l].ajuste;
  const editado = lados[editando].ajuste ?? lados[1].ajuste!;
  const cambiarLado = useCallback((l: Lado, cambio: Partial<EstadoLado>) => {
    setLados((x) => {
      const n: [EstadoLado, EstadoLado] = [x[0], x[1]];
      n[l] = { ...n[l], ...cambio };
      return n;
    });
  }, []);
  const ponerAjuste = useCallback((a: Ajuste) => cambiarLado(editando, { ajuste: a }), [cambiarLado, editando]);

  // La sección Presets avisa por eventos de ventana: cuando se aplica uno al
  // Estudio (al lado que se edita) y cuando cambia la lista.
  const ponerRef = useRef(ponerAjuste);
  ponerRef.current = ponerAjuste;
  useEffect(() => {
    const recargar = () => puente.presets().then(setGuardados, () => {});
    const aplicar = (e: Event) => ponerRef.current((e as CustomEvent<Ajuste>).detail);
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
        setLados((x) => [
          { ...x[0], vista: null, pixeles: null },
          { ...x[1], vista: null, pixeles: null },
        ]);
        setInfo(nueva);
      } catch (e) {
        setError((e as puente.Fallo).mensaje);
      }
    },
    [info],
  );

  // Soltar ficheros sobre la ventana de la aplicación. El Estudio no se
  // desmonta al cambiar de sección: solo los toma si es lo que se ve, y si
  // no, son de Lotes.
  const activoRef = useRef(activo);
  activoRef.current = activo;
  useEffect(() => {
    let quitar = () => {};
    void puente
      .alSoltar((rutas) => activoRef.current && rutas[0] && void abrir(rutas[0]))
      .then((f) => (quitar = f));
    return () => quitar();
  }, [abrir]);

  // Los píxeles del original: al abrir, y al enderezar o dejar de hacerlo
  // (se endereza si el lado derecho lo hace).
  const enderezado = lados[1].ajuste!.proceso.enderezar;
  useEffect(() => {
    if (!info) return;
    let vivo = true;
    puente.pixelesOriginal(info.id, enderezado).then(
      (p) => vivo && setOriginal(p),
      (e: puente.Fallo) => vivo && setError(e.mensaje),
    );
    return () => {
      vivo = false;
    };
  }, [info, enderezado]);

  // La vista previa en vivo de cada lado, cada una con su generación.
  const previsualizar = (l: Lado, ajuste: Ajuste | null) => {
    if (!info || !ajuste) return;
    const g = ++generaciones.current[l];
    cambiarLado(l, { ocupado: true });
    const reloj = window.setTimeout(async () => {
      try {
        const v = await puente.codificar(info.id, ajuste, l, g);
        if (g !== generaciones.current[l]) return;
        const px = await puente.pixelesResultado(info.id, l);
        if (g !== generaciones.current[l]) return;
        cambiarLado(l, { vista: v, pixeles: px, ocupado: false });
        setError(null);
      } catch (e) {
        const f = e as puente.Fallo;
        if (g !== generaciones.current[l] || f.cancelado) return;
        setError(f.mensaje);
        cambiarLado(l, { ocupado: false });
      }
    }, RETARDO_MS);
    return () => window.clearTimeout(reloj);
  };
  const ajusteIzq = ajusteDe(0);
  const ajusteDer = ajusteDe(1);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => previsualizar(0, ajusteIzq), [info, ajusteIzq]);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => previsualizar(1, ajusteDer), [info, ajusteDer]);

  const aplicarPreset = async (p: Preset) => ponerAjuste({ ...editado, webp: await puente.aplicarPreset(editado.webp, p) });
  const nivelSinPerdida = async (n: number) =>
    ponerAjuste({ ...editado, webp: await puente.nivelSinPerdida(editado.webp, n) });
  const guardar = async (nombre: string) => {
    try {
      setGuardados(await puente.guardarPreset({ nombre, ...editado }));
      window.dispatchEvent(new Event(EVENTO_PRESETS));
      setAviso(t("estudio.presetGuardado", { nombre }));
    } catch (e) {
      setError((e as puente.Fallo).mensaje);
    }
  };

  const ladoActivo: Lado = lados[editando].ajuste ? editando : 1;
  const vista = lados[ladoActivo].vista;

  const exportar = async () => {
    if (!info) return;
    try {
      const r = await puente.exportar(info.id, lados[ladoActivo].ajuste!, ladoActivo);
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

  const psnr = vista?.estadisticas?.psnr[3];
  const sinPerdida = vista?.formato === "webp" && lados[ladoActivo].ajuste!.webp.sin_perdida;
  const rotulo = (l: Lado) => {
    const v = lados[l].vista;
    if (!lados[l].ajuste) return t("comparador.original");
    return v ? `${nombreFormato(v.formato)} · ${formatoBytes(v.bytes)}` : nombreFormato(lados[l].ajuste!.formato);
  };

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

        {info.orientacion !== 1 && !enderezado && (
          <div className="aviso" role="status" data-prueba="aviso-orientacion">
            {t("estudio.avisoOrientacion")}
            <button
              onClick={() => {
                // Se enderezan los dos lados: si no, no se podrían comparar.
                for (const l of [0, 1] as const) {
                  const a = lados[l].ajuste;
                  if (a) cambiarLado(l, { ajuste: { ...a, proceso: { ...a.proceso, enderezar: true } } });
                }
              }}
            >
              {t("estudio.enderezar")}
            </button>
          </div>
        )}

        <Comparador
          izquierda={lados[0].ajuste ? lados[0].pixeles : original}
          derecha={lados[1].pixeles}
          rotulos={[rotulo(0), rotulo(1)]}
          ocupados={[lados[0].ocupado, lados[1].ocupado]}
          modo={modo}
        />

        <footer className="barra-estado" data-prueba="barra-estado">
          <div className="fila-estado">
            <Pesos info={info} vista={vista} />
            <span className="separador" />
            {vista && (
              <div className="datos apagado">
                {vista.ancho} × {vista.alto} · {vista.milisegundos} ms
                {psnr !== undefined && !sinPerdida && ` · PSNR ${psnr.toLocaleString("es", { maximumFractionDigits: 1 })} dB`}
              </div>
            )}
            <button className="principal" onClick={exportar} disabled={!vista} data-prueba="exportar">
              <Icono nombre="exportar" />
              {t("estudio.exportar")}
            </button>
          </div>
          <OrdenHerramienta vista={vista} formato={info.formato} ajuste={lados[ladoActivo].ajuste!} cambiar={ponerAjuste} />
          {vista?.motivo && (
            <p className="motivo apagado" data-prueba="motivo">
              {t(`orden.motivo.${vista.motivo}`, { herramienta: vista.herramienta, formato: info.formato })}
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
        ajuste={editado}
        contexto={{ alfa: info.alfa, ancho: info.ancho, alto: info.alto }}
        presetsCwebp={inicio.presets_cwebp}
        guardados={guardados}
        cambiar={ponerAjuste}
        aplicarPreset={aplicarPreset}
        nivelSinPerdida={nivelSinPerdida}
        guardar={guardar}
        lado={editando}
        elegirLado={setEditando}
        izquierdaOriginal={lados[0].ajuste === null}
        formatoIzquierda={lados[0].ajuste?.formato ?? null}
        formatoDerecha={lados[1].ajuste!.formato}
        usarFormatoIzquierda={(si) =>
          cambiarLado(0, si ? { ajuste: { ...lados[1].ajuste!, formato: lados[1].ajuste!.formato === "webp" ? "jpeg" : "webp" } } : ladoVacio(null))
        }
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
        <span className="etiqueta">{t("pesos.resultado", { formato: vista ? nombreFormato(vista.formato) : "…" })}</span>
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

/** La orden de la herramienta: se copia, y se puede pegar otra para cargarla. */
function OrdenHerramienta({
  vista,
  formato,
  ajuste,
  cambiar,
}: {
  vista: puente.Vista | null;
  formato: string;
  ajuste: Ajuste;
  cambiar: (a: Ajuste) => void;
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
            cambiar(await puente.leerOrden(texto, ajuste));
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
      <code title={vista?.orden} data-prueba="orden">{vista?.orden ?? "…"}</code>
      {vista && vista.motivo && (
        <span className="aviso-orden" title={t(`orden.motivo.${vista.motivo}`, { herramienta: vista.herramienta, formato })} data-prueba="no-equivalente">
          {/* Una frase corta según el caso; la explicación larga va debajo. */}
          {t(`orden.aviso.${vista.motivo}`, { formato, herramienta: vista.herramienta })}
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
