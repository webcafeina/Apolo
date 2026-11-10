//! EPS 3.0 (PostScript de nivel 3, por los degradados con `shfill`), del tamaño de la imagen. El prólogo
//! define los operadores de PDF con sus nombres, y el dibujo es el mismo contenido que el del PDF
//! (`pintura`). Las definiciones van en un diccionario propio, `ApoloDict`, para no pisar las del
//! programa que incruste el EPS.

use crate::exportar::OpcionesExportar;
use crate::modelo::Documento;
use crate::pintura;

const PROLOGO: &str = "/ApoloDict 20 dict def
ApoloDict begin
/m {moveto} bind def
/l {lineto} bind def
/c {curveto} bind def
/h {closepath} bind def
/re {4 2 roll moveto 1 index 0 rlineto 0 exch rlineto neg 0 rlineto closepath} bind def
/rg {setrgbcolor} bind def
/RG {setrgbcolor} bind def
/f* {eofill} bind def
/S {stroke} bind def
/w {setlinewidth} bind def
/j {setlinejoin} bind def
/J {setlinecap} bind def
/q {gsave} bind def
/Q {grestore} bind def
/W* {eoclip} bind def
/n {newpath} bind def
end
";

pub fn escribir(doc: &Documento, o: &OpcionesExportar) -> String {
    let cuerpo = pintura::contenido(doc, o, &mut |_, dic| format!("{dic} shfill"));
    let (w, h) = (doc.ancho, doc.alto);
    format!(
        "%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 0 {w} {h}
%%HiResBoundingBox: 0 0 {w} {h}
%%Creator: Apolo
%%Title: Apolo: vectorizar
%%LanguageLevel: 3
%%Pages: 1
%%EndComments
%%BeginProlog
{PROLOGO}%%EndProlog
%%Page: 1 1
save
ApoloDict begin
0 {h} translate 1 -1 scale
{cuerpo}end
restore
showpage
%%EOF
"
    )
}
