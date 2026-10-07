//! PNG con OxiPNG: el mismo fichero que el binario `oxipng` 10.2.1 (ADR 0020).
//!
//! OxiPNG no codifica píxeles, **optimiza un PNG**. Con un PNG de entrada,
//! Apolo le pasa el fichero tal cual, como `oxipng` con un fichero; con otro
//! formato, hace antes un PNG con los píxeles (y el perfil ICC, si lo hay), y
//! ese resultado ya no lo da ninguna orden de oxipng.
//!
//! Las opciones son las de la línea de órdenes de oxipng, y se convierten en
//! las `Options` del crate como lo hace su `parse_opts_into_struct`: el nivel
//! (`-o`) pone los filtros y la compresión, y lo demás los cambia encima. No
//! importa el orden.

use serde::{Deserialize, Serialize};

use oxipng::{Deflater, FilterStrategy, Options, StripChunks};

use crate::{Error, Resultado};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entrelazado {
    /// `-i off`, lo de oxipng (salvo con `--nx`, que lo deja como esté).
    #[default]
    No,
    /// `-i on`
    Si,
    /// `-i keep`
    Mantener,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quitar {
    /// Lo de oxipng: no quita nada.
    #[default]
    Nada,
    /// `--strip safe` (o `-s`): lo que no cambia cómo se ve.
    Seguro,
    /// `--strip all`: todo lo que no sea imagen.
    Todo,
}

/// Las opciones de oxipng. Lo que no está puesto es lo de oxipng.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpcionesPng {
    /// `-o` (0 a 6; «max» es 6). Por defecto, 2.
    pub nivel: u8,
    /// `-a`: cambiar el color de los píxeles transparentes si comprime más.
    pub alfa: bool,
    pub entrelazado: Entrelazado,
    pub quitar: Quitar,
    /// `--scale16`: pasar 16 bits a 8 escalando.
    pub escala16: bool,
    /// `--nb`, `--nc`, `--np`, `--ng`: no reducir la profundidad, el tipo de
    /// color, la paleta ni a gris.
    pub sin_bits: bool,
    pub sin_color: bool,
    pub sin_paleta: bool,
    pub sin_gris: bool,
    /// `--nx`: ninguna reducción (y el entrelazado se queda como esté).
    pub sin_reducciones: bool,
    /// `--nz`: no recomprimir si no hay reducción.
    pub sin_recodificar: bool,
    /// `-f`: los filtros a probar, «0,5» o «0-9».
    pub filtros: Option<String>,
    /// `--fast`
    pub rapido: bool,
    /// `--zc` (0 a 12): el nivel de libdeflate.
    pub compresion: Option<u8>,
    /// `-z`: Zopfli, mucho más lento y algo más pequeño.
    pub zopfli: bool,
    /// `--zi`: iteraciones de Zopfli (por defecto, 15).
    pub iteraciones: Option<u64>,
    /// `--ziwi`: parar si tantas iteraciones seguidas no mejoran.
    pub sin_mejora: Option<u64>,
    /// `--brute-level` (1 a 12) y `--brute-lines`.
    pub bruta_nivel: Option<u8>,
    pub bruta_lineas: Option<usize>,
    /// `--force`: escribir aunque no se gane nada.
    pub forzar: bool,
    /// `--fix`: intentar arreglar un PNG dañado.
    pub arreglar: bool,
    /// De Apolo: girar según la orientación EXIF.
    pub enderezar: bool,
}

impl Default for OpcionesPng {
    fn default() -> Self {
        OpcionesPng {
            nivel: 2,
            alfa: false,
            entrelazado: Entrelazado::No,
            quitar: Quitar::Nada,
            escala16: false,
            sin_bits: false,
            sin_color: false,
            sin_paleta: false,
            sin_gris: false,
            sin_reducciones: false,
            sin_recodificar: false,
            filtros: None,
            rapido: false,
            compresion: None,
            zopfli: false,
            iteraciones: None,
            sin_mejora: None,
            bruta_nivel: None,
            bruta_lineas: None,
            forzar: false,
            arreglar: false,
            enderezar: false,
        }
    }
}

pub const ENDEREZAR: &str = "-apolo_enderezar";

fn mal(t: impl Into<String>) -> Error {
    Error::Configuracion(t.into())
}

/// `-f`: un filtro, un rango «A-B» (A menor que B) o una lista «A,B,…» sin
/// repetidos. `parse_numeric_range_opts` de oxipng, tal cual.
fn leer_filtros(s: &str) -> Resultado<Vec<u8>> {
    let error = || {
        mal(format!(
            "-f {s}: no es un filtro, un rango ni una lista (de 0 a 9)"
        ))
    };
    if let Ok(v) = s.parse::<u8>()
        && v <= 9
    {
        return Ok(vec![v]);
    }
    let rango: Vec<&str> = s.split('-').collect();
    if rango.len() == 2 {
        if let (Ok(a), Ok(b)) = (rango[0].parse::<u8>(), rango[1].parse::<u8>())
            && a < b
            && b <= 9
        {
            return Ok((a..=b).collect());
        }
        return Err(error());
    }
    let lista: Vec<&str> = s.split(',').collect();
    if lista.len() > 1 {
        let mut v = Vec::new();
        for x in lista {
            match x.parse::<u8>() {
                Ok(n) if n <= 9 && !v.contains(&n) => v.push(n),
                _ => return Err(error()),
            }
        }
        return Ok(v);
    }
    Err(error())
}

impl OpcionesPng {
    /// Las `Options` del crate, como `parse_opts_into_struct` en oxipng.
    pub fn a_oxipng(&self) -> Resultado<Options> {
        if self.nivel > 6 {
            return Err(mal("-o va de 0 a 6 (o max)"));
        }
        let mut o = Options::from_preset(self.nivel);

        let mut lineas = self.bruta_lineas;
        let mut nivel_bruto = self.bruta_nivel;
        let filtros_previos: Vec<FilterStrategy> = o.filters.drain(..).collect();
        for mut f in filtros_previos {
            if let FilterStrategy::Brute { num_lines, level } = &mut f {
                *num_lines = lineas.unwrap_or(*num_lines);
                *level = nivel_bruto.unwrap_or(*level);
                lineas = Some(*num_lines);
                nivel_bruto = Some(*level);
            }
            o.filters.insert(f);
        }
        if let Some(f) = &self.filtros {
            o.filters = leer_filtros(f)?
                .into_iter()
                .map(|f| match f {
                    0..=4 => FilterStrategy::Basic(f.try_into().unwrap()),
                    5 => FilterStrategy::MinSum,
                    6 => FilterStrategy::Entropy,
                    7 => FilterStrategy::Bigrams,
                    8 => FilterStrategy::BigEnt,
                    _ => FilterStrategy::Brute {
                        num_lines: lineas.unwrap_or(3),
                        level: nivel_bruto.unwrap_or(1),
                    },
                })
                .collect();
        }
        o.optimize_alpha = self.alfa;
        o.scale_16 = self.escala16;
        if self.rapido {
            o.fast_evaluation = true;
        }
        o.force = self.forzar;
        o.fix_errors = self.arreglar;
        o.bit_depth_reduction = !self.sin_bits;
        o.color_type_reduction = !self.sin_color;
        o.palette_reduction = !self.sin_paleta;
        o.grayscale_reduction = !self.sin_gris;
        if self.sin_reducciones {
            o.bit_depth_reduction = false;
            o.color_type_reduction = false;
            o.palette_reduction = false;
            o.grayscale_reduction = false;
            o.interlace = None;
        }
        o.idat_recoding = !self.sin_recodificar;
        // -i: por defecto «off», salvo con --nx, que lo deja en «keep».
        o.interlace = match (self.entrelazado, self.sin_reducciones) {
            (Entrelazado::Si, _) => Some(true),
            (Entrelazado::Mantener, _) | (Entrelazado::No, true) => None,
            (Entrelazado::No, false) => Some(false),
        };
        o.strip = match self.quitar {
            Quitar::Nada => StripChunks::None,
            Quitar::Seguro => StripChunks::Safe,
            Quitar::Todo => StripChunks::All,
        };
        if self.zopfli {
            let n = |v: u64| {
                std::num::NonZeroU64::new(v)
                    .ok_or_else(|| mal("--zi y --ziwi van de 1 en adelante"))
            };
            o.deflater = Deflater::Zopfli(oxipng::ZopfliOptions {
                iteration_count: n(self.iteraciones.unwrap_or(15))?,
                iterations_without_improvement: match self.sin_mejora {
                    Some(v) => n(v)?,
                    None => std::num::NonZeroU64::MAX,
                },
                ..Default::default()
            });
        }
        if let (Deflater::Libdeflater { compression }, Some(c)) = (&mut o.deflater, self.compresion)
        {
            if c > 12 {
                return Err(mal("--zc va de 0 a 12"));
            }
            *compression = c;
        }
        Ok(o)
    }

    /// Los argumentos de oxipng que dicen estas opciones (sin las de Apolo).
    pub fn orden(&self) -> Vec<String> {
        let mut a: Vec<String> = Vec::new();
        let par = |a: &mut Vec<String>, o: &str, v: String| {
            a.push(o.into());
            a.push(v);
        };
        if self.nivel != 2 {
            par(&mut a, "-o", self.nivel.to_string());
        }
        if self.alfa {
            a.push("-a".into());
        }
        match (self.entrelazado, self.sin_reducciones) {
            (Entrelazado::Si, _) => par(&mut a, "-i", "on".into()),
            (Entrelazado::Mantener, false) => par(&mut a, "-i", "keep".into()),
            (Entrelazado::No, true) => par(&mut a, "-i", "off".into()),
            _ => {}
        }
        match self.quitar {
            Quitar::Nada => {}
            Quitar::Seguro => par(&mut a, "--strip", "safe".into()),
            Quitar::Todo => par(&mut a, "--strip", "all".into()),
        }
        if self.escala16 {
            a.push("--scale16".into());
        }
        if self.sin_reducciones {
            a.push("--nx".into());
        } else {
            for (si, o) in [
                (self.sin_bits, "--nb"),
                (self.sin_color, "--nc"),
                (self.sin_paleta, "--np"),
                (self.sin_gris, "--ng"),
            ] {
                if si {
                    a.push(o.into());
                }
            }
        }
        if self.sin_recodificar {
            a.push("--nz".into());
        }
        if let Some(f) = &self.filtros {
            par(&mut a, "-f", f.clone());
        }
        if self.rapido {
            a.push("--fast".into());
        }
        if let Some(c) = self.compresion {
            par(&mut a, "--zc", c.to_string());
        }
        if self.zopfli {
            a.push("-z".into());
            if let Some(i) = self.iteraciones {
                par(&mut a, "--zi", i.to_string());
            }
            if let Some(i) = self.sin_mejora {
                par(&mut a, "--ziwi", i.to_string());
            }
        }
        if let Some(n) = self.bruta_nivel {
            par(&mut a, "--brute-level", n.to_string());
        }
        if let Some(n) = self.bruta_lineas {
            par(&mut a, "--brute-lines", n.to_string());
        }
        if self.forzar {
            a.push("--force".into());
        }
        if self.arreglar {
            a.push("--fix".into());
        }
        a
    }

    pub fn orden_apolo(&self) -> Vec<String> {
        let mut a = self.orden();
        if self.enderezar {
            a.push(ENDEREZAR.into());
        }
        a
    }

    pub fn es_equivalente(&self) -> bool {
        !self.enderezar
    }
}

/// Una orden de oxipng entera.
#[derive(Debug, Clone, Default)]
pub struct OrdenOxipng {
    pub opciones: OpcionesPng,
    pub entradas: Vec<String>,
    /// `--out`
    pub salida: Option<String>,
    /// `--stdout`
    pub a_stdout: bool,
    /// Opciones aceptadas que no cambian el fichero (`-v`, `-q`, `-p`,
    /// `-t`, `-r`, `--sequential`…), para avisar.
    pub ignoradas: Vec<String>,
}

/// Lee una orden de oxipng: opciones y ficheros, en cualquier orden, como
/// clap. Admite `--opción valor` y `--opción=valor`, y las letras sueltas
/// juntas (`-sa`).
pub fn leer_orden<S: AsRef<str>>(args: &[S]) -> Resultado<OrdenOxipng> {
    let mut o = OrdenOxipng::default();
    let op = &mut o.opciones;
    let mut cola: std::collections::VecDeque<String> =
        args.iter().map(|s| s.as_ref().to_string()).collect();
    let mut tras_guiones = false;
    while let Some(a) = cola.pop_front() {
        if tras_guiones || !a.starts_with('-') || a == "-" {
            o.entradas.push(a);
            continue;
        }
        if a == "--" {
            tras_guiones = true;
            continue;
        }
        // --largo=valor
        let (nombre, en_linea) = match a.split_once('=') {
            Some((n, v)) if a.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        // Letras cortas juntas: -sa → -s -a (solo las que no llevan valor).
        if !nombre.starts_with("--") && nombre.len() > 2 {
            let letras: Vec<char> = nombre[1..].chars().collect();
            let con_valor = matches!(letras[0], 'o' | 'i' | 'f' | 't');
            if con_valor {
                cola.push_front(nombre[2..].to_string());
                cola.push_front(nombre[..2].to_string());
            } else {
                for l in letras.into_iter().rev() {
                    cola.push_front(format!("-{l}"));
                }
            }
            continue;
        }
        let mut valor = || -> Resultado<String> {
            en_linea
                .clone()
                .or_else(|| cola.pop_front())
                .ok_or_else(|| mal(format!("falta el valor de {nombre}")))
        };
        let num = |s: String, que: &str| -> Resultado<u64> {
            s.trim()
                .parse::<u64>()
                .map_err(|_| mal(format!("{que} {s}: no es un número")))
        };
        match nombre.as_str() {
            "-o" | "--opt" => {
                let v = valor()?;
                op.nivel = if v == "max" {
                    6
                } else {
                    v.parse::<u8>()
                        .ok()
                        .filter(|n| *n <= 6)
                        .ok_or_else(|| mal(format!("-o {v}: va de 0 a 6, o max")))?
                };
            }
            "-a" | "--alpha" => op.alfa = true,
            "-i" | "--interlace" => {
                op.entrelazado = match valor()?.as_str() {
                    "off" | "0" => Entrelazado::No,
                    "on" | "1" => Entrelazado::Si,
                    "keep" => Entrelazado::Mantener,
                    v => return Err(mal(format!("-i {v}: tiene que ser off, on o keep"))),
                }
            }
            "-s" => op.quitar = Quitar::Seguro,
            "--strip" => {
                op.quitar = match valor()?.as_str() {
                    "safe" => Quitar::Seguro,
                    "all" => Quitar::Todo,
                    v => {
                        return Err(mal(format!(
                            "--strip {v}: Apolo acepta safe y all; las listas de trozos no están todavía"
                        )));
                    }
                }
            }
            "--keep" => {
                return Err(mal("--keep no está todavía en Apolo"));
            }
            "--scale16" => op.escala16 = true,
            "--nb" => op.sin_bits = true,
            "--nc" => op.sin_color = true,
            "--np" => op.sin_paleta = true,
            "--ng" => op.sin_gris = true,
            "--nx" => op.sin_reducciones = true,
            "--nz" => op.sin_recodificar = true,
            "-f" | "--filters" => {
                let v = valor()?;
                leer_filtros(&v)?;
                op.filtros = Some(v);
            }
            "--fast" => op.rapido = true,
            "--zc" => {
                let v = num(valor()?, "--zc")?;
                if v > 12 {
                    return Err(mal("--zc va de 0 a 12"));
                }
                op.compresion = Some(v as u8);
            }
            "-z" | "--zopfli" => op.zopfli = true,
            "--zi" => op.iteraciones = Some(num(valor()?, "--zi")?),
            "--ziwi" => op.sin_mejora = Some(num(valor()?, "--ziwi")?),
            "--brute-level" => {
                let v = num(valor()?, "--brute-level")?;
                if !(1..=12).contains(&v) {
                    return Err(mal("--brute-level va de 1 a 12"));
                }
                op.bruta_nivel = Some(v as u8);
            }
            "--brute-lines" => op.bruta_lineas = Some(num(valor()?, "--brute-lines")? as usize),
            "--force" => op.forzar = true,
            "--fix" => op.arreglar = true,
            "--out" => o.salida = Some(valor()?),
            "--stdout" => o.a_stdout = true,
            ENDEREZAR => op.enderezar = true,
            "-v" | "--verbose" | "-q" | "--quiet" | "-p" | "--preserve" | "-r" | "--recursive"
            | "--sequential" | "-j" | "--json" => o.ignoradas.push(nombre),
            "-t" | "--threads" => {
                valor()?;
                o.ignoradas.push(nombre);
            }
            "--timeout" | "--max-raw-size" | "--dir" | "-d" | "--dry-run" => {
                return Err(mal(format!("{nombre} no está en Apolo")));
            }
            otro => return Err(mal(format!("oxipng no tiene la opción {otro}"))),
        }
    }
    Ok(o)
}

/// La orden para enseñar: `oxipng <opciones> --out <salida> <entrada>`.
pub fn texto(op: &OpcionesPng, entrada: &str, salida: &str) -> String {
    let mut partes = vec!["oxipng".to_string()];
    partes.extend(op.orden().iter().map(|a| crate::cwebp::citar(a)));
    partes.push("--out".into());
    partes.push(crate::cwebp::citar(salida));
    partes.push(crate::cwebp::citar(entrada));
    partes.join(" ")
}

/// Optimiza un PNG, como `oxipng <opciones> --out salida entrada`.
pub fn optimizar(png: &[u8], op: &OpcionesPng) -> Resultado<Vec<u8>> {
    let o = op.a_oxipng()?;
    oxipng::optimize_from_memory(png, &o).map_err(|e| Error::Codificacion(e.to_string()))
}

/// Un PNG sencillo a partir de píxeles (para lo que no llega como PNG), que
/// luego optimiza oxipng. Con el perfil ICC si lo hay.
pub fn png_de_pixeles(
    ancho: u32,
    alto: u32,
    rgba: &[u8],
    icc: Option<&[u8]>,
) -> Resultado<Vec<u8>> {
    let alfa = rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    let mut v = Vec::new();
    {
        let mut info = ::png::Info::with_size(ancho, alto);
        info.bit_depth = ::png::BitDepth::Eight;
        info.color_type = if alfa {
            ::png::ColorType::Rgba
        } else {
            ::png::ColorType::Rgb
        };
        info.icc_profile = icc.map(|p| std::borrow::Cow::Owned(p.to_vec()));
        let mut e = ::png::Encoder::with_info(&mut v, info)
            .map_err(|x| Error::Codificacion(x.to_string()))?;
        e.set_compression(::png::Compression::Fast);
        let mut w = e
            .write_header()
            .map_err(|x| Error::Codificacion(x.to_string()))?;
        let datos: Vec<u8> = if alfa {
            rgba.to_vec()
        } else {
            crate::entrada::quitar_alfa(rgba)
        };
        w.write_image_data(&datos)
            .map_err(|x| Error::Codificacion(x.to_string()))?;
    }
    Ok(v)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn ida_y_vuelta() {
        let o = OpcionesPng {
            nivel: 4,
            alfa: true,
            entrelazado: Entrelazado::Si,
            quitar: Quitar::Seguro,
            zopfli: true,
            iteraciones: Some(5),
            filtros: Some("0,5".into()),
            ..Default::default()
        };
        assert_eq!(leer_orden(&o.orden_apolo()).unwrap().opciones, o);
        assert!(OpcionesPng::default().orden().is_empty());
    }

    #[test]
    fn la_orden_de_oxipng() {
        let o = leer_orden(&["-o", "max", "-sa", "--out=b.png", "a.png"]).unwrap();
        assert_eq!(o.opciones.nivel, 6);
        assert!(o.opciones.alfa);
        assert_eq!(o.opciones.quitar, Quitar::Seguro);
        assert_eq!(o.salida.as_deref(), Some("b.png"));
        assert_eq!(o.entradas, ["a.png"]);
        assert_eq!(leer_filtros("0-3").unwrap(), [0, 1, 2, 3]);
        assert_eq!(leer_filtros("0,5").unwrap(), [0, 5]);
        assert!(leer_filtros("0-2,5").is_err()); // ni rangos dentro de listas
        assert!(leer_filtros("5,5").is_err()); // ni repetidos
        assert!(leer_orden(&["--nada"]).is_err());
    }
}
