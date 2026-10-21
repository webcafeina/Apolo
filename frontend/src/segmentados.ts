// La selección que se desliza en los segmentados de macOS 27 (ADR 0028): al
// elegir otra opción, el fondo de la elegida se desliza hasta ella, en vez de
// aparecer de golpe. El CSS no sabe dónde está la opción elegida, así que aquí
// se mide (`--sel-x`, `--sel-y`, `--sel-w`, `--sel-h`, `--sel-r`) y
// sistema-macos.css dibuja el fondo con esas medidas. Solo en macOS; en los
// demás sistemas el fondo sigue en el botón.

function medir(s: HTMLElement) {
  const b = s.querySelector<HTMLElement>('[aria-checked="true"]');
  if (!b || !s.offsetParent) return;
  s.style.setProperty("--sel-x", `${b.offsetLeft}px`);
  s.style.setProperty("--sel-y", `${b.offsetTop}px`);
  s.style.setProperty("--sel-w", `${b.offsetWidth}px`);
  s.style.setProperty("--sel-h", `${b.offsetHeight}px`);
  s.style.setProperty("--sel-r", getComputedStyle(b).borderRadius);
  // La primera vez, sin deslizarse desde la izquierda: el fondo aparece en su
  // sitio, y desde la siguiente se anima.
  if (!s.dataset.medido) requestAnimationFrame(() => (s.dataset.medido = "si"));
}

export function seguirSegmentados(): void {
  const vistos = new WeakSet<HTMLElement>();
  // Cambia el ancho de un segmentado (la barra lateral se oculta, la ventana
  // se estrecha y las etiquetas se esconden): se vuelve a medir.
  const tamano = new ResizeObserver((e) => e.forEach((x) => medir(x.target as HTMLElement)));
  const todos = () =>
    document.querySelectorAll<HTMLElement>(".segmentado").forEach((s) => {
      if (!vistos.has(s)) {
        vistos.add(s);
        tamano.observe(s);
      }
      medir(s);
    });
  new MutationObserver(todos).observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["aria-checked", "hidden"],
  });
  todos();
}
