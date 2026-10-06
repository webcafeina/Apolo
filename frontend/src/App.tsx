import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Icono, Marca } from "./componentes";
import { Estudio } from "./estudio/Estudio";
import * as puente from "./puente";

type Seccion = "estudio" | "lotes" | "ajustes";

export function App() {
  const { t } = useTranslation();
  const [seccion, setSeccion] = useState<Seccion>("estudio");
  const [inicio, setInicio] = useState<puente.Inicio | null>(null);
  const [fallo, setFallo] = useState<string | null>(null);

  useEffect(() => {
    puente.inicio().then(setInicio, (e: puente.Fallo) => setFallo(e.mensaje));
  }, []);

  return (
    <div className="marco">
      <nav className="barra" aria-label={t("app.nombre")}>
        {/* El hueco de los semáforos en macOS, y por donde se arrastra la ventana. */}
        <div className="arrastre" data-tauri-drag-region />
        <div className="sello" data-tauri-drag-region>
          <Marca lado={22} />
          <span>{t("app.nombre")}</span>
        </div>
        {(["estudio", "lotes", "ajustes"] as const).map((s) => (
          <button key={s} className="fila" aria-current={seccion === s ? "page" : undefined} onClick={() => setSeccion(s)}>
            {t(`nav.${s}`)}
          </button>
        ))}
        <span className="separador" />
        {inicio && (
          <div className="firma" data-prueba="firma">
            {t("app.casa")} <span className="firma-barra" aria-hidden="true" /> {inicio.version}
          </div>
        )}
      </nav>
      <main className="contenido">
        {fallo && (
          <p className="error" role="alert">
            {t("app.sinServicio")} {fallo}
          </p>
        )}
        {/* El Estudio no se desmonta al cambiar de sección: conserva la imagen. */}
        <div hidden={seccion !== "estudio"} className="seccion">
          {inicio && <Estudio inicio={inicio} />}
        </div>
        {seccion === "lotes" && (
          <section className="zona">
            <p className="zona-titulo">{t("lotes.soltar")}</p>
            <p className="apagado nota">{t("lotes.pronto")}</p>
          </section>
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
    <section className="ajustes">
      <h1>{t("ajustes.presets")}</h1>
      <p className="apagado">{t("ajustes.presetsDonde")}</p>
      {inicio && <code className="ruta">{inicio.carpeta_presets}</code>}
      {guardados.length === 0 ? (
        <p className="apagado">{t("ajustes.sinPresets")}</p>
      ) : (
        <ul className="lista-presets">
          {guardados.map((g) => (
            <li key={g.nombre}>
              <span>{g.nombre}</span>
              <code className="apagado">apolo webp -apolo_preset "{g.nombre}"</code>
              <button onClick={async () => setGuardados(await puente.borrarPreset(g.nombre))}>{t("ajustes.borrar")}</button>
            </li>
          ))}
        </ul>
      )}

      <h1>{t("ajustes.acercaDe")}</h1>
      <div className="ficha">
        <Icono lado={72} />
        <div>
          <p className="ficha-nombre">{t("app.nombre")}</p>
          <p className="apagado">{inicio ? t("ajustes.version", { version: inicio.version }) : ""}</p>
          <p>{t("app.lema")}</p>
        </div>
      </div>
      <h2>{t("ajustes.motores")}</h2>
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
  );
}
