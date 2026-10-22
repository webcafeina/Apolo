// Las descargas de la web (entrega 8d): pregunta a GitHub por la última versión, rellena la tabla
// con los ficheros de verdad y pone en el botón principal el de tu sistema. Sin esto (sin red, o si
// GitHub no contesta), todos los enlaces llevan a la página de la última versión, que ya están en
// el HTML. No se manda nada: es la misma petición que hace cualquiera que abre esa página.

const API = "https://api.github.com/repos/webcafeina/Apolo/releases/latest";

// Cada fila de la tabla, con su sistema y los ficheros que le tocan (por el final del nombre).
const FICHEROS = {
  "macos": { app: ["_universal.dmg"], cli: ["-macos-universal.tar.gz"], mcpb: ["-macos.mcpb"] },
  "windows-x64": { app: ["_x64-setup.exe"], cli: ["-windows-x64.zip"], mcpb: ["-windows-x64.mcpb"] },
  "windows-arm64": { app: ["_arm64-setup.exe"], cli: ["-windows-arm64.zip"], mcpb: ["-windows-arm64.mcpb"] },
  "linux-x64": { app: ["_amd64.deb", ".x86_64.rpm", "_amd64.AppImage"], cli: ["-linux-amd64.tar.gz"], mcpb: [] },
  "linux-arm64": { app: ["_arm64.deb", ".aarch64.rpm", "_aarch64.AppImage"], cli: ["-linux-arm64.tar.gz"], mcpb: [] },
};

// El sistema de quien visita. La arquitectura de Windows solo la dicen los navegadores Chromium
// (userAgentData); en los demás, x64, que es lo más común. En Linux no se puede saber: x64.
async function sistema() {
  const ua = navigator.userAgent;
  const datos = navigator.userAgentData;
  let plataforma = datos?.platform ?? "";
  if (!plataforma) {
    if (/Mac/.test(ua) && !/iPhone|iPad/.test(ua)) plataforma = "macOS";
    else if (/Windows/.test(ua)) plataforma = "Windows";
    else if (/Linux/.test(ua) && !/Android/.test(ua)) plataforma = "Linux";
  }
  if (plataforma === "macOS") return "macos";
  if (plataforma === "Windows") {
    try {
      const { architecture } = await datos.getHighEntropyValues(["architecture"]);
      if (architecture === "arm") return "windows-arm64";
    } catch {
      // Sin userAgentData: x64.
    }
    return "windows-x64";
  }
  if (plataforma === "Linux") return "linux-x64";
  return null;
}

function enlace(fichero) {
  const a = document.createElement("a");
  a.href = fichero.browser_download_url;
  a.textContent = fichero.name.endsWith(".AppImage") ? "AppImage" : fichero.name.split(".").pop().replace("gz", "tar.gz");
  a.title = fichero.name;
  return a;
}

async function rellenar() {
  const tuyo = await sistema();
  const boton = document.querySelector("[data-descarga-principal]");
  if (tuyo) document.querySelector(`tr[data-sistema="${tuyo}"]`)?.classList.add("tuyo");

  let version;
  try {
    const r = await fetch(API, { headers: { Accept: "application/vnd.github+json" } });
    if (!r.ok) return;
    version = await r.json();
  } catch {
    return;
  }
  const ficheros = version.assets ?? [];
  const busca = (finales) => finales.map((f) => ficheros.find((a) => a.name.endsWith(f))).filter(Boolean);

  for (const [id, partes] of Object.entries(FICHEROS)) {
    const fila = document.querySelector(`tr[data-sistema="${id}"]`);
    if (!fila) continue;
    for (const [columna, finales] of Object.entries(partes)) {
      const celda = fila.querySelector(`[data-columna="${columna}"]`);
      const encontrados = busca(finales);
      if (!celda || !encontrados.length) continue;
      celda.replaceChildren(...encontrados.map(enlace));
    }
  }

  document.querySelectorAll("[data-version]").forEach((e) => {
    e.textContent = version.tag_name?.replace(/^v/, "") ?? e.textContent;
  });

  // En Linux no se sabe la distribución: el botón lleva a los gestores de paquetes (ADR 0030).
  if (boton && tuyo?.startsWith("linux")) {
    boton.href = "#linux";
    if (boton.dataset.nombreLinuxX64) boton.querySelector("span").textContent = boton.dataset.nombreLinuxX64;
  } else if (boton && tuyo) {
    const [app] = busca(FICHEROS[tuyo].app);
    if (app) {
      boton.href = app.browser_download_url;
      const nombre = boton.dataset[`nombre${tuyo.replace(/(^|-)(\w)/g, (_, _g, c) => c.toUpperCase())}`];
      if (nombre) boton.querySelector("span").textContent = nombre;
    }
  }
}

rellenar();
