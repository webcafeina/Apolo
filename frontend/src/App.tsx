import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import * as actualizar from "./actualizar";
import { Agentes } from "./ajustes/Agentes";
import { BarraTitulo } from "./BarraTitulo";
import { Cabecera, esWindows, Icono, IconoApp, Marca, TarjetaAjuste } from "./componentes";
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
export function App() {
  const { t } = useTranslation();
  const [seccion, setSeccion] = useState<Seccion>("estudio");
  const [inicio, setInicio] = useState<puente.Inicio | null>(null);
  const [fallo, setFallo] = useState<string | null>(null);
  const [novedad, setNovedad] = useState<actualizar.Novedad | null>(null);
  const [traslocada, setTraslocada] = useState(false);
  const windows = esWindows();
  const { panel, alMenu, setEncima } = usePanel();

  useEffect(() => {
    puente.inicio().then(setInicio, (e: puente.Fallo) => setFallo(e.mensaje));
    puente.plataforma().then((p) => setTraslocada(p.traslocada), () => {});
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
      }}
    >
      <Icono nombre={s} lado={windows ? 16 : 18} />
      <span className="etiqueta">{t(`nav.${s}`)}</span>
    </button>
  );

  return (
    <div className="ventana" data-panel={windows ? panel : undefined}>
      {windows && <BarraTitulo panelAbierto={panel !== "compacto"} alMenu={alMenu} />}
      <aside className="lateral" data-tauri-drag-region>
        {/* En Windows, el nombre y el icono los pone la barra de título, y la
            versión está en Ajustes › Acerca de: el panel solo navega (ADR 0027). */}
        {!windows && (
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
        {inicio && !windows && (
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
        {esWindows() ? (
          // Como la Configuración de Windows 11: tarjetas en grupos, y «Acerca
          // de» al final (ADR 0027).
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

  if (esWindows()) {
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
  const casilla = (windows: boolean) => (
    <input
      type="checkbox"
      role={windows ? "switch" : undefined}
      aria-label={windows ? t("ajustes.avisarme") : undefined}
      checked={ajustes?.buscar_actualizaciones ?? true}
      disabled={ajustes === null}
      onChange={(e) => {
        // Se marca ya, sin esperar a Rust, que la guarda detrás.
        setAjustes({ ...ajustes!, buscar_actualizaciones: e.target.checked });
        puente.buscarActualizaciones(e.target.checked).then(setAjustes, () => {});
      }}
    />
  );
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

  if (esWindows()) {
    // En Windows, un interruptor (ToggleSwitch), como en la Configuración.
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
