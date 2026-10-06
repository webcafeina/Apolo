import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motores, type Motor } from "./puente";

type Vista = "estudio" | "lotes" | "ajustes";

export function App() {
  const { t } = useTranslation();
  const [vista, setVista] = useState<Vista>("estudio");

  return (
    <div className="marco">
      <nav className="barra" aria-label={t("app.nombre")}>
        <div className="marca">{t("app.nombre")}</div>
        {(["estudio", "lotes", "ajustes"] as const).map((v) => (
          <button
            key={v}
            className="fila"
            aria-current={vista === v ? "page" : undefined}
            onClick={() => setVista(v)}
          >
            {t(`nav.${v}`)}
          </button>
        ))}
      </nav>
      <main className="contenido">
        {vista === "estudio" && (
          <Zona titulo={t("estudio.soltar")} detalle={t("estudio.soltarDetalle")} nota={t("estudio.pronto")} />
        )}
        {vista === "lotes" && <Zona titulo={t("lotes.soltar")} nota={t("lotes.pronto")} />}
        {vista === "ajustes" && <Ajustes />}
      </main>
    </div>
  );
}

function Zona({ titulo, detalle, nota }: { titulo: string; detalle?: string; nota: string }) {
  return (
    <section className="zona">
      <p className="zona-titulo">{titulo}</p>
      {detalle && <p className="apagado">{detalle}</p>}
      <p className="apagado nota">{nota}</p>
    </section>
  );
}

function Ajustes() {
  const { t } = useTranslation();
  const [lista, setLista] = useState<Motor[] | null>(null);

  useEffect(() => {
    motores().then(setLista, () => setLista([]));
  }, []);

  return (
    <section className="ajustes">
      <h1>{t("ajustes.acercaDe")}</h1>
      <p>{t("app.lema")}</p>
      <h2>{t("ajustes.motores")}</h2>
      {lista && lista.length === 0 && <p className="apagado">{t("ajustes.sinMotores")}</p>}
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
