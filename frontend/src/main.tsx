import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./i18n";
import "./tokens.css";
import "./estilos.css";
import { App } from "./App";

createRoot(document.getElementById("raiz")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
