import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import * as actualizar from "./actualizar";
import { Cabecera, Icono, IconoApp, Marca } from "./componentes";
import { Estudio } from "./estudio/Estudio";
import { BandaNovedad } from "./Novedad";
import { Presets } from "./presets/Presets";
import * as puente from "./puente";

type Seccion = "estudio" | "lotes" | "presets" | "ajustes";

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
    <button key={s} aria-current={seccion === s ? "page" : undefined} onClick={() => setSeccion(s)}>
      <Icono nombre={s} lado={18} />
      {t(`nav.${s}`)}
    </button>
  );

  return (
    <div className="ventana">
      <aside className="lateral" data-tauri-drag-region>
        <div className="semaforos" data-tauri-drag-region />
        <div className="sello" data-tauri-drag-region>
          <Marca lado={26} />
          <span>{t("app.nombre")}</span>
        </div>
        <nav aria-label={t("app.nombre")}>{(["estudio", "lotes", "presets"] as const).map(fila)}</nav>
        <nav className="abajo" aria-label={t("nav.ajustes")}>
          {fila("ajustes")}
        </nav>
        {inicio && (
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
          {inicio && <Estudio inicio={inicio} />}
        </div>
        {seccion === "lotes" && (
          <div className="seccion columna">
            <Cabecera titulo={t("nav.lotes")} />
            <div className="contenido">
              <section className="zona-soltar">
                <Icono nombre="lotes" lado={40} />
                <p className="zona-titulo">{t("lotes.soltar")}</p>
                <p className="apagado">{t("lotes.pronto")}</p>
              </section>
            </div>
          </div>
        )}
        {seccion === "presets" && <Presets carpeta={inicio?.carpeta_presets ?? null} irAlEstudio={() => setSeccion("estudio")} />}
        {seccion === "ajustes" && <Ajustes inicio={inicio} alEncontrar={setNovedad} />}
      </main>
    </div>
  );
}

function Ajustes({
  inicio,
  alEncontrar,
}: {
  inicio: puente.Inicio | null;
  alEncontrar: (n: actualizar.Novedad) => void;
}) {
  const { t } = useTranslation();
  const [lista, setLista] = useState<puente.Motor[] | null>(null);

  useEffect(() => {
    puente.motores().then(setLista, () => setLista([]));
  }, []);

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.ajustes")} />
      <div className="contenido">
        <div className="panel-ajustes">
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
            <dl className="motores">
              {lista?.map((m) => (
                <div key={m.nombre}>
                  <dt>{m.nombre}</dt>
                  <dd>{m.version}</dd>
                </div>
              ))}
            </dl>
            <p className="apagado">{t("ajustes.licencia")}</p>
          </section>
          <Actualizaciones alEncontrar={alEncontrar} />
        </div>
      </div>
    </div>
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
    new Intl.DateTimeFormat("es", { dateStyle: "long", timeStyle: "short" }).format(new Date(segundos * 1000));

  return (
    <section className="grupo" data-prueba="actualizaciones">
      <h2>{t("ajustes.actualizaciones")}</h2>
      <label className="casilla">
        {/* Hasta que llegan los ajustes, la casilla no se toca: el clic no
            tendría de dónde partir y volvería sola a como estaba (Esfinge). */}
        <input
          type="checkbox"
          checked={ajustes?.buscar_actualizaciones ?? true}
          disabled={ajustes === null}
          onChange={(e) => {
            // Se marca ya, sin esperar a Rust, que la guarda detrás.
            setAjustes({ ...ajustes!, buscar_actualizaciones: e.target.checked });
            puente.buscarActualizaciones(e.target.checked).then(setAjustes, () => {});
          }}
        />
        {t("ajustes.avisarme")}
      </label>
      <p className="apagado">{t("ajustes.privacidad")}</p>
      {ajustes?.ultima_comprobacion != null && (
        <p className="apagado" data-prueba="ultima-vez">
          {t("ajustes.ultimaVez", { fecha: fecha(ajustes.ultima_comprobacion) })}
        </p>
      )}
      <div className="pareja">
        <button onClick={buscarAhora} disabled={buscando}>
          {buscando ? t("ajustes.buscando") : t("ajustes.buscarAhora")}
        </button>
      </div>
      {dicho && (
        <p className={dicho.error ? "error" : undefined} role="status">
          {dicho.texto}
        </p>
      )}
    </section>
  );
}
