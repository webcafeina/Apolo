import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import i18n from "./i18n";
import "./tokens.css";
import "./estilos.css";
import { App } from "./App";
import { inicio, plataforma } from "./puente";

// El CSS del vidrio y de cada sistema cuelga de estos dos atributos (ADR 0016).
// Se ponen antes de pintar para que no haya un parpadeo opaco. El idioma
// (ADR 0025), también: el de los ajustes, o el del sistema, lo dice Rust. Si
// Rust no contesta, español, y App enseña el fallo.
void Promise.allSettled([
  plataforma().then((p) => {
    document.documentElement.dataset.sistema = p.sistema;
    document.documentElement.dataset.vidrio = p.vidrio ? "si" : "no";
  }),
  inicio().then((i) => i18n.changeLanguage(i.idioma)),
]).finally(() =>
  createRoot(document.getElementById("raiz")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  ),
);
