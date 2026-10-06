//! EXIF, ICC y XMP: leerlos de la entrada y escribirlos en el WebP.
//!
//! La escritura es `WriteWebPWithMetadata` de `examples/cwebp.c` (libwebp
//! 1.6.0) traída a Rust tal cual, porque de ella depende que el fichero sea
//! idéntico al de cwebp: el orden de los trozos (ICCP, imagen, EXIF, XMP), el
//! relleno a par y cómo se monta la cabecera VP8X.

use serde::{Deserialize, Serialize};

/// Los metadatos que trae una imagen. Cada uno, los bytes crudos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadatos {
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub xmp: Option<Vec<u8>>,
}

/// Qué metadatos copiar a la salida (`cwebp -metadata`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conservar {
    pub exif: bool,
    pub icc: bool,
    pub xmp: bool,
}

impl Conservar {
    pub const NINGUNO: Conservar = Conservar {
        exif: false,
        icc: false,
        xmp: false,
    };
    pub const TODOS: Conservar = Conservar {
        exif: true,
        icc: true,
        xmp: true,
    };

    pub fn alguno(self) -> bool {
        self.exif || self.icc || self.xmp
    }
}

/// Qué se escribió, para contarlo.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Escritos {
    pub exif: Option<usize>,
    pub icc: Option<usize>,
    pub xmp: Option<usize>,
}

const CABECERA_TROZO: usize = 8;
const ETIQUETA: usize = 4;
const CABECERA_RIFF: usize = 12;
const TROZO_VP8X: usize = 18;

fn conservado(dato: &Option<Vec<u8>>, si: bool) -> Option<&[u8]> {
    match dato {
        Some(b) if si && !b.is_empty() => Some(b),
        _ => None,
    }
}

fn trozo(salida: &mut Vec<u8>, fourcc: &[u8; 4], datos: &[u8]) {
    salida.extend_from_slice(fourcc);
    salida.extend_from_slice(&(datos.len() as u32).to_le_bytes());
    salida.extend_from_slice(datos);
    if datos.len() & 1 == 1 {
        salida.push(0);
    }
}

/// Añade los metadatos al WebP `webp` recién codificado de `ancho`×`alto`.
/// Sin nada que añadir, devuelve el WebP tal cual.
pub fn escribir(
    webp: &[u8],
    ancho: u32,
    alto: u32,
    metadatos: &Metadatos,
    conservar: Conservar,
) -> Result<(Vec<u8>, Escritos), String> {
    const ALFA: u32 = 0x10;
    const EXIF: u32 = 0x08;
    const ICCP: u32 = 0x20;
    const XMP: u32 = 0x04;

    let exif = conservado(&metadatos.exif, conservar.exif);
    let icc = conservado(&metadatos.icc, conservar.icc);
    let xmp = conservado(&metadatos.xmp, conservar.xmp);

    let mut banderas = 0u32;
    let mut tamano_metadatos = 0u64;
    for (dato, bandera) in [(exif, EXIF), (icc, ICCP), (xmp, XMP)] {
        if let Some(d) = dato {
            banderas |= bandera;
            tamano_metadatos += (CABECERA_TROZO + d.len() + (d.len() & 1)) as u64;
        }
    }

    let mut escritos = Escritos::default();
    if webp.len() < CABECERA_RIFF + CABECERA_TROZO {
        return Err("El WebP codificado es demasiado corto".into());
    }
    let maximo = u32::MAX as u64 - CABECERA_TROZO as u64 - 1;
    if (webp.len() - CABECERA_TROZO) as u64 + tamano_metadatos > maximo {
        return Err("Con los metadatos, el fichero pasaría del límite del contenedor".into());
    }
    if tamano_metadatos == 0 {
        return Ok((webp.to_vec(), escritos));
    }

    let tiene_vp8x = &webp[CABECERA_RIFF..CABECERA_RIFF + ETIQUETA] == b"VP8X";
    let tamano_riff = (webp.len() - CABECERA_TROZO) as u64
        + if tiene_vp8x { 0 } else { TROZO_VP8X as u64 }
        + tamano_metadatos;

    let mut s = Vec::with_capacity(webp.len() + tamano_metadatos as usize + TROZO_VP8X);
    s.extend_from_slice(&webp[..ETIQUETA]); // RIFF
    s.extend_from_slice(&(tamano_riff as u32).to_le_bytes());
    let mut resto = &webp[CABECERA_TROZO..];
    s.extend_from_slice(&resto[..ETIQUETA]); // WEBP
    resto = &resto[ETIQUETA..];

    if tiene_vp8x {
        let mut vp8x = resto[..TROZO_VP8X].to_vec();
        vp8x[CABECERA_TROZO] |= (banderas & 0xff) as u8;
        s.extend_from_slice(&vp8x);
        resto = &resto[TROZO_VP8X..];
    } else {
        // En VP8L, la presencia de alfa va en el bit 28 tras la firma.
        if &resto[..ETIQUETA] == b"VP8L" && resto[CABECERA_TROZO + 4] & (1 << 4) != 0 {
            banderas |= ALFA;
        }
        s.extend_from_slice(b"VP8X\x0a\x00\x00\x00");
        s.extend_from_slice(&banderas.to_le_bytes());
        s.extend_from_slice(&(ancho - 1).to_le_bytes()[..3]);
        s.extend_from_slice(&(alto - 1).to_le_bytes()[..3]);
    }
    if let Some(d) = icc {
        trozo(&mut s, b"ICCP", d);
        escritos.icc = Some(d.len());
    }
    s.extend_from_slice(resto);
    if let Some(d) = exif {
        trozo(&mut s, b"EXIF", d);
        escritos.exif = Some(d.len());
    }
    if let Some(d) = xmp {
        trozo(&mut s, b"XMP ", d);
        escritos.xmp = Some(d.len());
    }
    Ok((s, escritos))
}
