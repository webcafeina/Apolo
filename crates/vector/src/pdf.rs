//! PDF 1.4 de una página, del tamaño de la imagen (un píxel, un punto). El contenido va comprimido con
//! Deflate y los degradados son sombreados axiales y radiales (tipos 2 y 3).

use crate::exportar::OpcionesExportar;
use crate::modelo::Documento;
use crate::pintura;
use std::fmt::Write;

pub fn escribir(doc: &Documento, o: &OpcionesExportar) -> Vec<u8> {
    let mut sombreados = Vec::new();
    let cuerpo = pintura::contenido(doc, o, &mut |i, dic| {
        sombreados.push(format!("/Sh{i} {dic}"));
        format!("/Sh{i} sh")
    });
    // La página tiene el eje Y hacia arriba; el dibujo, hacia abajo.
    let contenido = format!("1 0 0 -1 0 {} cm\n{cuerpo}", doc.alto);
    let comprimido = miniz_oxide::deflate::compress_to_vec_zlib(contenido.as_bytes(), 9);

    let recursos = if sombreados.is_empty() {
        String::new()
    } else {
        format!(" /Resources << /Shading << {} >> >>", sombreados.join(" "))
    };
    let objetos: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}]{recursos} /Contents 4 0 R >>",
            doc.ancho, doc.alto
        )
        .into_bytes(),
        {
            let mut s = format!(
                "<< /Length {} /Filter /FlateDecode >>\nstream\n",
                comprimido.len()
            )
            .into_bytes();
            s.extend_from_slice(&comprimido);
            s.extend_from_slice(b"\nendstream");
            s
        },
        b"<< /Producer (Apolo) /Creator (Apolo: vectorizar) >>".to_vec(),
    ];
    let mut pdf: Vec<u8> = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut desplaz = Vec::new();
    for (i, cuerpo) in objetos.iter().enumerate() {
        desplaz.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        pdf.extend_from_slice(cuerpo);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    let mut tabla = format!("xref\n0 {}\n0000000000 65535 f \n", objetos.len() + 1);
    for d in desplaz {
        // Cada entrada, 20 bytes justos: el espacio antes del salto de línea es parte del formato.
        let _ = writeln!(tabla, "{d:010} 00000 n ");
    }
    let _ = write!(
        tabla,
        "trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objetos.len() + 1
    );
    pdf.extend_from_slice(tabla.as_bytes());
    pdf
}
