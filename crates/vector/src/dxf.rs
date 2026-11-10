//! DXF R2007 (AC1021): las formas como contornos, una capa de DXF por color (con su color real, código
//! 420). En DXF no hay rellenos ni degradados: cada capa lleva el color medio. Coordenadas en píxeles, con
//! el eje Y hacia arriba.
//!
//! Tres niveles, porque hay programas de corte y de CAD que no leen splines, o ni siquiera arcos
//! ([`NivelDxf`]). La estructura es la mínima que piden AutoCAD y compañía a partir de R2000: cabecera, las
//! nueve tablas, los bloques del espacio modelo y papel, y los objetos (diccionario raíz, grupos y diseños),
//! todo con su identificador y su dueño.

use crate::exportar::{Estilo, NivelDxf, OpcionesExportar};
use crate::geometria::P;
use crate::modelo::{Documento, Forma, Tramo};
use crate::trazado::{self, Vertice};
use std::fmt::Write;

/// Tolerancia al aplanar curvas o aproximarlas por arcos, en píxeles.
const TOL: f64 = 0.05;

fn n(x: f64) -> String {
    // Cuatro decimales: en CAD se mide.
    let r = (x * 10_000.0).round() / 10_000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    format!("{r}")
}

/// Pares código–valor.
struct Dxf {
    s: String,
    siguiente: u32,
}

impl Dxf {
    fn par(&mut self, codigo: i32, valor: impl std::fmt::Display) {
        let _ = write!(self.s, "{codigo:>3}\n{valor}\n");
    }
    fn id(&mut self) -> String {
        let h = format!("{:X}", self.siguiente);
        self.siguiente += 1;
        h
    }
}

/// El color ACI (índice de AutoCAD) más parecido, para los programas que no leen el color real.
fn aci(c: [u8; 3]) -> u8 {
    // Los siete básicos más grises, que es lo que leen bien todos.
    let base: [(u8, [u8; 3]); 10] = [
        (1, [255, 0, 0]),
        (2, [255, 255, 0]),
        (3, [0, 255, 0]),
        (4, [0, 255, 255]),
        (5, [0, 0, 255]),
        (6, [255, 0, 255]),
        (7, [255, 255, 255]),
        (8, [128, 128, 128]),
        (9, [192, 192, 192]),
        (250, [51, 51, 51]),
    ];
    let d = |a: [u8; 3], b: [u8; 3]| {
        (0..3)
            .map(|i| (a[i] as i32 - b[i] as i32).pow(2))
            .sum::<i32>()
    };
    // El negro no tiene ACI propio (el 7 es blanco o negro según el fondo).
    if d(c, [0, 0, 0]) < 40 * 40 {
        return 7;
    }
    base.iter()
        .min_by_key(|(_, b)| d(c, *b))
        .map(|x| x.0)
        .unwrap_or(7)
}

fn nombre_capa(c: [u8; 3]) -> String {
    format!("Color_{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

/// Una entidad ya escrita: lo que va tras el identificador y el dueño.
enum Entidad {
    Polilinea { v: Vec<Vertice>, cerrada: bool },
    Circulo { c: P, r: f64 },
    Elipse { c: P, eje: P, razon: f64 },
    Spline { control: Vec<P>, nudos: Vec<f64> },
}

/// Una cadena de tramos como spline de grado 3: cada tramo, una Bézier (las rectas, con los puntos de
/// control a un tercio), unidas con nudos triples.
fn spline(t: &[Tramo]) -> Entidad {
    let mut control = vec![t[0].ini()];
    for s in t {
        match *s {
            Tramo::Linea(a, b) => {
                control.push((a.0 + (b.0 - a.0) / 3.0, a.1 + (b.1 - a.1) / 3.0));
                control.push((a.0 + 2.0 * (b.0 - a.0) / 3.0, a.1 + 2.0 * (b.1 - a.1) / 3.0));
                control.push(b);
            }
            Tramo::Cubica(_, b, c, d) => {
                control.push(b);
                control.push(c);
                control.push(d);
            }
        }
    }
    let k = t.len();
    let mut nudos = vec![0.0; 4];
    for i in 1..k {
        nudos.extend([i as f64; 3]);
    }
    nudos.extend([k as f64; 4]);
    Entidad::Spline { control, nudos }
}

fn polilinea(t: &[Tramo], cerrada: bool, nivel: NivelDxf) -> Entidad {
    let mut v = match nivel {
        NivelDxf::Rectas => trazado::aplanar(t, TOL)
            .into_iter()
            .map(|p| Vertice { p, bulge: 0.0 })
            .collect(),
        _ => {
            let mut v = trazado::en_arcos(t, TOL);
            if let Some(u) = t.last() {
                v.push(Vertice {
                    p: u.fin(),
                    bulge: 0.0,
                });
            }
            v
        }
    };
    // Cerrada: el último punto es el primero y sobra.
    if cerrada && v.len() > 1 && trazado_igual(v[0].p, v[v.len() - 1].p) {
        v.pop();
    }
    Entidad::Polilinea { v, cerrada }
}

fn trazado_igual(a: P, b: P) -> bool {
    (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
}

/// Las entidades de una forma cerrada (ya con el eje Y hacia arriba).
fn entidades_forma(f: &Forma, nivel: NivelDxf) -> Entidad {
    match (f, nivel) {
        (Forma::Circulo { c, r }, NivelDxf::Arcos | NivelDxf::Splines) => {
            Entidad::Circulo { c: *c, r: *r }
        }
        (Forma::Elipse { c, rx, ry, ang }, NivelDxf::Splines) => {
            // DXF: el eje mayor como vector y la razón menor/mayor (≤ 1).
            let (mayor, menor, a) = if rx >= ry {
                (*rx, *ry, *ang)
            } else {
                (*ry, *rx, ang + std::f64::consts::FRAC_PI_2)
            };
            Entidad::Elipse {
                c: *c,
                eje: (mayor * a.cos(), mayor * a.sin()),
                razon: menor / mayor.max(1e-9),
            }
        }
        (f, NivelDxf::Splines) => {
            let t = trazado::tramos(f);
            if t.iter().all(|s| matches!(s, Tramo::Linea(..))) {
                polilinea(&t, true, nivel)
            } else {
                spline(&t)
            }
        }
        (f, _) => polilinea(&trazado::tramos(f), true, nivel),
    }
}

fn entidades_borde(t: &[Tramo], cerrado: bool, nivel: NivelDxf) -> Entidad {
    if nivel == NivelDxf::Splines && t.iter().any(|s| matches!(s, Tramo::Cubica(..))) {
        spline(t)
    } else {
        polilinea(t, cerrado, nivel)
    }
}

/// El DXF del documento.
pub fn escribir(doc: &Documento, o: &OpcionesExportar) -> String {
    let alto = doc.alto as f64;
    // El eje Y hacia arriba: y → alto − y. Las elipses giran al revés.
    let voltear = |p: P| (p.0, alto - p.1);
    let mut lista: Vec<([u8; 3], Entidad)> = Vec::new();
    let color = |c: [u8; 3]| o.color_trazo.unwrap_or(c);
    match o.estilo {
        Estilo::Rellenar | Estilo::TrazarFormas => {
            for capa in &doc.capas {
                let c = color(trazado::color_de(&capa.relleno));
                for f in &capa.formas {
                    let f = match f {
                        Forma::Circulo { c, r } => Forma::Circulo {
                            c: voltear(*c),
                            r: *r,
                        },
                        Forma::Elipse { c, rx, ry, ang } => Forma::Elipse {
                            c: voltear(*c),
                            rx: *rx,
                            ry: *ry,
                            ang: -ang,
                        },
                        Forma::Tramos(t) => Forma::Tramos(trazado::transformar(t, voltear)),
                    };
                    lista.push((c, entidades_forma(&f, o.nivel_dxf)));
                }
            }
        }
        Estilo::TrazarBordes => {
            for b in trazado::bordes(doc) {
                let t = trazado::transformar(&b.tramos, voltear);
                lista.push((color(b.color), entidades_borde(&t, b.cerrado, o.nivel_dxf)));
            }
        }
    }
    let mut colores: Vec<[u8; 3]> = lista.iter().map(|x| x.0).collect();
    colores.sort();
    colores.dedup();

    // Identificadores fijos de la estructura; a partir de 0x100, capas y entidades.
    const TABLAS: [(&str, &str); 9] = [
        ("VPORT", "1"),
        ("LTYPE", "2"),
        ("LAYER", "3"),
        ("STYLE", "4"),
        ("VIEW", "5"),
        ("UCS", "6"),
        ("APPID", "7"),
        ("DIMSTYLE", "8"),
        ("BLOCK_RECORD", "9"),
    ];
    let (bloque_modelo, bloque_papel) = ("18", "19");
    let (raiz, grupos, disenos, diseno_modelo, diseno_papel) = ("1E", "1F", "20", "21", "22");
    let mut d = Dxf {
        s: String::new(),
        siguiente: 0x100,
    };
    let capas: Vec<(String, [u8; 3], String)> = colores
        .iter()
        .map(|c| (nombre_capa(*c), *c, d.id()))
        .collect();
    let ents: Vec<(String, String, &Entidad)> = lista
        .iter()
        .map(|(c, e)| (d.id(), nombre_capa(*c), e))
        .collect();
    let semilla = format!("{:X}", d.siguiente);
    let (w, h) = (doc.ancho as f64, alto);

    // Cabecera.
    d.par(0, "SECTION");
    d.par(2, "HEADER");
    for (var, cod, val) in [
        ("$ACADVER", 1, "AC1021".to_string()),
        ("$DWGCODEPAGE", 3, "ANSI_1252".into()),
        ("$HANDSEED", 5, semilla.clone()),
        ("$INSUNITS", 70, "0".into()),
        ("$MEASUREMENT", 70, "1".into()),
    ] {
        d.par(9, var);
        d.par(cod, val);
    }
    for (var, x, y) in [
        ("$EXTMIN", 0.0, 0.0),
        ("$EXTMAX", w, h),
        ("$LIMMIN", 0.0, 0.0),
        ("$LIMMAX", w, h),
    ] {
        d.par(9, var);
        d.par(10, n(x));
        d.par(20, n(y));
        if var.starts_with("$EXT") {
            d.par(30, 0);
        }
    }
    d.par(0, "ENDSEC");
    d.par(0, "SECTION");
    d.par(2, "CLASSES");
    d.par(0, "ENDSEC");

    // Tablas.
    d.par(0, "SECTION");
    d.par(2, "TABLES");
    for (tabla, id) in TABLAS {
        d.par(0, "TABLE");
        d.par(2, tabla);
        d.par(5, id);
        d.par(330, 0);
        d.par(100, "AcDbSymbolTable");
        let cuantos = match tabla {
            "VPORT" | "STYLE" | "APPID" | "DIMSTYLE" => 1,
            "LTYPE" => 3,
            "LAYER" => 1 + capas.len(),
            "BLOCK_RECORD" => 2,
            _ => 0,
        };
        d.par(70, cuantos);
        if tabla == "DIMSTYLE" {
            d.par(100, "AcDbDimStyleTable");
        }
        let cabecera = |d: &mut Dxf, tipo: &str, h: &str, sub: &str| {
            d.par(0, tipo);
            d.par(if tipo == "DIMSTYLE" { 105 } else { 5 }, h);
            d.par(330, id);
            d.par(100, "AcDbSymbolTableRecord");
            d.par(100, sub);
        };
        match tabla {
            "VPORT" => {
                cabecera(&mut d, "VPORT", "10", "AcDbViewportTableRecord");
                d.par(2, "*Active");
                d.par(70, 0);
                for (c, v) in [
                    (10, 0.0),
                    (20, 0.0),
                    (11, 1.0),
                    (21, 1.0),
                    (12, w / 2.0),
                    (22, h / 2.0),
                ] {
                    d.par(c, n(v));
                }
                for (c, v) in [
                    (13, 0.0),
                    (23, 0.0),
                    (14, 1.0),
                    (24, 1.0),
                    (15, 1.0),
                    (25, 1.0),
                ] {
                    d.par(c, n(v));
                }
                for (c, v) in [
                    (16, 0.0),
                    (26, 0.0),
                    (36, 1.0),
                    (17, 0.0),
                    (27, 0.0),
                    (37, 0.0),
                ] {
                    d.par(c, n(v));
                }
                d.par(40, n(h.max(w)));
                d.par(41, n(w / h.max(1.0)));
                d.par(42, 50);
                d.par(43, 0);
                d.par(44, 0);
                d.par(50, 0);
                d.par(51, 0);
                for (c, v) in [
                    (71, 0),
                    (72, 100),
                    (73, 1),
                    (74, 3),
                    (75, 0),
                    (76, 0),
                    (77, 0),
                    (78, 0),
                ] {
                    d.par(c, v);
                }
            }
            "LTYPE" => {
                for (h, nombre, desc) in [
                    ("11", "ByBlock", ""),
                    ("12", "ByLayer", ""),
                    ("13", "Continuous", "Solid line"),
                ] {
                    cabecera(&mut d, "LTYPE", h, "AcDbLinetypeTableRecord");
                    d.par(2, nombre);
                    d.par(70, 0);
                    d.par(3, desc);
                    d.par(72, 65);
                    d.par(73, 0);
                    d.par(40, 0);
                }
            }
            "LAYER" => {
                let capa = |d: &mut Dxf, h: &str, nombre: &str, c: Option<[u8; 3]>| {
                    cabecera(d, "LAYER", h, "AcDbLayerTableRecord");
                    d.par(2, nombre);
                    d.par(70, 0);
                    d.par(62, c.map(aci).unwrap_or(7));
                    if let Some(c) = c {
                        d.par(420, (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32);
                    }
                    d.par(6, "Continuous");
                    d.par(370, -3);
                    d.par(390, "0");
                };
                capa(&mut d, "14", "0", None);
                for (nombre, c, h) in &capas {
                    capa(&mut d, h, nombre, Some(*c));
                }
            }
            "STYLE" => {
                cabecera(&mut d, "STYLE", "15", "AcDbTextStyleTableRecord");
                d.par(2, "Standard");
                d.par(70, 0);
                d.par(40, 0);
                d.par(41, 1);
                d.par(50, 0);
                d.par(71, 0);
                d.par(42, n(2.5));
                d.par(3, "txt");
                d.par(4, "");
            }
            "APPID" => {
                cabecera(&mut d, "APPID", "16", "AcDbRegAppTableRecord");
                d.par(2, "ACAD");
                d.par(70, 0);
            }
            "DIMSTYLE" => {
                cabecera(&mut d, "DIMSTYLE", "17", "AcDbDimStyleTableRecord");
                d.par(2, "Standard");
                d.par(70, 0);
            }
            "BLOCK_RECORD" => {
                for (h, nombre, diseno) in [
                    (bloque_modelo, "*Model_Space", diseno_modelo),
                    (bloque_papel, "*Paper_Space", diseno_papel),
                ] {
                    cabecera(&mut d, "BLOCK_RECORD", h, "AcDbBlockTableRecord");
                    d.par(2, nombre);
                    d.par(340, diseno);
                    d.par(70, 0);
                    d.par(280, 1);
                    d.par(281, 0);
                }
            }
            _ => {}
        }
        d.par(0, "ENDTAB");
    }
    d.par(0, "ENDSEC");

    // Bloques: los del espacio modelo y papel, vacíos (las entidades van en ENTITIES).
    d.par(0, "SECTION");
    d.par(2, "BLOCKS");
    for (bloque, fin, dueno, nombre, papel) in [
        ("1A", "1B", bloque_modelo, "*Model_Space", false),
        ("1C", "1D", bloque_papel, "*Paper_Space", true),
    ] {
        d.par(0, "BLOCK");
        d.par(5, bloque);
        d.par(330, dueno);
        d.par(100, "AcDbEntity");
        if papel {
            d.par(67, 1);
        }
        d.par(8, "0");
        d.par(100, "AcDbBlockBegin");
        d.par(2, nombre);
        d.par(70, 0);
        d.par(10, 0);
        d.par(20, 0);
        d.par(30, 0);
        d.par(3, nombre);
        d.par(1, "");
        d.par(0, "ENDBLK");
        d.par(5, fin);
        d.par(330, dueno);
        d.par(100, "AcDbEntity");
        if papel {
            d.par(67, 1);
        }
        d.par(8, "0");
        d.par(100, "AcDbBlockEnd");
    }
    d.par(0, "ENDSEC");

    // Entidades.
    d.par(0, "SECTION");
    d.par(2, "ENTITIES");
    for (h, capa, e) in &ents {
        let comun = |d: &mut Dxf, tipo: &str, sub: &str| {
            d.par(0, tipo);
            d.par(5, h);
            d.par(330, bloque_modelo);
            d.par(100, "AcDbEntity");
            d.par(8, capa);
            d.par(100, sub);
        };
        match e {
            Entidad::Polilinea { v, cerrada } => {
                comun(&mut d, "LWPOLYLINE", "AcDbPolyline");
                d.par(90, v.len());
                d.par(70, if *cerrada { 1 } else { 0 });
                d.par(43, 0);
                for x in v {
                    d.par(10, n(x.p.0));
                    d.par(20, n(x.p.1));
                    if x.bulge != 0.0 {
                        d.par(42, n_fino(x.bulge));
                    }
                }
            }
            Entidad::Circulo { c, r } => {
                comun(&mut d, "CIRCLE", "AcDbCircle");
                d.par(10, n(c.0));
                d.par(20, n(c.1));
                d.par(30, 0);
                d.par(40, n(*r));
            }
            Entidad::Elipse { c, eje, razon } => {
                comun(&mut d, "ELLIPSE", "AcDbEllipse");
                d.par(10, n(c.0));
                d.par(20, n(c.1));
                d.par(30, 0);
                d.par(11, n(eje.0));
                d.par(21, n(eje.1));
                d.par(31, 0);
                d.par(210, 0);
                d.par(220, 0);
                d.par(230, 1);
                d.par(40, n_fino(*razon));
                d.par(41, 0);
                d.par(42, n_fino(std::f64::consts::TAU));
            }
            Entidad::Spline { control, nudos } => {
                comun(&mut d, "SPLINE", "AcDbSpline");
                d.par(210, 0);
                d.par(220, 0);
                d.par(230, 1);
                // 8: plana.
                d.par(70, 8);
                d.par(71, 3);
                d.par(72, nudos.len());
                d.par(73, control.len());
                d.par(74, 0);
                d.par(42, "0.0000001");
                d.par(43, "0.0000001");
                for k in nudos {
                    d.par(40, n(*k));
                }
                for p in control {
                    d.par(10, n(p.0));
                    d.par(20, n(p.1));
                    d.par(30, 0);
                }
            }
        }
    }
    d.par(0, "ENDSEC");

    // Objetos: el diccionario raíz con los grupos y los diseños (Model y Layout1).
    d.par(0, "SECTION");
    d.par(2, "OBJECTS");
    d.par(0, "DICTIONARY");
    d.par(5, raiz);
    d.par(330, 0);
    d.par(100, "AcDbDictionary");
    d.par(281, 1);
    d.par(3, "ACAD_GROUP");
    d.par(350, grupos);
    d.par(3, "ACAD_LAYOUT");
    d.par(350, disenos);
    for (hd, entradas) in [
        (grupos, vec![]),
        (
            disenos,
            vec![("Model", diseno_modelo), ("Layout1", diseno_papel)],
        ),
    ] {
        d.par(0, "DICTIONARY");
        d.par(5, hd);
        d.par(102, "{ACAD_REACTORS");
        d.par(330, raiz);
        d.par(102, "}");
        d.par(330, raiz);
        d.par(100, "AcDbDictionary");
        d.par(281, 1);
        for (nombre, e) in entradas {
            d.par(3, nombre);
            d.par(350, e);
        }
    }
    for (hd, nombre, bloque, orden, modelo) in [
        (diseno_modelo, "Model", bloque_modelo, 0, true),
        (diseno_papel, "Layout1", bloque_papel, 1, false),
    ] {
        d.par(0, "LAYOUT");
        d.par(5, hd);
        d.par(102, "{ACAD_REACTORS");
        d.par(330, disenos);
        d.par(102, "}");
        d.par(330, disenos);
        d.par(100, "AcDbPlotSettings");
        d.par(1, "");
        d.par(4, "");
        d.par(6, "");
        for c in [40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 140, 141] {
            d.par(c, 0);
        }
        d.par(142, 1);
        d.par(143, 1);
        d.par(70, if modelo { 1712 } else { 688 });
        d.par(72, 0);
        d.par(73, 0);
        d.par(74, 5);
        d.par(7, "");
        d.par(75, 16);
        d.par(76, 0);
        d.par(77, 2);
        d.par(78, 300);
        d.par(147, 1);
        d.par(148, 0);
        d.par(149, 0);
        d.par(100, "AcDbLayout");
        d.par(1, nombre);
        d.par(70, 1);
        d.par(71, orden);
        d.par(10, 0);
        d.par(20, 0);
        d.par(11, n(w));
        d.par(21, n(h));
        d.par(12, 0);
        d.par(22, 0);
        d.par(32, 0);
        d.par(14, 0);
        d.par(24, 0);
        d.par(34, 0);
        d.par(15, n(w));
        d.par(25, n(h));
        d.par(35, 0);
        d.par(146, 0);
        d.par(13, 0);
        d.par(23, 0);
        d.par(33, 0);
        d.par(16, 1);
        d.par(26, 0);
        d.par(36, 0);
        d.par(17, 0);
        d.par(27, 1);
        d.par(37, 0);
        d.par(76, 1);
        d.par(330, bloque);
    }
    d.par(0, "ENDSEC");
    d.par(0, "EOF");
    d.s
}

/// Con siete decimales: abombamientos, razones y ángulos.
fn n_fino(x: f64) -> String {
    let r = (x * 1e7).round() / 1e7;
    format!("{r}")
}
