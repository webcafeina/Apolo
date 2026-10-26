// Las páginas de descarga por sistema (ADR 0033), en los dos idiomas. Cada una:
// su cabeza, su tarjeta de descarga (Descargas.astro) y estas secciones.
import type { Idioma } from "../i18n/rutas";

export interface Seccion {
  titulo: string;
  parrafos?: string[];
  pasos?: string[];
  lista?: string[];
  comandos?: { nombre: string; lineas: string[]; titulo?: string }[];
}
export interface PaginaDescarga {
  titulo: string;
  descripcion: string;
  etiqueta: string;
  h1: string;
  entrada: string;
  secciones: Seccion[];
}
export type Sistema = "macos" | "windows" | "linux";

const ACTUALIZA = {
  es: "Cuando hay una versión nueva, Apolo lo avisa y se instala con un clic. Para saberlo pregunta a GitHub una vez al día, y nada más: se puede apagar en Ajustes.",
  en: "When there's a new version, Apolo tells you and installs it with one click. To know, it asks GitHub once a day, and nothing else: you can turn it off in Settings.",
};

export const DESCARGAR: Record<Idioma, Record<Sistema, PaginaDescarga>> = {
  es: {
    macos: {
      titulo: "Descargar Apolo para Mac · Optimizador de imágenes gratuito",
      descripcion:
        "Apolo para macOS 11 o posterior, en Apple Silicon e Intel: optimiza imágenes a WebP, AVIF, JPEG XL, JPEG y PNG viendo el resultado. Gratis y de código abierto.",
      etiqueta: "Descargar · macOS",
      h1: "Apolo para Mac.",
      entrada: "Una sola descarga para Apple Silicon e Intel, con el aspecto de macOS y el vidrio del sistema.",
      secciones: [
        {
          titulo: "Instalar",
          pasos: [
            "Abre el .dmg y arrastra Apolo a la carpeta Aplicaciones.",
            "La primera vez, macOS avisa de que Apolo no está firmada. Ve a Ajustes del Sistema › Privacidad y seguridad y pulsa «Abrir igualmente».",
            "Si dice que está dañada, abre Terminal y escribe esto:",
          ],
          comandos: [{ nombre: "xattr", lineas: ["xattr -dr com.apple.quarantine /Applications/Apolo.app"] }],
        },
        {
          titulo: "Con Homebrew",
          parrafos: ["La aplicación, y aparte la línea de comandos con el servidor para agentes de IA:"],
          comandos: [
            { nombre: "brew-cask", lineas: ["brew install --cask webcafeina/apolo/apolo"] },
            { nombre: "brew", lineas: ["brew install webcafeina/apolo/apolo"] },
          ],
        },
        { titulo: "Se actualiza sola", parrafos: [ACTUALIZA.es] },
        {
          titulo: "Requisitos",
          lista: [
            "macOS 11 Big Sur o posterior.",
            "Un Mac con Apple Silicon o con Intel: el mismo .dmg vale para los dos.",
            "Con fotos de 24 megapíxeles no pasa de unos 2 GB de memoria, ni en lotes de cientos: cabe en un Mac de 8 GB.",
          ],
        },
      ],
    },
    windows: {
      titulo: "Descargar Apolo para Windows · Optimizador de imágenes gratuito",
      descripcion:
        "Apolo para Windows 10 y 11, en x64 y ARM: optimiza imágenes a WebP, AVIF, JPEG XL, JPEG y PNG viendo el resultado. Gratis y de código abierto.",
      etiqueta: "Descargar · Windows",
      h1: "Apolo para Windows.",
      entrada: "Para Windows 10 y 11, en x64 y en ARM, con el aspecto de Windows 11 y Mica.",
      secciones: [
        {
          titulo: "Instalar",
          pasos: [
            "Abre el instalador: Apolo se instala para tu usuario, sin pedir permisos de administrador.",
            "Si sale SmartScreen, pulsa «Más información» y después «Ejecutar de todos modos». Avisa porque Apolo no está firmada.",
          ],
        },
        {
          titulo: "¿x64 o ARM?",
          parrafos: [
            "Si tu ordenador lleva un procesador Snapdragon u otro ARM, elige ARM. Con Intel o AMD, x64. Edge y Chrome lo saben y lo eligen solos; Firefox no lo dice.",
          ],
        },
        { titulo: "Se actualiza sola", parrafos: [ACTUALIZA.es] },
        {
          titulo: "Requisitos",
          lista: ["Windows 10 o 11, de 64 bits.", "Un procesador x64 (Intel o AMD) o ARM.", "Nada más: no hace falta instalar Visual C++ ni otras bibliotecas."],
        },
      ],
    },
    linux: {
      titulo: "Descargar Apolo para Linux · Optimizador de imágenes gratuito",
      descripcion:
        "Apolo para Ubuntu, Debian, Fedora, openSUSE, Arch y más, en x64 y ARM: .deb, .rpm, AppImage y repositorios. Gratis y de código abierto.",
      etiqueta: "Descargar · Linux",
      h1: "Apolo para Linux.",
      entrada: "En .deb, .rpm, AppImage y un PKGBUILD para Arch, en x64 y ARM, con el aspecto de GNOME o de KDE.",
      secciones: [
        {
          titulo: "Ubuntu y Debian",
          parrafos: ["Con el repositorio de Apolo, se actualiza con el resto del sistema:"],
          comandos: [
            {
              nombre: "apt",
              lineas: [
                "sudo install -dm755 /etc/apt/keyrings",
                "sudo curl -fsSLo /etc/apt/keyrings/apolo.asc https://apolo.webcafeina.com/apolo.asc",
                "sudo curl -fsSLo /etc/apt/sources.list.d/apolo.sources https://apolo.webcafeina.com/apt/apolo.sources",
                "sudo apt update && sudo apt install apolo",
              ],
            },
          ],
        },
        {
          titulo: "Fedora y openSUSE",
          comandos: [
            { titulo: "Fedora", nombre: "dnf", lineas: ["sudo curl -fsSLo /etc/yum.repos.d/apolo.repo https://apolo.webcafeina.com/rpm/apolo.repo", "sudo dnf install apolo"] },
            { titulo: "openSUSE", nombre: "zypper", lineas: ["sudo zypper addrepo https://apolo.webcafeina.com/rpm/apolo.repo", "sudo zypper install apolo"] },
          ],
        },
        {
          titulo: "Arch, Manjaro, EndeavourOS y CachyOS",
          parrafos: ["Con el PKGBUILD de esta web. Pronto estará en el AUR, como apolo-bin."],
          comandos: [{ nombre: "makepkg", lineas: ["mkdir apolo && cd apolo", "curl -fsSLO https://apolo.webcafeina.com/aur/PKGBUILD", "makepkg -si"] }],
        },
        {
          titulo: "Cualquier otra",
          parrafos: ["La AppImage funciona en casi todas: dale permiso para ejecutarse y ábrela."],
        },
        {
          titulo: "Actualizar",
          lista: [
            "Con el repositorio, con el resto del sistema (apt, dnf o zypper).",
            "Con el .deb o el .rpm sueltos, Apolo avisa de cada versión nueva y se instala con un clic.",
            "Con Arch, volviendo a construir el PKGBUILD.",
          ],
        },
        {
          titulo: "Requisitos",
          lista: [
            "Linux de 64 bits, en x64 o ARM.",
            "WebKitGTK 4.1, que el gestor de paquetes instala solo con el .deb y el .rpm.",
            "Se ve como GNOME o como KDE, según tu escritorio.",
          ],
        },
      ],
    },
  },
  en: {
    macos: {
      titulo: "Download Apolo for Mac · Free image optimizer",
      descripcion:
        "Apolo for macOS 11 or later, on Apple silicon and Intel: optimize images to WebP, AVIF, JPEG XL, JPEG and PNG while seeing the result. Free and open source.",
      etiqueta: "Download · macOS",
      h1: "Apolo for Mac.",
      entrada: "One download for Apple silicon and Intel, with the macOS look and the system's glass.",
      secciones: [
        {
          titulo: "Install",
          pasos: [
            "Open the .dmg and drag Apolo to the Applications folder.",
            "The first time, macOS warns you that Apolo isn't signed. Go to System Settings › Privacy & Security and click “Open Anyway”.",
            "If it says it's damaged, open Terminal and type this:",
          ],
          comandos: [{ nombre: "xattr", lineas: ["xattr -dr com.apple.quarantine /Applications/Apolo.app"] }],
        },
        {
          titulo: "With Homebrew",
          parrafos: ["The app, and separately the command line with the server for AI agents:"],
          comandos: [
            { nombre: "brew-cask", lineas: ["brew install --cask webcafeina/apolo/apolo"] },
            { nombre: "brew", lineas: ["brew install webcafeina/apolo/apolo"] },
          ],
        },
        { titulo: "It updates itself", parrafos: [ACTUALIZA.en] },
        {
          titulo: "Requirements",
          lista: [
            "macOS 11 Big Sur or later.",
            "A Mac with Apple silicon or Intel: the same .dmg works on both.",
            "With 24-megapixel photos it stays around 2 GB of memory, even in batches of hundreds: it fits an 8 GB Mac.",
          ],
        },
      ],
    },
    windows: {
      titulo: "Download Apolo for Windows · Free image optimizer",
      descripcion:
        "Apolo for Windows 10 and 11, on x64 and ARM: optimize images to WebP, AVIF, JPEG XL, JPEG and PNG while seeing the result. Free and open source.",
      etiqueta: "Download · Windows",
      h1: "Apolo for Windows.",
      entrada: "For Windows 10 and 11, on x64 and ARM, with the Windows 11 look and Mica.",
      secciones: [
        {
          titulo: "Install",
          pasos: [
            "Open the installer: Apolo installs for your user, without asking for administrator rights.",
            "If SmartScreen appears, click “More info” and then “Run anyway”. It warns you because Apolo isn't signed.",
          ],
        },
        {
          titulo: "x64 or ARM?",
          parrafos: [
            "If your computer has a Snapdragon or another ARM processor, choose ARM. With Intel or AMD, x64. Edge and Chrome know and choose for you; Firefox doesn't say.",
          ],
        },
        { titulo: "It updates itself", parrafos: [ACTUALIZA.en] },
        {
          titulo: "Requirements",
          lista: ["Windows 10 or 11, 64-bit.", "An x64 (Intel or AMD) or ARM processor.", "Nothing else: no Visual C++ or other libraries to install."],
        },
      ],
    },
    linux: {
      titulo: "Download Apolo for Linux · Free image optimizer",
      descripcion:
        "Apolo for Ubuntu, Debian, Fedora, openSUSE, Arch and more, on x64 and ARM: .deb, .rpm, AppImage and repositories. Free and open source.",
      etiqueta: "Download · Linux",
      h1: "Apolo for Linux.",
      entrada: "As .deb, .rpm, AppImage and a PKGBUILD for Arch, on x64 and ARM, with the GNOME or KDE look.",
      secciones: [
        {
          titulo: "Ubuntu and Debian",
          parrafos: ["With Apolo's repository, it updates with the rest of your system:"],
          comandos: [
            {
              nombre: "apt",
              lineas: [
                "sudo install -dm755 /etc/apt/keyrings",
                "sudo curl -fsSLo /etc/apt/keyrings/apolo.asc https://apolo.webcafeina.com/apolo.asc",
                "sudo curl -fsSLo /etc/apt/sources.list.d/apolo.sources https://apolo.webcafeina.com/apt/apolo.sources",
                "sudo apt update && sudo apt install apolo",
              ],
            },
          ],
        },
        {
          titulo: "Fedora and openSUSE",
          comandos: [
            { titulo: "Fedora", nombre: "dnf", lineas: ["sudo curl -fsSLo /etc/yum.repos.d/apolo.repo https://apolo.webcafeina.com/rpm/apolo.repo", "sudo dnf install apolo"] },
            { titulo: "openSUSE", nombre: "zypper", lineas: ["sudo zypper addrepo https://apolo.webcafeina.com/rpm/apolo.repo", "sudo zypper install apolo"] },
          ],
        },
        {
          titulo: "Arch, Manjaro, EndeavourOS and CachyOS",
          parrafos: ["With the PKGBUILD from this site. Coming soon to the AUR, as apolo-bin."],
          comandos: [{ nombre: "makepkg", lineas: ["mkdir apolo && cd apolo", "curl -fsSLO https://apolo.webcafeina.com/aur/PKGBUILD", "makepkg -si"] }],
        },
        {
          titulo: "Any other",
          parrafos: ["The AppImage works on almost all of them: allow it to run and open it."],
        },
        {
          titulo: "Updates",
          lista: [
            "With the repository, along with the rest of your system (apt, dnf or zypper).",
            "With a standalone .deb or .rpm, Apolo tells you about each new version and installs it with one click.",
            "On Arch, by building the PKGBUILD again.",
          ],
        },
        {
          titulo: "Requirements",
          lista: [
            "64-bit Linux, on x64 or ARM.",
            "WebKitGTK 4.1, which the package manager installs for you with the .deb and the .rpm.",
            "It looks like GNOME or KDE, depending on your desktop.",
          ],
        },
      ],
    },
  },
};
