// Ajustes › Agentes (ADR 0023): cómo se añade Apolo a un agente de IA por MCP.
//
// Como en Esfinge, se enseña y se copia; no se escribe en la configuración de
// nadie. Todo apunta al apolo-mcp que va junto a esta Apolo, que el
// actualizador reemplaza: por eso no hay que reinstalar nada al actualizar. El
// .mcpb de Claude Desktop lleva su copia, pero le cede el turno a este.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { esWindows, Icono, TarjetaAjuste } from "../componentes";
import * as puente from "../puente";

/** Un texto para copiar, con su botón. */
function ParaCopiar({ valor, prueba, bloque = false }: { valor: string; prueba: string; bloque?: boolean }) {
  const { t } = useTranslation();
  const [copiado, setCopiado] = useState(false);
  return (
    <div className={`para-copiar${bloque ? " bloque" : ""}`} data-prueba={prueba}>
      {bloque ? <pre>{valor}</pre> : <code>{valor}</code>}
      <button
        className="discreto"
        onClick={async () => {
          await navigator.clipboard.writeText(valor);
          setCopiado(true);
          setTimeout(() => setCopiado(false), 1500);
        }}
      >
        <Icono nombre="copiar" />
        {copiado ? t("agentes.copiado") : t("agentes.copiar")}
      </button>
    </div>
  );
}

export function Agentes() {
  const { t } = useTranslation();
  const [a, setA] = useState<puente.Agentes | null>(null);
  const [creando, setCreando] = useState(false);
  const [creada, setCreada] = useState<{ error: boolean; texto: string; ruta?: string } | null>(null);

  useEffect(() => {
    puente.agentes().then(setA, () => {});
  }, []);

  async function crear() {
    setCreando(true);
    setCreada(null);
    try {
      const ruta = await puente.crearMcpb();
      setCreada({ error: false, texto: t("agentes.creada", { ruta }), ruta });
    } catch (e) {
      setCreada({ error: true, texto: t("agentes.errorCrear", { detalle: (e as puente.Fallo).mensaje }) });
    } finally {
      setCreando(false);
    }
  }

  if (!a) return null;
  const aviso = {
    appimage: t("agentes.avisoAppimage"),
    traslocada: t("agentes.avisoTraslocada"),
    no_encontrado: t("agentes.avisoNoEncontrado"),
  };

  const windows = esWindows();
  const botones = (
    <>
      <button className="principal" onClick={crear} disabled={creando}>
        {creando ? t("agentes.creando") : t("agentes.crear")}
      </button>
      {creada?.ruta && puente.enTauri() && (
        <button onClick={() => puente.mostrarEnCarpeta(creada.ruta!)}>{t("agentes.mostrar")}</button>
      )}
    </>
  );
  const creadaTexto = creada && (
    <p className={creada.error ? "error" : "apagado"} role="status" data-prueba="mcpb-creada">
      {creada.texto}
    </p>
  );

  return (
    <div className="panel-ajustes" data-prueba="agentes">
      <section className="grupo">
        <h2 className="con-icono">
          <Icono nombre="mcp" lado={18} />
          {t("agentes.titulo")}
        </h2>
        <p>{t("agentes.subtitulo")}</p>
        <p data-prueba="version-conector">
          <strong>{t("agentes.version", { version: a.version })}</strong> {t("agentes.seActualiza")}
        </p>
        {/* Destacado: lo pidió el cliente. Claude Desktop sigue con el servidor
            viejo hasta reiniciarse, y es lo que más se olvida (ADR 0023). */}
        <div className="aviso reiniciar" role="note" data-prueba="aviso-reiniciar">
          <strong>{t("agentes.reiniciarTitulo")}</strong>
          <span>{t("agentes.reiniciarTexto")}</span>
        </div>
        {a.aviso && (
          <p className="error" role="alert" data-prueba="aviso-agentes">
            {aviso[a.aviso]}
          </p>
        )}
      </section>

      {a.ruta && (
        <>
          {windows ? (
            // En Windows, SettingsCards juntas: el botón a la derecha (ADR 0027).
            <div className="grupo-ajustes">
              <TarjetaAjuste icono="escritorio" titulo={t("agentes.desktop")} descripcion={t("agentes.desktopTexto")} control={botones} prueba="desktop">
                {creadaTexto}
              </TarjetaAjuste>
              <TarjetaAjuste icono="codigo" titulo={t("agentes.code")} descripcion={t("agentes.codeTexto")}>
                <ParaCopiar valor={a.claude_code!} prueba="orden-claude-code" />
              </TarjetaAjuste>
            </div>
          ) : (
            <>
              <section className="grupo" data-prueba="desktop">
                <h2>{t("agentes.desktop")}</h2>
                <p>{t("agentes.desktopTexto")}</p>
                <div className="pareja">{botones}</div>
                {creadaTexto}
              </section>

              <section className="grupo">
                <h2>{t("agentes.code")}</h2>
                <p>{t("agentes.codeTexto")}</p>
                <ParaCopiar valor={a.claude_code!} prueba="orden-claude-code" />
              </section>
            </>
          )}

          <section className="grupo">
            <details>
              <summary>{t("agentes.cursor")}</summary>
              <p>{t("agentes.cursorTexto")}</p>
              <ParaCopiar valor={a.json_mcp_servers!} prueba="json-mcp-servers" bloque />
            </details>
            <details>
              <summary>{t("agentes.vscode")}</summary>
              <p>{t("agentes.vscodeTexto")}</p>
              <ParaCopiar valor={a.json_vscode!} prueba="json-vscode" bloque />
            </details>
            <h2>{t("agentes.ruta")}</h2>
            <ParaCopiar valor={a.ruta} prueba="ruta-mcp" />
          </section>
        </>
      )}

      <section className="grupo">
        <h2>{t("agentes.queHace")}</h2>
        <p>{t("agentes.queHaceTexto")}</p>
        <p className="apagado">{t("agentes.terminal")}</p>
        <h2>{t("agentes.privacidad")}</h2>
        <p className="apagado">{t("agentes.privacidadTexto")}</p>
        <p className="apagado">{t("agentes.permiso")}</p>
      </section>
    </div>
  );
}
