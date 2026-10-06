import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./i18n";
import "./tokens.css";
import "./estilos.css";
import { App } from "./App";
import { plataforma } from "./puente";

// El CSS del vidrio y de cada sistema cuelga de estos dos atributos (ADR 0016).
// Se ponen antes de pintar para que no haya un parpadeo opaco.
void plataforma()
  .then((p) => {
    document.documentElement.dataset.sistema = p.sistema;
    document.documentElement.dataset.vidrio = p.vidrio ? "si" : "no";
  })
  .catch(() => {})
  .finally(() =>
    createRoot(document.getElementById("raiz")!).render(
      <StrictMode>
        <App />
      </StrictMode>,
    ),
  );
