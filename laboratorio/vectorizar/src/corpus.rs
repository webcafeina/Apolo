//! Los casos del banco: de dónde sale cada imagen y cómo se prepara.

use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grupo {
    Logotipos,
    Ilustraciones,
    ByN,
    Fotos,
    PixelArt,
}

impl Grupo {
    pub const TODOS: [Grupo; 5] = [Grupo::Logotipos, Grupo::Ilustraciones, Grupo::ByN, Grupo::Fotos, Grupo::PixelArt];
    pub fn nombre(self) -> &'static str {
        match self {
            Grupo::Logotipos => "Logotipos",
            Grupo::Ilustraciones => "Ilustraciones",
            Grupo::ByN => "Blanco y negro",
            Grupo::Fotos => "Fotos",
            Grupo::PixelArt => "Pixel art",
        }
    }
    pub fn clave(self) -> &'static str {
        match self {
            Grupo::Logotipos => "logotipos",
            Grupo::Ilustraciones => "ilustraciones",
            Grupo::ByN => "byn",
            Grupo::Fotos => "fotos",
            Grupo::PixelArt => "pixelart",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Origen {
    /// Un SVG original, pasado a mapa de bits con el lado mayor `lado`. Es la referencia exacta.
    Svg { ruta: PathBuf, lado: u32, suavizado: bool },
    /// Pixel art: el SVG a su tamaño de cuadrícula, sin suavizar, ampliado `escala` veces sin filtrar.
    Pixel { ruta: PathBuf, celda: u32, escala: u32 },
    /// Una foto, reducida a `lado_max` si es mayor.
    Foto { ruta: PathBuf, lado_max: u32 },
}

#[derive(Clone, Debug)]
pub struct Caso {
    pub id: String,
    pub grupo: Grupo,
    pub origen: Origen,
}

impl Caso {
    pub fn referencia(&self) -> Option<&PathBuf> {
        match &self.origen {
            Origen::Svg { ruta, .. } | Origen::Pixel { ruta, .. } => Some(ruta),
            Origen::Foto { .. } => None,
        }
    }
}

pub fn casos(raiz: &std::path::Path) -> Vec<Caso> {
    let f = |sub: &str| raiz.join("corpus/fuentes").join(sub);
    let mut v = Vec::new();
    let svg = |v: &mut Vec<Caso>, id: String, g: Grupo, ruta: PathBuf, lado: u32, suavizado: bool| {
        v.push(Caso { id, grupo: g, origen: Origen::Svg { ruta, lado, suavizado } })
    };
    for b in ["jp", "br", "us", "ca", "tr", "kr", "ch", "gb", "za", "se"] {
        svg(&mut v, format!("bandera-{b}"), Grupo::Logotipos, f(&format!("banderas/{b}.svg")), 512, true);
    }
    // Más difíciles: pequeñas, y una sin suavizar (bordes en escalera).
    for b in ["jp", "us", "ca"] {
        svg(&mut v, format!("bandera-{b}-128"), Grupo::Logotipos, f(&format!("banderas/{b}.svg")), 128, true);
    }
    svg(&mut v, "bandera-gb-dentada".into(), Grupo::Logotipos, f("banderas/gb.svg"), 512, false);
    for e in ["1f600", "1f34e", "1f680", "1f308", "1f981", "1f33b", "1f355", "1f3a8"] {
        svg(&mut v, format!("noto-{e}"), Grupo::Ilustraciones, f(&format!("noto/emoji_u{e}.svg")), 512, true);
    }
    for e in ["1F600", "1F34E", "1F680", "2615", "1F431", "1F3E0"] {
        svg(&mut v, format!("openmoji-{}", e.to_lowercase()), Grupo::Ilustraciones, f(&format!("openmoji/{e}.svg")), 512, true);
    }
    for i in ["home", "heart", "settings", "camera", "bike", "brand-github"] {
        svg(&mut v, format!("tabler-{i}"), Grupo::ByN, f(&format!("tabler/{i}.svg")), 256, true);
    }
    for i in ["anchor", "crown-coin", "tree-branch", "feather", "caterpillar", "fox-tail"] {
        svg(&mut v, format!("silueta-{i}"), Grupo::ByN, f(&format!("gameicons/{i}.svg")), 512, true);
    }
    for i in ["heart", "home", "android", "cloud"] {
        v.push(Caso { id: format!("pixel-{i}"), grupo: Grupo::PixelArt, origen: Origen::Pixel { ruta: f(&format!("pixelarticons/{i}.svg")), celda: 24, escala: 1 } });
        v.push(Caso { id: format!("pixel-{i}-x8"), grupo: Grupo::PixelArt, origen: Origen::Pixel { ruta: f(&format!("pixelarticons/{i}.svg")), celda: 24, escala: 8 } });
    }
    for (id, fichero) in [("arandanos", "arandanos.jpg"), ("pera", "pera.png"), ("retrato", "retrato.jpg"), ("mascara", "mascara.webp"), ("libwebp", "libwebp.webp")] {
        v.push(Caso { id: format!("foto-{id}"), grupo: Grupo::Fotos, origen: Origen::Foto { ruta: f(&format!("fotos/{fichero}")), lado_max: 1024 } });
    }
    v
}
