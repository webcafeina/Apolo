import { describe, expect, it } from "vitest";
import { PREGUNTAS } from "../src/contenido/preguntas";
import { completo, indice } from "../src/datos/llms";
import { release } from "../src/datos/version";
import { RUTAS } from "../src/i18n/rutas";

const SITIO = new URL("https://apolo.webcafeina.com");

describe("llms.txt", () => {
  const txt = indice(SITIO);

  it("empieza como pide llmstxt.org: el título y el resumen", () => {
    expect(txt).toMatch(/^# Apolo\n\n> /);
    expect(txt).toContain(release.version);
  });

  it("enlaza las páginas en los dos idiomas (menos las cookies y el mapa para personas)", () => {
    for (const [pagina, par] of Object.entries(RUTAS)) {
      if (pagina === "cookies" || pagina === "mapa") continue;
      expect(txt, par.es).toContain(`(https://apolo.webcafeina.com${par.es})`);
      expect(txt, par.en).toContain(`(https://apolo.webcafeina.com${par.en})`);
    }
  });

  it("el completo lleva cada pregunta, en los dos idiomas", () => {
    const todo = completo(SITIO);
    for (const q of [...PREGUNTAS.es.lista, ...PREGUNTAS.en.lista]) expect(todo).toContain(`### ${q.p}`);
    expect(todo).toContain("```sh\nbrew install webcafeina/apolo/apolo\n```");
  });
});
