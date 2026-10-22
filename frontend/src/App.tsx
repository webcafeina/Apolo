import { useEffect, useState, type ChangeEvent, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import * as actualizar from "./actualizar";
import { Agentes } from "./ajustes/Agentes";
import { BarraTitulo } from "./BarraTitulo";
import { BotonesVentana, ladosBotones } from "./BotonesVentana";
import { Cabecera, esLinux, esMac, esWindows, formularioNativo, Icono, IconoApp, Interruptor, Marca, TarjetaAjuste } from "./componentes";
import { Estudio } from "./estudio/Estudio";
import i18n, { lengua } from "./i18n";
import { Lotes } from "./lotes/Lotes";
import { BandaNovedad } from "./Novedad";
import { Presets } from "./presets/Presets";
import * as puente from "./puente";

// «mcp» no es una sección: es una puerta a Ajustes por el bloque de los
// agentes, como la fila «MCP» de Esfinge (lo pidió el cliente, ADR 0023).
type Seccion = "estudio" | "lotes" | "presets" | "mcp" | "ajustes";

// La ventana, con la anatomía de la de Esfinge: la barra lateral de 225 px con
// el hueco de los semáforos, el sello y las secciones (Ajustes abajo del todo),
// y a la derecha cada sección con su cabecera de 52 px y el título a la izquierda.
// En Windows, la de WinUI (ADR 0027); en macOS, la de macOS 27 (ADR 0028): la
// barra lateral de borde a borde, que se oculta con su botón, y la cabecera de
// cada sección hecha barra de herramientas unificada (sistema-macos.css).
export function App() {
  const { t } = useTranslation();
  const [seccion, setSeccion] = useState<Seccion>("estudio");
  const [inicio, setInicio] = useState<puente.Inicio | null>(null);
  const [fallo, setFallo] = useState<string | null>(null);
  const [novedad, setNovedad] = useState<actualizar.Novedad | null>(null);
  const [traslocada, setTraslocada] = useState(false);
  const [simulado, setSimulado] = useState(false);
  const [disposicion, setDisposicion] = useState(":minimize,maximize,close");
  const windows = esWindows();
  const mac = esMac();
  const linux = esLinux();
  const barraPropia = linux && document.documentElement.dataset.barra === "propia";
  const { panel, alMenu, setEncima } = usePanel();
  const { visible: lateralVisible, alternar: alternarLateral } = useLateral();
  const gnome = useLateralGnome();

  useEffect(() => {
    puente.inicio().then(setInicio, (e: puente.Fallo) => setFallo(e.mensaje));
    puente.plataforma().then(
      (p) => {
        setTraslocada(p.traslocada);
        setSimulado(!!p.simulado);
        if (p.botones) setDisposicion(p.botones);
      },
      () => {},
    );
  }, []);

  // Al abrir y luego cada hora, como Esfinge; la puerta de Rust deja preguntar
  // una vez al día. Un fallo de red aquí se calla: ya se mirará mañana, y
  // «Buscar ahora» sí lo enseña. «Ahora no» la quita hasta la próxima vez que
  // se abra Apolo: no se vuelve a buscar mientras haya una encontrada.
  useEffect(() => {
    if (!actualizar.sePuedeBuscar()) return;
    let encontrada = false;
    const mirar = () => {
      if (encontrada) return;
      actualizar.buscar(false).then(
        (n) => {
          if (n) {
            encontrada = true;
            setNovedad(n);
          }
        },
        () => {},
      );
    };
    mirar();
    const reloj = setInterval(mirar, actualizar.CADA_HORA);
    return () => clearInterval(reloj);
  }, []);

  const botones = ladosBotones(disposicion);
  const lateralAbierta = mac ? lateralVisible : gnome.estado !== "oculta";

  const fila = (s: Seccion) => (
    <button
      key={s}
      aria-current={seccion === s ? "page" : undefined}
      aria-label={t(`nav.${s}`)}
      // Plegado, solo queda el icono: el nombre sale al dejar el puntero.
      title={windows && panel === "compacto" ? t(`nav.${s}`) : undefined}
      onClick={() => {
        setSeccion(s);
        setEncima(false);
        gnome.cerrarEncima();
      }}
    >
      <Icono nombre={s} lado={mac ? 20 : windows || linux ? 16 : 18} />
      <span className="etiqueta">{t(`nav.${s}`)}</span>
    </button>
  );

  return (
    <div
      className="ventana"
      data-panel={windows ? panel : undefined}
      data-lateral={mac ? (lateralVisible ? "visible" : "oculta") : linux ? gnome.estado : undefined}
      style={linux ? ({ "--botones-izq": botones.izquierda.length, "--botones-der": botones.derecha.length } as CSSProperties) : undefined}
    >
      {windows && <BarraTitulo panelAbierto={panel !== "compacto"} alMenu={alMenu} />}
      {/* En el navegador no hay semáforos: unos dibujados ocupan su sitio en
          las capturas de macOS. */}
      {mac && simulado && (
        <div className="semaforos-simulados" aria-hidden="true">
          <i />
          <i />
          <i />
        </div>
      )}
      {/* En Linux con la barra del sistema, en el navegador se dibuja una que
          ocupa su sitio, para compararla en las capturas (ADR 0029). */}
      {linux && simulado && !barraPropia && (
        <div className="barra-sistema-simulada" aria-hidden="true">
          <span>{t("app.nombre")}</span>
          <BotonesVentana botones={["minimize", "maximize", "close"]} />
        </div>
      )}
      {barraPropia && botones.izquierda.length > 0 && (
        <div className="lado-botones izquierda">
          <BotonesVentana botones={botones.izquierda} />
        </div>
      )}
      {barraPropia && botones.derecha.length > 0 && (
        <div className="lado-botones derecha">
          <BotonesVentana botones={botones.derecha} />
        </div>
      )}
      {(mac || linux) && (
        <button
          className="boton-lateral"
          aria-label={t(lateralAbierta ? "ventana.ocultarLateral" : "ventana.mostrarLateral")}
          title={t(lateralAbierta ? "ventana.ocultarLateral" : "ventana.mostrarLateral")}
          aria-expanded={lateralAbierta}
          onClick={mac ? alternarLateral : gnome.alternar}
          data-prueba="boton-lateral"
        >
          <Icono nombre="lateral" lado={16} />
        </button>
      )}
      <aside className="lateral" data-tauri-drag-region inert={!lateralAbierta && (mac || linux) ? true : undefined}>
        {mac && <div className="cabeza-lateral" data-tauri-drag-region />}
        {/* En Linux, la cabecera de la barra lateral, como la de AdwNavigationSplitView. */}
        {linux && (
          <div className="cabeza-lateral" data-tauri-drag-region>
            <span className="titulo-lateral">{t("app.nombre")}</span>
          </div>
        )}
        {/* En Windows y en macOS, el nombre y el icono los pone la ventana, y la
            versión está en Ajustes › Acerca de: el panel solo navega (ADR 0027
            y 0028). */}
        {!windows && !mac && !linux && (
          <>
            <div className="semaforos" data-tauri-drag-region />
            <div className="sello" data-tauri-drag-region>
              <Marca lado={26} />
              <span>{t("app.nombre")}</span>
            </div>
          </>
        )}
        <nav aria-label={t("app.nombre")}>{(["estudio", "lotes", "presets"] as const).map(fila)}</nav>
        <nav className="abajo" aria-label={t("nav.ajustes")}>
          {fila("mcp")}
          {fila("ajustes")}
        </nav>
        {inicio && !windows && !mac && !linux && (
          <p className="firma" data-prueba="firma">
            {t("app.casa")}
            <span className="firma-barra" aria-hidden="true">▍</span>
            {inicio.version}
          </p>
        )}
      </aside>
      <main className="zona">
        {novedad && <BandaNovedad novedad={novedad} traslocada={traslocada} alCerrar={() => setNovedad(null)} />}
        {fallo && (
          <p className="error mensaje" role="alert">
            {t("app.sinServicio")} {fallo}
          </p>
        )}
        {/* El Estudio no se desmonta al cambiar de sección: conserva la imagen. */}
        <div hidden={seccion !== "estudio"} className="seccion">
          {inicio && <Estudio inicio={inicio} activo={seccion === "estudio"} />}
        </div>
        {/* Lotes tampoco: un lote largo sigue contando mientras se mira otra cosa. */}
        <div hidden={seccion !== "lotes"} className="seccion">
          {inicio && <Lotes inicio={inicio} activo={seccion === "lotes"} />}
        </div>
        {seccion === "presets" && <Presets carpeta={inicio?.carpeta_presets ?? null} irAlEstudio={() => setSeccion("estudio")} />}
        {(seccion === "ajustes" || seccion === "mcp") && (
          <Ajustes inicio={inicio} alEncontrar={setNovedad} porLosAgentes={seccion === "mcp"} />
        )}
      </main>
    </div>
  );
}

const CLAVE_LATERAL = "apolo.lateral";

/**
 * La barra lateral de macOS (ADR 0028), como la de Finder: su botón la oculta
 * entera o la muestra, con su animación, y se recuerda. Abre visible.
 */
function useLateral() {
  const [visible, setVisible] = useState(() => {
    try {
      return localStorage.getItem(CLAVE_LATERAL) !== "oculta";
    } catch {
      return true;
    }
  });
  const alternar = () =>
    setVisible((v) => {
      try {
        localStorage.setItem(CLAVE_LATERAL, v ? "oculta" : "visible");
      } catch {
        // Sin almacenamiento, se oculta igual; no se recordará.
      }
      return !v;
    });
  return { visible, alternar };
}

const ESTRECHA_GNOME = "(max-width: 859px)";
const CLAVE_LATERAL_GNOME = "apolo.lateral-gnome";

/**
 * La barra lateral de Linux (ADR 0029), como AdwNavigationSplitView: con la
 * ventana ancha, su botón la oculta o la muestra, y se recuerda; con la ventana
 * estrecha va oculta y el botón la saca encima del contenido hasta que se elige
 * una sección. Abre visible.
 */
function useLateralGnome() {
  const [estrecha, setEstrecha] = useState(() => window.matchMedia(ESTRECHA_GNOME).matches);
  const [oculta, setOculta] = useState(() => {
    try {
      return localStorage.getItem(CLAVE_LATERAL_GNOME) === "oculta";
    } catch {
      return false;
    }
  });
  const [encima, setEncima] = useState(false);

  useEffect(() => {
    const m = window.matchMedia(ESTRECHA_GNOME);
    const cambiar = () => {
      setEstrecha(m.matches);
      setEncima(false);
    };
    m.addEventListener("change", cambiar);
    return () => m.removeEventListener("change", cambiar);
  }, []);

  const estado: "visible" | "oculta" | "encima" = estrecha ? (encima ? "encima" : "oculta") : oculta ? "oculta" : "visible";
  const alternar = () => {
    if (estrecha) return setEncima((e) => !e);
    setOculta((o) => {
      try {
        localStorage.setItem(CLAVE_LATERAL_GNOME, o ? "visible" : "oculta");
      } catch {
        // Sin almacenamiento, se oculta igual; no se recordará.
      }
      return !o;
    });
  };
  return { estado, alternar, cerrarEncima: () => setEncima(false) };
}

const ANCHA = "(min-width: 1008px)";
const CLAVE_PANEL = "apolo.panel";

/**
 * El panel de navegación de Windows, como el NavigationView de WinUI en modo
 * automático: con la ventana ancha, ☰ lo pliega a solo iconos (48 px) o lo
 * despliega (320), y se recuerda; con la ventana estrecha va plegado, y ☰ lo
 * abre encima del contenido hasta que se elige una sección. La primera vez
 * abre plegado, para dejarle sitio a la imagen (lo eligió el cliente).
 */
function usePanel() {
  const [ancha, setAncha] = useState(() => window.matchMedia(ANCHA).matches);
  const [plegado, setPlegado] = useState(() => {
    try {
      return localStorage.getItem(CLAVE_PANEL) !== "abierto";
    } catch {
      return true;
    }
  });
  const [encima, setEncima] = useState(false);

  useEffect(() => {
    const m = window.matchMedia(ANCHA);
    const cambiar = () => {
      setAncha(m.matches);
      setEncima(false);
    };
    m.addEventListener("change", cambiar);
    return () => m.removeEventListener("change", cambiar);
  }, []);

  const panel: "abierto" | "compacto" | "encima" = ancha ? (plegado ? "compacto" : "abierto") : encima ? "encima" : "compacto";
  const alMenu = () => {
    if (!ancha) return setEncima((e) => !e);
    setPlegado((p) => {
      try {
        localStorage.setItem(CLAVE_PANEL, p ? "abierto" : "plegado");
      } catch {
        // Sin almacenamiento, se pliega igual; no se recordará.
      }
      return !p;
    });
  };
  return { panel, alMenu, setEncima };
}

function Ajustes({
  inicio,
  alEncontrar,
  porLosAgentes,
}: {
  inicio: puente.Inicio | null;
  alEncontrar: (n: actualizar.Novedad) => void;
  /** Se ha entrado por la fila «MCP»: se abre en la pestaña de los agentes. */
  porLosAgentes: boolean;
}) {
  const { t } = useTranslation();
  const [lista, setLista] = useState<puente.Motor[] | null>(null);
  const [pestana, setPestana] = useState<"general" | "agentes">(porLosAgentes ? "agentes" : "general");

  useEffect(() => {
    puente.motores().then(setLista, () => setLista([]));
  }, []);
  // Pasar de «Ajustes» a «MCP» (o al revés) con la pantalla abierta también
  // cambia de pestaña: no se vuelve a montar.
  useEffect(() => setPestana(porLosAgentes ? "agentes" : "general"), [porLosAgentes]);

  const motores = (
    <dl className="motores">
      {lista?.map((m) => (
        <div key={m.nombre}>
          <dt>{m.nombre}</dt>
          <dd>{m.version}</dd>
        </div>
      ))}
    </dl>
  );

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.ajustes")} />
      <div className="contenido">
        <div className="segmentado pestanas-ajustes" role="radiogroup" aria-label={t("ajustes.pestanas.etiqueta")}>
          {(["general", "agentes"] as const).map((p) => (
            <button key={p} role="radio" aria-checked={pestana === p} onClick={() => setPestana(p)}>
              {t(`ajustes.pestanas.${p}`)}
            </button>
          ))}
        </div>
        {pestana === "agentes" && <Agentes />}
        {formularioNativo() ? (
          // Como la Configuración de Windows 11 (tarjetas) o los Ajustes del
          // Sistema de macOS (filas agrupadas), y «Acerca de» al final (ADR 0027
          // y 0028).
          <div className="panel-ajustes" hidden={pestana !== "general"}>
            <ElegirIdioma />
            <Actualizaciones alEncontrar={alEncontrar} />
            <section className="grupo-ajustes">
              <h2>{t("ajustes.acercaDe")}</h2>
              <TarjetaAjuste
                icono={<IconoApp lado={32} />}
                titulo={t("app.nombre")}
                descripcion={inicio ? t("ajustes.version", { version: inicio.version }) : ""}
              >
                <p>{t("app.lema")}</p>
                {motores}
                <p className="apagado">{t("ajustes.licencia")}</p>
              </TarjetaAjuste>
            </section>
          </div>
        ) : (
          <div className="panel-ajustes" hidden={pestana !== "general"}>
            <ElegirIdioma />
            <section className="grupo">
              <h2>{t("ajustes.acercaDe")}</h2>
              <div className="ficha">
                <IconoApp lado={64} />
                <div>
                  <p className="ficha-nombre">{t("app.nombre")}</p>
                  <p className="apagado">{inicio ? t("ajustes.version", { version: inicio.version }) : ""}</p>
                  <p>{t("app.lema")}</p>
                </div>
              </div>
              {motores}
              <p className="apagado">{t("ajustes.licencia")}</p>
            </section>
            <Actualizaciones alEncontrar={alEncontrar} />
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * «Idioma»: el del sistema, español o inglés (ADR 0025). Se guarda en Rust,
 * que lo pone también en sus mensajes, y la interfaz cambia al que diga Rust:
 * con «Igual que el sistema», solo Rust sabe cuál es.
 */
function ElegirIdioma() {
  const { t } = useTranslation();
  const [preferencia, setPreferencia] = useState<puente.PreferenciaIdioma | null>(null);

  useEffect(() => {
    puente.ajustes().then((a) => setPreferencia(a.idioma), () => {});
  }, []);

  function elegir(p: puente.PreferenciaIdioma) {
    setPreferencia(p);
    puente.ponerIdioma(p).then((efectivo) => i18n.changeLanguage(efectivo), () => {});
  }

  // Cada idioma, con su nombre en su idioma: quien no entiende el actual tiene
  // que encontrar el suyo.
  const desplegable = (
    <select
          value={preferencia ?? "sistema"}
          disabled={preferencia === null}
          aria-label={t("ajustes.idioma")}
          onChange={(e) => elegir(e.target.value as puente.PreferenciaIdioma)}
        >
          <option value="sistema">{t("ajustes.idiomaSistema")}</option>
          <option value="es" lang="es">
            {t("ajustes.idiomas.es")}
          </option>
          <option value="en" lang="en">
            {t("ajustes.idiomas.en")}
          </option>
        </select>
  );

  if (formularioNativo()) {
    return (
      <TarjetaAjuste icono="idioma" titulo={t("ajustes.idioma")} descripcion={t("ajustes.idiomaDetalle")} control={desplegable} prueba="idioma" />
    );
  }
  return (
    <section className="grupo" data-prueba="idioma">
      <h2>{t("ajustes.idioma")}</h2>
      <div className="pareja">{desplegable}</div>
    </section>
  );
}

/** La casilla, la nota de privacidad y «Buscar ahora» (ADR 0018). */
function Actualizaciones({ alEncontrar }: { alEncontrar: (n: actualizar.Novedad) => void }) {
  const { t } = useTranslation();
  const [ajustes, setAjustes] = useState<puente.Ajustes | null>(null);
  const [buscando, setBuscando] = useState(false);
  const [dicho, setDicho] = useState<{ error: boolean; texto: string } | null>(null);

  useEffect(() => {
    puente.ajustes().then(setAjustes, () => {});
  }, []);

  async function buscarAhora() {
    setBuscando(true);
    setDicho(null);
    try {
      const n = await actualizar.buscar(true);
      if (n) {
        alEncontrar(n);
        setDicho({ error: false, texto: t("ajustes.hayNueva", { version: n.version }) });
      } else {
        setDicho({ error: false, texto: t("ajustes.alDia") });
      }
    } catch (e) {
      const detalle = e && typeof e === "object" && "mensaje" in e ? String(e.mensaje) : String(e);
      setDicho({ error: true, texto: t("ajustes.errorBuscar", { detalle }) });
    } finally {
      setBuscando(false);
      puente.ajustes().then(setAjustes, () => {});
    }
  }

  const fecha = (segundos: number) =>
    new Intl.DateTimeFormat(lengua(), { dateStyle: "long", timeStyle: "short" }).format(new Date(segundos * 1000));

  // Hasta que llegan los ajustes, la casilla no se toca: el clic no tendría de
  // dónde partir y volvería sola a como estaba (Esfinge).
  const casilla = (nativo: boolean) => {
    const comun = {
      checked: ajustes?.buscar_actualizaciones ?? true,
      disabled: ajustes === null,
      onChange: (e: ChangeEvent<HTMLInputElement>) => {
        // Se marca ya, sin esperar a Rust, que la guarda detrás.
        setAjustes({ ...ajustes!, buscar_actualizaciones: e.target.checked });
        puente.buscarActualizaciones(e.target.checked).then(setAjustes, () => {});
      },
    };
    return nativo ? <Interruptor aria-label={t("ajustes.avisarme")} {...comun} /> : <input type="checkbox" {...comun} />;
  };
  const ultimaVez = ajustes?.ultima_comprobacion != null && (
    <span className="apagado" data-prueba="ultima-vez">
      {t("ajustes.ultimaVez", { fecha: fecha(ajustes.ultima_comprobacion) })}
    </span>
  );
  const boton = (
    <button onClick={buscarAhora} disabled={buscando}>
      {buscando ? t("ajustes.buscando") : t("ajustes.buscarAhora")}
    </button>
  );
  const respuesta = dicho && (
    <span className={dicho.error ? "error" : undefined} role="status">
      {dicho.texto}
    </span>
  );

  if (formularioNativo()) {
    // En Windows y en macOS, un interruptor, como en sus ajustes.
    return (
      <section className="grupo-ajustes" data-prueba="actualizaciones">
        <h2>{t("ajustes.actualizaciones")}</h2>
        <TarjetaAjuste icono="avisos" titulo={t("ajustes.avisarme")} descripcion={t("ajustes.privacidad")} control={casilla(true)} />
        <TarjetaAjuste icono="actualizar" titulo={t("ajustes.comprobar")} descripcion={respuesta || ultimaVez} control={boton} />
      </section>
    );
  }
  return (
    <section className="grupo" data-prueba="actualizaciones">
      <h2>{t("ajustes.actualizaciones")}</h2>
      <label className="casilla">
        {casilla(false)}
        {t("ajustes.avisarme")}
      </label>
      <p className="apagado">{t("ajustes.privacidad")}</p>
      {ultimaVez && <p>{ultimaVez}</p>}
      <div className="pareja">{boton}</div>
      {respuesta && <p>{respuesta}</p>}
    </section>
  );
}
