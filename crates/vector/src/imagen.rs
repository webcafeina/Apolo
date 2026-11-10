//! La imagen de entrada: RGBA de 8 bits, aplanada sobre blanco.

/// RGBA de 8 bits, fila a fila. El motor trabaja sobre la imagen opaca (aplanada sobre blanco).
#[derive(Clone)]
pub struct Rgba {
    pub ancho: u32,
    pub alto: u32,
    pub px: Vec<u8>,
}

impl Rgba {
    /// La imagen aplanada sobre blanco (la transparencia, mezclada con el fondo).
    pub fn nueva(ancho: u32, alto: u32, mut px: Vec<u8>) -> Rgba {
        assert_eq!(
            px.len(),
            (ancho * alto * 4) as usize,
            "RGBA de {ancho}×{alto}"
        );
        for p in px.as_chunks_mut::<4>().0 {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
            }
            p[3] = 255;
        }
        Rgba { ancho, alto, px }
    }

    pub fn color(&self, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * self.ancho + x) * 4) as usize;
        [self.px[i], self.px[i + 1], self.px[i + 2]]
    }
}
