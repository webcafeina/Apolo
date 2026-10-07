//! Las opciones de MozJPEG, y la orden de `cjpeg` que las dice (ADR 0020).
//!
//! Las opciones se **escriben** siempre en el mismo orden y se **ejecutan**
//! escritas: [`OpcionesJpeg::orden`] da los argumentos que [`super::cjpeg`]
//! procesa como lo haría cjpeg. Así lo que se enseña es lo que se ejecuta.
//!
//! Al **leer** una orden pegada se aplica la misma precedencia que en cjpeg,
//! que en tres casos depende del orden:
//! - `-revert` vuelve a los valores de libjpeg y deshace lo anterior que vive
//!   en la librería; lo que guarda cjpeg aparte (`-quality`, `-sample`,
//!   `-qslots`, `-progressive`, `-baseline`, `-quant-baseline`) se queda;
//! - `-tune-*` fija la tabla base y las lambdas: un `-quant-table` o un
//!   `-lambda1`/`-lambda2` anteriores se pierden, y los posteriores mandan;
//!
//! `-arithmetic` no está: el cjpeg 4.1.5 oficial viene sin ella.
//!
//! El orden en que se escriben las respeta: `-revert`, `-tune-*`,
//! `-quant-table`, `-lambda*` y luego todo lo demás.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorJpeg {
    /// El de la imagen: gris si es gris, YCbCr si es color.
    #[default]
    Auto,
    /// `-grayscale`
    Gris,
    /// `-rgb`: sin pasar a YCbCr (más grande; casi nunca hace falta).
    Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Escaneo {
    /// Lo de MozJPEG: progresivo (o secuencial con `-revert`).
    #[default]
    PorDefecto,
    /// `-progressive`
    Progresivo,
    /// `-baseline`: secuencial y con tablas de 8 bits.
    Secuencial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dct {
    Int,
    Fast,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Afinado {
    Psnr,
    HvsPsnr,
    Ssim,
    MsSsim,
}

impl Afinado {
    pub const TODOS: [Afinado; 4] = [
        Afinado::HvsPsnr,
        Afinado::Psnr,
        Afinado::Ssim,
        Afinado::MsSsim,
    ];

    pub fn opcion(self) -> &'static str {
        match self {
            Afinado::Psnr => "-tune-psnr",
            Afinado::HvsPsnr => "-tune-hvs-psnr",
            Afinado::Ssim => "-tune-ssim",
            Afinado::MsSsim => "-tune-ms-ssim",
        }
    }
}

/// `-restart N` (filas de MCU) o `-restart NB` (bloques).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "en", content = "n")]
pub enum Reinicio {
    Filas(u32),
    Bloques(u32),
}

/// Las opciones de `cjpeg`. Lo que no está puesto es lo de MozJPEG.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpcionesJpeg {
    /// `-quality N[,N...]` (una por tabla). Vacío: lo de cjpeg, 75.
    pub calidad: Vec<f32>,
    /// `-revert`: los valores de libjpeg en vez de los de MozJPEG.
    pub revertir: bool,
    pub color: ColorJpeg,
    pub escaneo: Escaneo,
    /// `-quant-baseline` (o implícito en `-baseline`): tablas de 8 bits.
    pub tablas_baseline: bool,
    /// `-optimize`
    pub optimizar: bool,
    /// `-dct`
    pub dct: Option<Dct>,
    /// `-fastcrush`: sin optimizar los escaneos progresivos.
    pub rapido: bool,
    /// `-dc-scan-opt` (0, 1 o 2)
    pub dc_scan_opt: Option<i32>,
    /// `-notrellis`
    pub sin_trellis: bool,
    /// `-trellis-dc` (sí) o `-notrellis-dc` (no)
    pub trellis_dc: Option<bool>,
    /// `-trellis-dc-ver-weight`
    pub peso_dc: Option<f32>,
    /// `-tune-*`
    pub afinado: Option<Afinado>,
    /// `-quant-table` (0 a 8)
    pub tabla: Option<i32>,
    /// `-lambda1`
    pub lambda1: Option<f32>,
    /// `-lambda2`
    pub lambda2: Option<f32>,
    /// `-noovershoot`
    pub sin_overshoot: bool,
    /// `-nojfif`
    pub sin_jfif: bool,
    /// `-restart`
    pub reinicio: Option<Reinicio>,
    /// `-smooth` (0 a 100)
    pub suavizado: Option<i32>,
    /// `-sample HxV[,HxV...]`
    pub muestreo: Option<String>,
    /// `-qslots N[,N...]`
    pub ranuras: Option<String>,
    /// De Apolo: girar según la orientación EXIF antes de codificar.
    pub enderezar: bool,
}

impl Default for OpcionesJpeg {
    fn default() -> Self {
        OpcionesJpeg {
            calidad: Vec::new(),
            revertir: false,
            color: ColorJpeg::Auto,
            escaneo: Escaneo::PorDefecto,
            tablas_baseline: false,
            optimizar: false,
            dct: None,
            rapido: false,
            dc_scan_opt: None,
            sin_trellis: false,
            trellis_dc: None,
            peso_dc: None,
            afinado: None,
            tabla: None,
            lambda1: None,
            lambda2: None,
            sin_overshoot: false,
            sin_jfif: false,
            reinicio: None,
            suavizado: None,
            muestreo: None,
            ranuras: None,
            enderezar: false,
        }
    }
}

/// La opción propia de Apolo, como `-apolo_enderezar` en `apolo webp`.
pub const ENDEREZAR: &str = "-apolo_enderezar";

fn numero(v: f32) -> String {
    // Como lo escribiría una persona: 80 y no 80.0.
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl OpcionesJpeg {
    /// La calidad efectiva (la de la primera tabla), para la interfaz.
    pub fn calidad_principal(&self) -> f32 {
        self.calidad.first().copied().unwrap_or(75.0)
    }

    /// Los argumentos de cjpeg, en el orden que respeta su precedencia. Sin
    /// las de Apolo.
    pub fn orden(&self) -> Vec<String> {
        fn par(a: &mut Vec<String>, o: &str, v: String) {
            a.push(o.into());
            a.push(v);
        }
        let mut a: Vec<String> = Vec::new();
        if self.revertir {
            a.push("-revert".into());
        }
        if let Some(t) = self.afinado {
            a.push(t.opcion().into());
        }
        if let Some(t) = self.tabla {
            par(&mut a, "-quant-table", t.to_string());
        }
        if let Some(l) = self.lambda1 {
            par(&mut a, "-lambda1", numero(l));
        }
        if let Some(l) = self.lambda2 {
            par(&mut a, "-lambda2", numero(l));
        }
        if !self.calidad.is_empty() {
            let q: Vec<String> = self.calidad.iter().map(|&q| numero(q)).collect();
            par(&mut a, "-quality", q.join(","));
        }
        match self.color {
            ColorJpeg::Auto => {}
            ColorJpeg::Gris => a.push("-grayscale".into()),
            ColorJpeg::Rgb => a.push("-rgb".into()),
        }
        match self.escaneo {
            Escaneo::Secuencial => a.push("-baseline".into()),
            Escaneo::Progresivo => a.push("-progressive".into()),
            Escaneo::PorDefecto => {}
        }
        if self.tablas_baseline && self.escaneo != Escaneo::Secuencial {
            a.push("-quant-baseline".into());
        }
        if self.optimizar {
            a.push("-optimize".into());
        }
        if let Some(d) = self.dct {
            let d = match d {
                Dct::Int => "int",
                Dct::Fast => "fast",
                Dct::Float => "float",
            };
            par(&mut a, "-dct", d.into());
        }
        if let Some(n) = self.dc_scan_opt {
            par(&mut a, "-dc-scan-opt", n.to_string());
        }
        if let Some(w) = self.peso_dc {
            par(&mut a, "-trellis-dc-ver-weight", numero(w));
        }
        if let Some(r) = self.reinicio {
            let r = match r {
                Reinicio::Filas(n) => n.to_string(),
                Reinicio::Bloques(n) => format!("{n}B"),
            };
            par(&mut a, "-restart", r);
        }
        if let Some(s) = self.suavizado {
            par(&mut a, "-smooth", s.to_string());
        }
        if let Some(m) = &self.muestreo {
            par(&mut a, "-sample", m.clone());
        }
        if let Some(r) = &self.ranuras {
            par(&mut a, "-qslots", r.clone());
        }
        if self.rapido {
            a.push("-fastcrush".into());
        }
        if self.sin_trellis {
            a.push("-notrellis".into());
        }
        match self.trellis_dc {
            Some(true) => a.push("-trellis-dc".into()),
            Some(false) => a.push("-notrellis-dc".into()),
            None => {}
        }
        if self.sin_overshoot {
            a.push("-noovershoot".into());
        }
        if self.sin_jfif {
            a.push("-nojfif".into());
        }
        a
    }

    /// La orden de `apolo jpeg`: la de cjpeg más las de Apolo.
    pub fn orden_apolo(&self) -> Vec<String> {
        let mut a = self.orden();
        if self.enderezar {
            a.push(ENDEREZAR.into());
        }
        a
    }

    /// Si la orden cjpeg da exactamente este fichero (sin lo de Apolo).
    pub fn es_equivalente(&self) -> bool {
        !self.enderezar
    }

    /// Aplica una opción de cjpeg (o de Apolo) con su precedencia. `valor`
    /// es el argumento siguiente, si la opción lo lleva; devuelve si lo usó.
    fn aplicar(&mut self, op: &str, valor: Option<&str>) -> Result<bool, String> {
        let k = |p: &str, n: usize| keymatch(op, p, n);
        let falta = || format!("falta el valor de -{op}");
        let v = || valor.ok_or_else(falta);
        let num = |s: &str| -> Result<f32, String> {
            s.trim()
                .parse::<f32>()
                .map_err(|_| format!("-{op} {s}: no es un número"))
        };
        let ent = |s: &str| -> Result<i32, String> {
            s.trim()
                .parse::<i32>()
                .map_err(|_| format!("-{op} {s}: no es un número entero"))
        };
        // El mismo orden de comprobación que parse_switches: con abreviaturas,
        // la primera que encaja gana.
        if k("arithmetic", 1) {
            // Como el cjpeg 4.1.5 oficial, que viene sin ella.
            return Err("cjpeg no tiene codificación aritmética (-arithmetic)".into());
        } else if k("baseline", 1) {
            self.escaneo = Escaneo::Secuencial;
            self.tablas_baseline = true;
        } else if k("dct", 2) {
            let s = v()?;
            self.dct = Some(if keymatch(s, "int", 1) {
                Dct::Int
            } else if keymatch(s, "fast", 2) {
                Dct::Fast
            } else if keymatch(s, "float", 2) {
                Dct::Float
            } else {
                return Err(format!("-dct {s}: tiene que ser int, fast o float"));
            });
            return Ok(true);
        } else if k("fastcrush", 4) {
            self.rapido = true;
        } else if k("grayscale", 2) || k("greyscale", 2) {
            self.color = ColorJpeg::Gris;
        } else if k("rgb", 3) {
            self.color = ColorJpeg::Rgb;
        } else if k("lambda1", 7) {
            self.lambda1 = Some(num(v()?)?);
            return Ok(true);
        } else if k("lambda2", 7) {
            self.lambda2 = Some(num(v()?)?);
            return Ok(true);
        } else if k("dc-scan-opt", 3) {
            self.dc_scan_opt = Some(ent(v()?)?);
            return Ok(true);
        } else if k("optimize", 1) || k("optimise", 1) {
            self.optimizar = true;
        } else if k("progressive", 1) {
            self.escaneo = Escaneo::Progresivo;
        } else if k("quality", 1) {
            let s = v()?;
            let q: Result<Vec<f32>, String> = s.trim_end_matches(',').split(',').map(num).collect();
            self.calidad = q?;
            return Ok(true);
        } else if k("qslots", 2) {
            self.ranuras = Some(v()?.to_string());
            return Ok(true);
        } else if k("quant-table", 7) {
            let t = ent(v()?)?;
            if !(0..=8).contains(&t) {
                return Err(format!("-quant-table {t}: tiene que ir de 0 a 8"));
            }
            self.tabla = Some(t);
            return Ok(true);
        } else if k("quant-baseline", 7) {
            self.tablas_baseline = true;
        } else if k("restart", 1) {
            let s = v()?.trim();
            let (n, bloques) = match s.strip_suffix(['b', 'B']) {
                Some(n) => (n, true),
                None => (s, false),
            };
            let n: u32 = n
                .parse()
                .ok()
                .filter(|n| *n <= 65535)
                .ok_or_else(|| format!("-restart {s}: tiene que ir de 0 a 65535"))?;
            self.reinicio = Some(if bloques {
                Reinicio::Bloques(n)
            } else {
                Reinicio::Filas(n)
            });
            return Ok(true);
        } else if k("revert", 3) {
            // jpeg_set_defaults: se va todo lo que vive en cinfo.
            let guardado = OpcionesJpeg {
                calidad: std::mem::take(&mut self.calidad),
                escaneo: self.escaneo,
                tablas_baseline: self.tablas_baseline,
                muestreo: self.muestreo.take(),
                ranuras: self.ranuras.take(),
                enderezar: self.enderezar,
                revertir: true,
                ..OpcionesJpeg::default()
            };
            *self = guardado;
        } else if k("sample", 2) {
            self.muestreo = Some(v()?.to_string());
            return Ok(true);
        } else if k("smooth", 2) {
            let s = ent(v()?)?;
            if !(0..=100).contains(&s) {
                return Err(format!("-smooth {s}: tiene que ir de 0 a 100"));
            }
            self.suavizado = Some(s);
            return Ok(true);
        } else if k("notrellis-dc", 11) {
            self.trellis_dc = Some(false);
        } else if k("notrellis", 1) {
            self.sin_trellis = true;
        } else if k("trellis-dc-ver-weight", 12) {
            self.peso_dc = Some(num(v()?)?);
            return Ok(true);
        } else if k("trellis-dc", 9) {
            self.trellis_dc = Some(true);
        } else if let Some(t) = [
            ("tune-psnr", Afinado::Psnr),
            ("tune-ssim", Afinado::Ssim),
            ("tune-ms-ssim", Afinado::MsSsim),
            ("tune-hvs-psnr", Afinado::HvsPsnr),
        ]
        .iter()
        .find(|(p, _)| k(p, 6))
        .map(|(_, t)| *t)
        {
            self.afinado = Some(t);
            self.tabla = None;
            self.lambda1 = None;
            self.lambda2 = None;
        } else if k("noovershoot", 11) {
            self.sin_overshoot = true;
        } else if k("nojfif", 6) {
            self.sin_jfif = true;
        } else if format!("-{op}") == ENDEREZAR {
            self.enderezar = true;
        } else {
            return Err(format!("cjpeg no tiene la opción -{op}"));
        }
        Ok(false)
    }

    /// Lee opciones de cjpeg (y de Apolo), sin ficheros.
    pub fn leer<S: AsRef<str>>(args: &[S]) -> Result<OpcionesJpeg, String> {
        let mut o = OpcionesJpeg::default();
        let mut i = 0;
        while i < args.len() {
            let a = args[i].as_ref();
            let Some(op) = a.strip_prefix('-') else {
                return Err(format!("«{a}» no es una opción"));
            };
            if o.aplicar(op, args.get(i + 1).map(|s| s.as_ref()))? {
                i += 1;
            }
            i += 1;
        }
        Ok(o)
    }
}

/// Una orden de cjpeg entera: opciones, ficheros y lo que solo importa en
/// la línea de órdenes.
#[derive(Debug, Clone, Default)]
pub struct OrdenCjpeg {
    pub opciones: OpcionesJpeg,
    pub entrada: Option<String>,
    pub salida: Option<String>,
    /// `-icc FICHERO`
    pub icc: Option<String>,
    /// `-strict`: los avisos son errores.
    pub estricto: bool,
    /// `-version`
    pub version: bool,
    /// Opciones aceptadas que no cambian el fichero (`-verbose`, `-report`,
    /// `-memdst`, `-maxmemory`), para avisar.
    pub ignoradas: Vec<String>,
}

/// Lee una orden de cjpeg: `[opciones] [entrada]` o `-outfile salida`.
/// Como cjpeg en Unix, admite también `entrada salida` (con un aviso de que
/// es la forma de Windows). `-qtables` y `-scans` (ficheros de tablas y de
/// escaneos) no están todavía.
pub fn leer_orden<S: AsRef<str>>(args: &[S]) -> Result<OrdenCjpeg, String> {
    let mut o = OrdenCjpeg::default();
    let mut ficheros = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_ref();
        let Some(op) = a.strip_prefix('-').filter(|op| !op.is_empty()) else {
            ficheros.push(a.to_string());
            i += 1;
            continue;
        };
        let valor = args.get(i + 1).map(|s| s.as_ref().to_string());
        let falta = || format!("falta el valor de -{op}");
        if keymatch(op, "outfile", 4) {
            o.salida = Some(valor.ok_or_else(falta)?);
            i += 2;
            continue;
        }
        if keymatch(op, "icc", 1) {
            o.icc = Some(valor.ok_or_else(falta)?);
            i += 2;
            continue;
        }
        if keymatch(op, "qtables", 2) || keymatch(op, "scans", 4) {
            return Err(format!(
                "-{op}: los ficheros de tablas y de escaneos no están todavía en Apolo"
            ));
        }
        if keymatch(op, "maxmemory", 3) {
            valor.ok_or_else(falta)?;
            o.ignoradas.push(format!("-{op}"));
            i += 2;
            continue;
        }
        if keymatch(op, "debug", 1)
            || keymatch(op, "verbose", 1)
            || keymatch(op, "report", 3)
            || keymatch(op, "memdst", 2)
            || keymatch(op, "targa", 1)
        {
            o.ignoradas.push(format!("-{op}"));
            i += 1;
            continue;
        }
        if keymatch(op, "version", 4) {
            o.version = true;
            i += 1;
            continue;
        }
        if keymatch(op, "strict", 2) {
            o.estricto = true;
            i += 1;
            continue;
        }
        if o.opciones.aplicar(op, valor.as_deref())? {
            i += 1;
        }
        i += 1;
    }
    match ficheros.len() {
        0 => {}
        1 => o.entrada = ficheros.pop(),
        2 if o.salida.is_none() => {
            o.salida = ficheros.pop();
            o.entrada = ficheros.pop();
        }
        _ => return Err("cjpeg lee un solo fichero de entrada".into()),
    }
    Ok(o)
}

/// La orden para enseñar: `cjpeg <opciones> -outfile <salida> <entrada>`.
pub fn texto(op: &OpcionesJpeg, entrada: &str, salida: &str) -> String {
    let mut partes = vec!["cjpeg".to_string()];
    partes.extend(op.orden().iter().map(|a| crate::cwebp::citar(a)));
    partes.push("-outfile".into());
    partes.push(crate::cwebp::citar(salida));
    partes.push(crate::cwebp::citar(entrada));
    partes.join(" ")
}

/// `keymatch` de cdjpeg.c.
pub(super) fn keymatch(arg: &str, palabra: &str, minimo: usize) -> bool {
    let mut p = palabra.bytes();
    let mut n = 0;
    for a in arg.bytes() {
        let Some(k) = p.next() else { return false };
        if a.to_ascii_lowercase() != k {
            return false;
        }
        n += 1;
    }
    n >= minimo
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn ida_y_vuelta(o: &OpcionesJpeg) {
        assert_eq!(
            &OpcionesJpeg::leer(&o.orden_apolo()).unwrap(),
            o,
            "{:?}",
            o.orden_apolo()
        );
    }

    #[test]
    fn por_defecto_no_dice_nada() {
        assert!(OpcionesJpeg::default().orden().is_empty());
    }

    #[test]
    fn se_escriben_y_se_leen_igual() {
        ida_y_vuelta(&OpcionesJpeg {
            calidad: vec![82.0],
            afinado: Some(Afinado::Ssim),
            tabla: Some(2),
            escaneo: Escaneo::Secuencial,
            tablas_baseline: true,
            reinicio: Some(Reinicio::Bloques(16)),
            muestreo: Some("1x1".into()),
            trellis_dc: Some(false),
            dct: Some(Dct::Float),
            enderezar: true,
            ..Default::default()
        });
        ida_y_vuelta(&OpcionesJpeg {
            revertir: true,
            calidad: vec![90.0, 70.5],
            escaneo: Escaneo::Progresivo,
            tablas_baseline: true,
            optimizar: true,
            color: ColorJpeg::Gris,
            ..Default::default()
        });
    }

    #[test]
    fn la_precedencia_de_cjpeg() {
        // -revert deshace lo de la librería pero no -quality.
        let o = OpcionesJpeg::leer(&["-notrellis", "-quality", "60", "-revert"]).unwrap();
        assert!(o.revertir && !o.sin_trellis && o.calidad == vec![60.0]);
        // -tune pisa un -quant-table anterior; uno posterior manda.
        let o = OpcionesJpeg::leer(&["-quant-table", "2", "-tune-psnr"]).unwrap();
        assert_eq!((o.afinado, o.tabla), (Some(Afinado::Psnr), None));
        let o = OpcionesJpeg::leer(&["-tune-psnr", "-quant-table", "2"]).unwrap();
        assert_eq!((o.afinado, o.tabla), (Some(Afinado::Psnr), Some(2)));
        // Sin codificación aritmética, como el cjpeg oficial.
        assert!(OpcionesJpeg::leer(&["-arithmetic"]).is_err());
        // Abreviaturas, como cjpeg: -q es -quality y -dc es -dct.
        let o = OpcionesJpeg::leer(&["-q", "50", "-dc", "fast"]).unwrap();
        assert_eq!((o.calidad.clone(), o.dct), (vec![50.0], Some(Dct::Fast)));
    }

    #[test]
    fn la_orden_entera() {
        let o = leer_orden(&["-quality", "80", "-outfile", "b.jpg", "a.png"]).unwrap();
        assert_eq!(
            (o.entrada.as_deref(), o.salida.as_deref()),
            (Some("a.png"), Some("b.jpg"))
        );
        assert!(leer_orden(&["-scans", "s.txt", "a.png"]).is_err());
        assert!(leer_orden(&["-nada"]).is_err());
    }
}
