//! Exportar el [`Documento`] a SVG, PDF, EPS o DXF, con uno de los tres estilos de dibujo.

use crate::modelo::Documento;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatoVector {
    Svg,
    Pdf,
    Eps,
    Dxf,
}

impl FormatoVector {
    pub const TODOS: [FormatoVector; 4] = [Self::Svg, Self::Pdf, Self::Eps, Self::Dxf];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Pdf => "pdf",
            Self::Eps => "eps",
            Self::Dxf => "dxf",
        }
    }

    /// El formato por la extensión de un nombre de fichero (sin distinguir mayúsculas).
    pub fn desde_extension(nombre: &str) -> Option<Self> {
        let ext = nombre
            .rsplit_once('.')
            .map(|(_, e)| e)?
            .to_ascii_lowercase();
        Self::TODOS.into_iter().find(|f| f.extension() == ext)
    }

    pub fn tipo_mime(self) -> &'static str {
        match self {
            Self::Svg => "image/svg+xml",
            Self::Pdf => "application/pdf",
            Self::Eps => "application/postscript",
            Self::Dxf => "image/vnd.dxf",
        }
    }
}

/// Cómo se dibuja.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Estilo {
    /// Formas rellenas de su color: el dibujo tal cual.
    #[default]
    Rellenar,
    /// El contorno de cada forma, sin relleno.
    TrazarFormas,
    /// Cada borde entre colores, una sola vez: para cortar (vinilo, láser, plotter).
    TrazarBordes,
}

/// Qué entidades usa el DXF: hay programas de corte y de CAD que no leen splines, o ni siquiera arcos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NivelDxf {
    /// Solo polilíneas de rectas: las curvas, aplanadas.
    Rectas,
    /// Rectas y arcos de circunferencia: polilíneas con arcos y círculos.
    Arcos,
    /// Rectas, arcos, elipses y splines: las curvas tal cual.
    #[default]
    Splines,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpcionesExportar {
    pub estilo: Estilo,
    /// Grosor del trazo en píxeles (en los estilos de trazar).
    pub grosor: f64,
    /// Color del trazo; `None`, el de cada capa.
    pub color_trazo: Option<[u8; 3]>,
    pub nivel_dxf: NivelDxf,
}

impl Default for OpcionesExportar {
    fn default() -> Self {
        Self {
            estilo: Estilo::Rellenar,
            grosor: 1.0,
            color_trazo: None,
            nivel_dxf: NivelDxf::Splines,
        }
    }
}

/// El fichero, en el formato pedido.
pub fn exportar(doc: &Documento, formato: FormatoVector, o: &OpcionesExportar) -> Vec<u8> {
    match formato {
        FormatoVector::Svg => crate::svg::escribir_con(doc, o).into_bytes(),
        FormatoVector::Pdf => crate::pdf::escribir(doc, o),
        FormatoVector::Eps => crate::eps::escribir(doc, o).into_bytes(),
        FormatoVector::Dxf => crate::dxf::escribir(doc, o).into_bytes(),
    }
}

/// Un número con dos decimales como mucho, sin ceros de más ni «-0».
pub(crate) fn num(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0;
    let r = if r == 0.0 { 0.0 } else { r };
    format!("{r}")
}
