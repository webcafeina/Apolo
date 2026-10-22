import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import i18n from "./i18n";
import "./tokens.css";
import "./estilos.css";
import "./sistema-windows.css";
import "./sistema-macos.css";
import "./sistema-linux.css";
import { App } from "./App";
import { alCambiarEsquinas, inicio, plataforma } from "./puente";
import { seguirSegmentados } from "./segmentados";
import vidrioMacos from "./vidrio-macos.css?inline";

// El CSS del vidrio y de cada sistema cuelga de estos dos atributos (ADR 0016).
// Se ponen antes de pintar para que no haya un parpadeo opaco. El idioma
// (ADR 0025), también: el de los ajustes, o el del sistema, lo dice Rust. Si
// Rust no contesta, español, y App enseña el fallo.
void Promise.allSettled([
  plataforma().then((p) => {
    document.documentElement.dataset.sistema = p.sistema;
    document.documentElement.dataset.vidrio = p.vidrio ? "si" : "no";
    // En el navegador no hay vidrio de verdad: un degradado lo imita para las capturas.
    if (p.simulado && p.vidrio) document.documentElement.dataset.vidrioSimulado = "si";
    // En las capturas de Linux con la barra del sistema, una dibujada ocupa su sitio arriba.
    if (p.simulado) document.documentElement.dataset.simulado = "si";
    // En Linux, si la parte de arriba es la barra de Apolo o la del sistema (ADR 0029).
    if (p.sistema === "linux") document.documentElement.dataset.barra = p.barra_propia ? "propia" : "sistema";
    // Y si la ventana tiene el marco de GTK con sus esquinas redondeadas: la interfaz redondea las
    // suyas, y las cuadra al maximizarla o ponerla en mosaico.
    if (p.sistema === "linux" && p.esquinas !== undefined) {
      const poner = (redondas: boolean) => (document.documentElement.dataset.esquinas = redondas ? "redondas" : "rectas");
      poner(p.esquinas);
      void alCambiarEsquinas(poner);
    }
    // En macOS, la selección de los segmentados se desliza (ADR 0028).
    if (p.sistema === "macos") seguirSegmentados();
    // El vidrio del sistema en los controles, si Rust ha podido encenderlo:
    // la hoja se inyecta ahora, después, para que WebKit no descarte la
    // propiedad (vidrio-macos.css).
    if (p.vidrio_controles) {
      const hoja = document.createElement("style");
      hoja.textContent = vidrioMacos;
      document.head.appendChild(hoja);
      document.documentElement.dataset.vidrioControles = "si";
    }
  }),
  inicio().then((i) => i18n.changeLanguage(i.idioma)),
]).finally(() =>
  createRoot(document.getElementById("raiz")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  ),
);
