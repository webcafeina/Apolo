// La política de cookies de esta web (ADR 0033), en los dos idiomas. El aviso
// legal y la privacidad son los de webcafeina.com. Si el contenedor de GTM
// cambia de herramientas, esta lista tiene que cambiar con él.
import type { Idioma } from "../i18n/rutas";
import type { PaginaDescarga as PaginaSecciones } from "./descargar";

export const COOKIES: Record<Idioma, PaginaSecciones> = {
  es: {
    titulo: "Política de cookies · Apolo",
    descripcion: "Qué guarda la web de Apolo en tu navegador, para qué, cuánto dura y cómo cambiar tu elección.",
    etiqueta: "Cookies",
    h1: "Política de cookies.",
    entrada: "Qué guarda esta web en tu navegador, para qué y cómo cambiarlo. La aplicación Apolo no usa cookies ni envía tus imágenes a ningún sitio.",
    secciones: [
      {
        titulo: "Quién es el responsable",
        parrafos: [
          "Webcafeína, que hace Apolo. Sus datos y cómo ejercer tus derechos están en su aviso legal y en su política de privacidad, enlazados al pie de cada página.",
        ],
      },
      {
        titulo: "Lo que se guarda siempre",
        parrafos: ["Es necesario para que la web funcione como la eliges, y no sale de tu navegador:"],
        lista: [
          "apolo-vista, apolo-distro y apolo-arquitectura: el sistema, la distribución y el procesador que eliges, para enseñarte las descargas que te tocan. Van en el almacenamiento local del navegador, no en cookies.",
          "apolo-consentimiento: tu elección sobre las cookies, durante 12 meses.",
        ],
      },
      {
        titulo: "Lo que se guarda solo si aceptas",
        lista: [
          "_ga: de Google Analytics, para distinguir a quien visita. Dura 2 años.",
          "_ga_ seguida del identificador de la propiedad: de Google Analytics, para mantener la sesión. Dura 2 años.",
        ],
        parrafos: [
          "Sirven para contar visitas y descargas, sin saber quién eres. Las trata Google Ireland Limited, que puede transferir datos a Estados Unidos con las garantías del Marco de Privacidad de Datos UE-EE. UU. Sin tu permiso, Google Analytics no se carga.",
        ],
      },
      {
        titulo: "Cómo cambiar tu elección",
        parrafos: [
          "Con «Preferencias de cookies», al pie de cada página. También puedes borrar las cookies y el almacenamiento de esta web desde los ajustes de tu navegador.",
        ],
      },
      {
        titulo: "Cambios",
        parrafos: ["Si cambian las cookies que usa esta web, te lo volverá a preguntar."],
      },
    ],
  },
  en: {
    titulo: "Cookie policy · Apolo",
    descripcion: "What Apolo's website stores in your browser, what for, how long, and how to change your choice.",
    etiqueta: "Cookies",
    h1: "Cookie policy.",
    entrada: "What this site stores in your browser, what for, and how to change it. The Apolo app doesn't use cookies or send your images anywhere.",
    secciones: [
      {
        titulo: "Who is responsible",
        parrafos: [
          "Webcafeína, who makes Apolo. Its details and how to exercise your rights are in its legal notice and privacy policy, linked in the footer of every page.",
        ],
      },
      {
        titulo: "What is always stored",
        parrafos: ["It's needed for the site to work the way you choose, and it never leaves your browser:"],
        lista: [
          "apolo-vista, apolo-distro and apolo-arquitectura: the system, distribution and processor you choose, to show you the right downloads. They live in the browser's local storage, not in cookies.",
          "apolo-consentimiento: your cookie choice, for 12 months.",
        ],
      },
      {
        titulo: "What is stored only if you accept",
        lista: [
          "_ga: from Google Analytics, to tell visitors apart. It lasts 2 years.",
          "_ga_ followed by the property ID: from Google Analytics, to keep the session. It lasts 2 years.",
        ],
        parrafos: [
          "They're used to count visits and downloads, without knowing who you are. Google Ireland Limited processes them, and may transfer data to the United States under the EU-US Data Privacy Framework. Without your permission, Google Analytics doesn't load.",
        ],
      },
      {
        titulo: "How to change your choice",
        parrafos: [
          "With “Cookie preferences”, in the footer of every page. You can also delete this site's cookies and storage from your browser's settings.",
        ],
      },
      {
        titulo: "Changes",
        parrafos: ["If the cookies this site uses change, it will ask you again."],
      },
    ],
  },
};
