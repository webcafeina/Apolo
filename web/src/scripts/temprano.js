// Va en línea, lo primero de la cabeza, antes de pintar (ADR 0033): pone en
// <html> qué ordenador visita la web (data-plataforma, data-arquitectura) y lo
// que eligió la otra vez (data-vista, data-distro), para que el CSS acierte la
// descarga y las capturas desde el primer momento. Es una copia corta de
// detectar() (plataforma.ts), sin los valores de Chromium, que piden una
// promesa: los afina web.ts después. pruebas/temprano.test.ts comprueba que las
// dos dicen lo mismo.
(function () {
  var d = document.documentElement;
  var ua = navigator.userAgent;
  var u = navigator.userAgentData || {};
  var p = u.platform || "";
  var toques = navigator.maxTouchPoints || 0;
  var s;
  var a = "x64";
  if (u.mobile || /Android|iPhone|iPod/.test(ua) || (/Macintosh/.test(ua) && toques > 1)) s = "movil";
  else if (/CrOS/.test(ua) || p === "Chrome OS") s = "chromeos";
  else if (p === "macOS" || /Macintosh|Mac OS X/.test(ua)) s = "macos";
  else if (p === "Windows" || /Windows/.test(ua)) {
    s = "windows";
    if (/ARM64|aarch64/i.test(ua)) a = "arm64";
  } else if (p === "Linux" || /Linux|X11/.test(ua)) {
    s = "linux";
    if (/aarch64|arm64/i.test(ua)) a = "arm64";
  } else s = "otro";
  var leer = function (k) {
    try {
      return localStorage.getItem(k);
    } catch (e) {
      return null;
    }
  };
  d.setAttribute("data-plataforma", s);
  d.setAttribute("data-arquitectura", leer("apolo-arquitectura") || a);
  d.setAttribute("data-vista", leer("apolo-vista") || { macos: "macos", windows: "windows", linux: "gnome" }[s] || "macos");
  d.setAttribute("data-distro", leer("apolo-distro") || "ubuntu");
})();
