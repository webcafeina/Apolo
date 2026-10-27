// La versión que enseña la web, con sus ficheros: la de release.json, que
// escribe web.yml al compilar (ADR 0033).
import json from "./release.json";
import sumas from "./sumas.txt?raw";
import { leerRelease } from "./release";

export const release = leerRelease(json, sumas);
