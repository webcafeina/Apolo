import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cabecera, Icono, IconoApp, Marca } from "./componentes";
import { Estudio } from "./estudio/Estudio";
import * as puente from "./puente";

type Seccion = "estudio" | "lotes" | "ajustes";

// La ventana, con la anatomía de la de Esfinge: la barra lateral de 225 px con
// el hueco de los semáforos, el sello y las secciones (Ajustes abajo del todo),
// y a la derecha cada sección con su cabecera de 52 px y el título a la izquierda.
export function App() {
  const { t } = useTranslation();
  const [seccion, setSeccion] = useState<Seccion>("estudio");
  const [inicio, setInicio] = useState<puente.Inicio | null>(null);
  const [fallo, setFallo] = useState<string | null>(null);

  useEffect(() => {
    puente.inicio().then(setInicio, (e: puente.Fallo) => setFallo(e.mensaje));
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
        <nav aria-label={t("app.nombre")}>{(["estudio", "lotes"] as const).map(fila)}</nav>
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
        {seccion === "ajustes" && <Ajustes inicio={inicio} />}
      </main>
    </div>
  );
}

function Ajustes({ inicio }: { inicio: puente.Inicio | null }) {
  const { t } = useTranslation();
  const [lista, setLista] = useState<puente.Motor[] | null>(null);
  const [guardados, setGuardados] = useState<puente.PresetGuardado[]>([]);

  useEffect(() => {
    puente.motores().then(setLista, () => setLista([]));
    puente.presets().then(setGuardados, () => {});
  }, []);

  return (
    <div className="seccion columna">
      <Cabecera titulo={t("nav.ajustes")} />
      <div className="contenido">
        <div className="panel-ajustes">
          <section className="grupo">
            <h2>{t("ajustes.presets")}</h2>
            <p className="apagado">{t("ajustes.presetsDonde")}</p>
            {inicio && <code className="ruta seleccionable">{inicio.carpeta_presets}</code>}
            {guardados.length === 0 ? (
              <p className="apagado">{t("ajustes.sinPresets")}</p>
            ) : (
              <ul className="lista-presets">
                {guardados.map((g) => (
                  <li key={g.nombre}>
                    <span>{g.nombre}</span>
                    <code className="apagado seleccionable">apolo webp -apolo_preset "{g.nombre}"</code>
                    <button onClick={async () => setGuardados(await puente.borrarPreset(g.nombre))}>{t("ajustes.borrar")}</button>
                  </li>
                ))}
              </ul>
            )}
          </section>

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
        </div>
      </div>
    </div>
  );
}
