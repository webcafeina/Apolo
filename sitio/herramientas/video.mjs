// Junta los fotogramas de frontend/e2e/video.spec.ts en el vídeo de la web
// (ADR 0033): `node herramientas/video.mjs <carpeta> <vista> <idioma>`. Cada
// fotograma dura lo que tardó en llegar el siguiente (el screencast solo manda
// los que cambian), y todo va 1,6 veces más rápido (guardar los fotogramas a
// doble densidad ralentiza la página), a 30 por segundo, sin sonido. Sale public/video/<vista>-<idioma>.mp4
// (H.264, que lo pinta cualquier navegador) y su póster en WebP, hecho con Apolo.
// Necesita ffmpeg y target/release/apolo.
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const [carpeta, vista, idioma] = process.argv.slice(2);
if (!carpeta || !vista || !idioma) throw new Error("Uso: node herramientas/video.mjs <carpeta> <vista> <idioma>");
const SITIO = resolve(import.meta.dirname, "..");
const APOLO = resolve(SITIO, "../target/release/apolo");
const salida = join(SITIO, "public/video");
mkdirSync(salida, { recursive: true });
const correr = (prog, args) => {
  const r = spawnSync(prog, args, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`${prog} ${args.join(" ")}:\n${r.stderr}`);
};

const fotogramas = JSON.parse(readFileSync(join(carpeta, "fotogramas.json"), "utf8"));
// Sin los primeros 0,4 s (la página aún cargando) y con el último un segundo más.
const inicio = fotogramas[0].t + 0.4;
const lista = fotogramas
  .filter((f, i) => f.t >= inicio || fotogramas[i + 1]?.t > inicio)
  .map((f, i, todos) => {
    const sig = todos[i + 1]?.t ?? f.t + 1;
    return `file '${f.fichero}'\nduration ${Math.max(0.001, sig - Math.max(f.t, inicio)).toFixed(4)}`;
  });
writeFileSync(join(carpeta, "lista.txt"), lista.join("\n") + `\nfile '${fotogramas.at(-1).fichero}'\n`);

const nombre = `${vista}-${idioma}`;
correr("ffmpeg", [
  "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", join(carpeta, "lista.txt"),
  "-vf", "setpts=PTS/1.6,fps=30,scale=1600:-2:flags=lanczos,format=yuv420p",
  "-c:v", "libx264", "-crf", "26", "-preset", "veryslow", "-tune", "animation", "-movflags", "+faststart", "-an",
  join(salida, `${nombre}.mp4`),
]);
// El póster: el fotograma de los 6 s, con la foto abierta.
const poster = join(carpeta, "poster.png");
correr("ffmpeg", ["-v", "error", "-y", "-ss", "6", "-i", join(salida, `${nombre}.mp4`), "-frames:v", "1", poster]);
correr(APOLO, ["webp", "-q", "80", "-quiet", poster, "-o", join(salida, `${nombre}-poster.webp`)]);
console.log(`public/video/${nombre}.mp4`);
