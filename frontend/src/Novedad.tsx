import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Avance, Novedad } from "./actualizar";

type Paso = { es: "aviso" } | { es: "bajando"; avance: Avance } | { es: "lista" } | { es: "instalando" };

const texto = (e: unknown) => (e && typeof e === "object" && "message" in e ? String(e.message) : String(e));

/**
 * La banda de versión nueva (ADR 0018), la de Esfinge: va encima de la sección,
 * fuera del trabajo, y cuenta en el mismo sitio los momentos de la novedad
 * —hay una, se está bajando, está lista, se está instalando— o el error.
 */
export function BandaNovedad({
  novedad,
  traslocada,
  alCerrar,
}: {
  novedad: Novedad;
  traslocada: boolean;
  alCerrar: () => void;
}) {
  const { t } = useTranslation();
  const [paso, setPaso] = useState<Paso>({ es: "aviso" });
  const [error, setError] = useState<string | null>(null);
  const version = novedad.version;

  async function descargar() {
    setError(null);
    setPaso({ es: "bajando", avance: { bytes: 0, total: 0 } });
    try {
      await novedad.descargar((avance) => setPaso({ es: "bajando", avance }));
      setPaso({ es: "lista" });
    } catch (e) {
      setError(t("novedad.errorDescarga", { detalle: texto(e) }));
      setPaso({ es: "aviso" });
    }
  }

  async function instalar() {
    setError(null);
    setPaso({ es: "instalando" });
    try {
      await novedad.instalarYReiniciar();
    } catch (e) {
      setError(t("novedad.errorInstalar", { detalle: texto(e) }));
      setPaso({ es: "lista" });
    }
  }

  const porcentaje = (a: Avance) => (a.total > 0 ? Math.min(100, Math.round((a.bytes / a.total) * 100)) : null);

  return (
    <div className="novedad" role="status" data-prueba="novedad">
      <div className="dice">
        {error ? (
          <span className="error">{error}</span>
        ) : paso.es === "instalando" ? (
          <span>{t("novedad.instalando", { version })}</span>
        ) : paso.es === "lista" ? (
          <span>{t("novedad.lista", { version })}</span>
        ) : paso.es === "bajando" ? (
          <span>
            {porcentaje(paso.avance) === null
              ? t("novedad.descargando", { version })
              : t("novedad.descargandoPorcentaje", { version, porcentaje: porcentaje(paso.avance) })}
          </span>
        ) : (
          <span>
            {t("novedad.hay")} <strong>{t("novedad.nombre", { version })}</strong>
            {traslocada && <span className="apagado"> · {t("novedad.traslocada")}</span>}
          </span>
        )}
        {paso.es === "bajando" && (
          <div className="barra-progreso">
            <i style={{ width: `${porcentaje(paso.avance) ?? 100}%` }} />
          </div>
        )}
      </div>
      <div className="aparte">
        {paso.es === "lista" && (
          <button className="principal" onClick={instalar}>
            {t("novedad.instalar")}
          </button>
        )}
        {paso.es === "aviso" && (
          <>
            {!traslocada && (
              <button className="principal" onClick={descargar}>
                {t("novedad.descargar")}
              </button>
            )}
            <button onClick={alCerrar}>{t("novedad.ahoraNo")}</button>
          </>
        )}
      </div>
    </div>
  );
}
