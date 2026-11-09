//! `resultados.csv` y la galería (una página sin dependencias, lista para publicar como artefacto).

use crate::corpus::{Caso, Grupo};
use crate::Fila;
use std::fmt::Write as _;
use std::path::Path;

pub fn csv(ruta: &Path, filas: &[Fila]) -> std::io::Result<()> {
    let mut s = String::from("caso,grupo,motor,nota,bytes,trazados,figuras,nodos,colores,ms,error\n");
    for f in filas {
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{},{},{}",
            f.caso,
            f.grupo.clave(),
            f.motor,
            f.nota.map(|n| format!("{n:.2}")).unwrap_or_default(),
            f.bytes,
            f.trazados,
            f.figuras,
            f.nodos,
            f.colores,
            f.ms,
            f.error.as_deref().unwrap_or("").replace([',', '\n'], " ")
        );
    }
    std::fs::write(ruta, s)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn kb(b: usize) -> String {
    if b >= 1024 * 1024 { format!("{:.1} MB", b as f64 / 1048576.0) } else { format!("{:.1} KB", b as f64 / 1024.0) }
}

/// El orden de las columnas: la referencia, los motores del banco y Vectorizer.ai al final.
fn motores_de(filas: &[Fila]) -> Vec<String> {
    let mut v = vec!["referencia".to_string()];
    v.extend(crate::motores::MOTORES.iter().map(|m| m.id.to_string()));
    v.push("vectorizer-ai".into());
    v.retain(|m| filas.iter().any(|f| &f.motor == m));
    v
}

fn descripcion(m: &str) -> &'static str {
    match m {
        "referencia" => "El SVG original del que sale la imagen: lo que habría que encontrar.",
        "vectorizer-ai" => "Vectorizer.ai, descargado con la suscripción del cliente. Solo para comparar.",
        _ => crate::motores::MOTORES.iter().find(|x| x.id == m).map(|x| x.descripcion).unwrap_or(""),
    }
}

pub fn galeria(ruta: &Path, casos: &[Caso], filas: &[Fila]) -> std::io::Result<()> {
    let todos = motores_de(filas);
    escribir(ruta, casos, filas, &todos, &todos, &|c, m| Some(if m == "original" { format!("{c}/original.png") } else { format!("{c}/{m}.svg") }))
}

/// La misma galería en un solo fichero, con las imágenes dentro (para publicarla como artefacto):
/// los SVG tal cual, salvo los de fotos, que van como JPEG de su versión rasterizada.
pub fn galeria_artefacto(ruta: &Path, salida: &Path, casos: &[Caso], filas: &[Fila]) -> std::io::Result<()> {
    let todos = motores_de(filas);
    let visibles: Vec<String> = todos.iter().filter(|m| !["paleta", "vtracer-poster", "local", "local-x3", "vtracer-mosaico", "vtracer-byn"].contains(&m.as_str())).cloned().collect();
    let foto = |c: &str| casos.iter().any(|x| x.id == c && x.grupo == Grupo::Fotos);
    let src = |c: &str, m: &str| -> Option<String> {
        let dir = salida.join(c);
        if m == "original" || foto(c) {
            let png = dir.join(if m == "original" { "original.png".to_string() } else { format!("{m}.png") });
            let im = image::open(&png).ok()?.to_rgb8();
            if m == "original" && !foto(c) {
                return Some(format!("data:image/png;base64,{}", base64(&std::fs::read(&png).ok()?)));
            }
            let mut buf = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 80).encode_image(&im).ok()?;
            return Some(format!("data:image/jpeg;base64,{}", base64(&buf)));
        }
        let svg = std::fs::read(dir.join(format!("{m}.svg"))).ok()?;
        Some(format!("data:image/svg+xml;base64,{}", base64(&svg)))
    };
    escribir(ruta, casos, filas, &todos, &visibles, &src)
}

fn base64(d: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(d.len() * 4 / 3 + 4);
    for t in d.chunks(3) {
        let n = (t[0] as u32) << 16 | (*t.get(1).unwrap_or(&0) as u32) << 8 | *t.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= t.len() { s.push(A[(n >> (18 - 6 * i) & 63) as usize] as char) } else { s.push('=') }
        }
    }
    s
}

fn escribir(ruta: &Path, casos: &[Caso], filas: &[Fila], motores: &[String], visibles: &[String], src: &dyn Fn(&str, &str) -> Option<String>) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str(CABEZA);
    s.push_str("<main class=\"hoja\"><header><div class=\"etiqueta\">Apolo · Laboratorio · Fase 1</div><h1>Banco de vectorizar</h1>");
    let _ = write!(s, "<p class=\"nota\">{} imágenes con licencia libre en cinco grupos. Cada SVG se vuelve a pasar a imagen con resvg y se mide frente al original con SSIMULACRA 2, la nota de Apolo: 90 o más no se distingue, 70 es buena calidad, 50 ya se nota. Nodos: órdenes de dibujo dentro de los trazados; cuantos menos, más fácil de editar.</p>", casos.len());
    s.push_str("<div class=\"mandos\" role=\"group\" aria-label=\"Ampliación\"><label><input type=\"radio\" name=\"lupa\" id=\"lupa1\" checked> Tamaño real</label><label><input type=\"radio\" name=\"lupa\" id=\"lupa4\"> Lupa 4×</label></div></header>");

    s.push_str("<section><h2>Los motores</h2><dl class=\"motores\">");
    for m in motores {
        let _ = write!(s, "<dt>{}</dt><dd>{}</dd>", esc(m), esc(descripcion(m)));
    }
    s.push_str("</dl></section>");

    for g in Grupo::TODOS {
        let casos_g: Vec<&Caso> = casos.iter().filter(|c| c.grupo == g).collect();
        if casos_g.is_empty() {
            continue;
        }
        let _ = write!(s, "<section id=\"{}\"><h2>{}</h2>", g.clave(), g.nombre());
        // Resumen: medias por motor en este grupo.
        s.push_str("<div class=\"tabla\"><table><thead><tr><th>Motor</th><th>Nota media</th><th>Peor nota</th><th>Nodos (media)</th><th>Peso (media)</th><th>Tiempo (media)</th><th>Casos</th></tr></thead><tbody>");
        let mut resumen: Vec<(String, f64, f64, f64, f64, f64, usize)> = Vec::new();
        for m in motores {
            let fs: Vec<&Fila> = filas.iter().filter(|f| f.grupo == g && &f.motor == m && f.nota.is_some()).collect();
            if fs.is_empty() {
                continue;
            }
            let n = fs.len() as f64;
            let media = fs.iter().map(|f| f.nota.unwrap()).sum::<f64>() / n;
            let peor = fs.iter().map(|f| f.nota.unwrap()).fold(f64::INFINITY, f64::min);
            let nodos = fs.iter().map(|f| f.nodos as f64).sum::<f64>() / n;
            let bytes = fs.iter().map(|f| f.bytes as f64).sum::<f64>() / n;
            let ms = fs.iter().map(|f| f.ms as f64).sum::<f64>() / n;
            resumen.push((m.clone(), media, peor, nodos, bytes, ms, fs.len()));
        }
        let mejor = resumen.iter().filter(|r| r.0 != "referencia").map(|r| r.1).fold(f64::NEG_INFINITY, f64::max);
        for r in &resumen {
            let clase = if r.0 != "referencia" && (r.1 - mejor).abs() < 1e-9 { " class=\"mejor\"" } else { "" };
            let _ = write!(s, "<tr{clase}><th scope=\"row\">{}</th><td>{:.1}</td><td>{:.1}</td><td>{:.0}</td><td>{}</td><td>{:.0} ms</td><td>{}</td></tr>", esc(&r.0), r.1, r.2, r.3, kb(r.4 as usize), r.5, r.6);
        }
        s.push_str("</tbody></table></div>");

        for c in casos_g {
            let _ = write!(s, "<article class=\"caso\"><h3>{}</h3><div class=\"fila\">", esc(&c.id));
            let _ = write!(s, "<figure><div class=\"marco\"><img src=\"{}\" alt=\"Original de {}\" loading=\"lazy\"></div><figcaption><b>Original</b></figcaption></figure>", src(&c.id, "original").unwrap_or_default(), esc(&c.id));
            for m in visibles {
                let Some(f) = filas.iter().find(|f| f.caso == c.id && &f.motor == m) else { continue };
                let pie = match (&f.nota, &f.error) {
                    (Some(n), _) => format!("<b>{n:.1}</b> · {} · {} nodos", kb(f.bytes), f.nodos),
                    (None, Some(e)) => format!("Error: {}", esc(e)),
                    _ => String::new(),
                };
                let Some(imagen) = src(&c.id, m) else { continue };
                let _ = write!(s, "<figure><div class=\"marco\"><img src=\"{imagen}\" alt=\"{1} sobre {0}\" loading=\"lazy\"></div><figcaption><span class=\"motor\">{1}</span>{2}</figcaption></figure>", esc(&c.id), esc(m), pie);
            }
            s.push_str("</div></article>");
        }
        s.push_str("</section>");
    }
    s.push_str("</main>");
    std::fs::write(ruta, s)
}

const CABEZA: &str = r#"<title>Banco de vectorizar</title>
<meta name="claude:viewport-fit-sides" content="cover">
<style>
/* Hoja de laboratorio: tablas de medidas arriba, comparación visual debajo. Paleta de Apolo. */
:root { --bg:#ffffff; --suave:#f5f5f7; --fg:#1d1d1f; --tenue:#5c5c62; --oro:#7a5c14; --borde:#dcdce0; --mejor:#fff3d1;
  --texto: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; --mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace; }
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { --bg:#1e1e1e; --suave:#28282b; --fg:#f5f5f7; --tenue:#b4b4ba; --oro:#ffc83d; --borde:#3c3c40; --mejor:#4a3a12; color-scheme: dark } }
:root[data-theme="dark"] { --bg:#1e1e1e; --suave:#28282b; --fg:#f5f5f7; --tenue:#b4b4ba; --oro:#ffc83d; --borde:#3c3c40; --mejor:#4a3a12; color-scheme: dark }
body { background: var(--bg); color: var(--fg); font: 15px/1.5 var(--texto); }
.hoja { max-width: 1400px; margin-inline: auto; padding-block: 32px 64px; padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px)); display: grid; gap: 40px; }
h1 { font-size: clamp(28px, 4vw, 40px); margin: 0 0 8px; letter-spacing: -0.02em; }
h2 { font-size: 24px; margin: 0 0 12px; }
h3 { font: 600 14px var(--mono); margin: 0 0 8px; color: var(--tenue); }
p { margin: 0; max-width: 75ch; }
.nota { color: var(--tenue); }
.etiqueta { font-size: 12px; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; color: var(--oro); margin-bottom: 6px; }
.mandos { display: flex; gap: 16px; margin-top: 12px; }
.motores { display: grid; grid-template-columns: max-content 1fr; gap: 6px 16px; margin: 0; }
.motores dt { font: 600 13px var(--mono); }
.motores dd { margin: 0; color: var(--tenue); }
section { display: grid; gap: 20px; min-width: 0; }
.tabla { overflow-x: auto; }
table { border-collapse: collapse; font-variant-numeric: tabular-nums; min-width: 640px; }
th, td { padding: 6px 12px; border-bottom: 1px solid var(--borde); text-align: right; }
th:first-child { text-align: left; font: 600 13px var(--mono); }
thead th { font: 600 12px var(--texto); color: var(--tenue); text-align: right; }
thead th:first-child { font: 600 12px var(--texto); }
tr.mejor { background: var(--mejor); }
.caso { display: grid; gap: 4px; min-width: 0; }
.fila { display: flex; gap: 12px; overflow-x: auto; padding-bottom: 8px; }
figure { margin: 0; flex: 0 0 auto; width: 200px; display: grid; gap: 4px; }
.marco { width: 200px; height: 200px; border: 1px solid var(--borde); border-radius: 8px; overflow: hidden; display: grid; place-items: center; background: #fff; }
.marco img { max-width: 100%; max-height: 100%; }
body:has(#lupa4:checked) .marco img { transform: scale(4); image-rendering: pixelated; }
figcaption { font-size: 12px; color: var(--tenue); display: grid; }
figcaption b { color: var(--fg); }
.motor { font: 600 12px var(--mono); color: var(--fg); }
</style>
"#;
