//! La imagen de entrada: RGBA de 8 bits, aplanada sobre blanco, con su transparencia aparte.

/// RGBA de 8 bits, fila a fila. El motor analiza los colores sobre la imagen aplanada sobre blanco (`px`, opaca);
/// la transparencia, si la hay, queda en `alfa`, y lo transparente no se dibuja (ni fondo ni capa).
#[derive(Clone)]
pub struct Rgba {
    pub ancho: u32,
    pub alto: u32,
    pub px: Vec<u8>,
    /// El alfa de cada píxel, si alguno no es opaco del todo.
    pub alfa: Option<Vec<u8>>,
}

impl Rgba {
    /// La imagen aplanada sobre blanco (la transparencia, mezclada con el fondo).
    pub fn nueva(ancho: u32, alto: u32, mut px: Vec<u8>) -> Rgba {
        assert_eq!(
            px.len(),
            (ancho * alto * 4) as usize,
            "RGBA de {ancho}×{alto}"
        );
        let alfa: Vec<u8> = px.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
        let alfa = alfa.iter().any(|a| *a < 255).then_some(alfa);
        for p in px.as_chunks_mut::<4>().0 {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
            }
            p[3] = 255;
        }
        Rgba {
            ancho,
            alto,
            px,
            alfa,
        }
    }

    /// El alfa del píxel `i` (255 si la imagen es opaca).
    pub fn alfa(&self, i: usize) -> u8 {
        self.alfa.as_ref().map_or(255, |a| a[i])
    }

    /// Si hay algo transparente del todo: entonces lo transparente es un «color» más, que no se dibuja.
    pub fn con_transparencia(&self) -> bool {
        self.alfa.as_ref().is_some_and(|a| a.contains(&0))
    }

    pub fn color(&self, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * self.ancho + x) * 4) as usize;
        [self.px[i], self.px[i + 1], self.px[i + 2]]
    }
}
