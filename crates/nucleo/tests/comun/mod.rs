//! Lo que comparten las pruebas de equivalencia: el corpus de imágenes que se
//! genera a partir de `pruebas/corpus/foto.webp`, con una variante por cada
//! camino de los lectores (alfa, 16 bits, gamma, paleta, gris, metadatos,
//! TIFF, PNM, WebP, YUV).

#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub struct Caso {
    pub nombre: String,
    pub datos: Vec<u8>,
    /// Opciones que hay que añadir siempre (p. ej. `-s` para el YUV).
    pub previas: Vec<String>,
}

pub fn tempdir() -> PathBuf {
    let d = std::env::temp_dir().join(format!("apolo-equivalencia-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub mod corpus {
    use super::*;
    use libwebp_sys as w;

    /// La foto de ejemplo de libwebp, decodificada a RGBA.
    fn foto() -> (u32, u32, Vec<u8>) {
        let datos = std::fs::read(raiz().join("pruebas/corpus/foto.webp")).unwrap();
        let (mut ancho, mut alto) = (0, 0);
        // SAFETY: búfer propio; libwebp devuelve memoria que se libera con WebPFree.
        unsafe {
            let p = w::WebPDecodeRGBA(datos.as_ptr(), datos.len(), &mut ancho, &mut alto);
            assert!(!p.is_null());
            let n = (ancho * alto * 4) as usize;
            let v = std::slice::from_raw_parts(p, n).to_vec();
            w::WebPFree(p as *mut _);
            (ancho as u32, alto as u32, v)
        }
    }

    /// Una foto más grande y de tamaño impar: la de ejemplo en mosaico con
    /// espejos, para que haya bordes de macrobloque y algo de tamaño.
    fn foto_grande() -> (u32, u32, Vec<u8>) {
        let (w0, h0, f) = foto();
        let (w, h) = (w0 * 3 - 17, h0 * 2 + 9);
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let (tx, ty) = (x / w0, y / h0);
                let (mut sx, mut sy) = (x % w0, y % h0);
                if tx % 2 == 1 {
                    sx = w0 - 1 - sx;
                }
                if ty % 2 == 1 {
                    sy = h0 - 1 - sy;
                }
                let i = ((sy * w0 + sx) * 4) as usize;
                v.extend_from_slice(&f[i..i + 4]);
            }
        }
        (w, h, v)
    }

    /// Alfa en degradado, con una zona totalmente transparente y otra opaca.
    fn con_alfa(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
        let mut v = rgba.to_vec();
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4 + 3) as usize;
                v[i] = if x < w / 5 {
                    0
                } else if x > w * 4 / 5 {
                    255
                } else {
                    ((x * 255) / w) as u8 ^ (y as u8 & 7)
                };
            }
        }
        v
    }

    fn rgb(rgba: &[u8]) -> Vec<u8> {
        rgba.as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect()
    }

    fn gris(rgba: &[u8]) -> Vec<u8> {
        rgba.as_chunks::<4>()
            .0
            .iter()
            .map(|p| ((p[0] as u32 * 299 + p[1] as u32 * 587 + p[2] as u32 * 114) / 1000) as u8)
            .collect()
    }

    fn png(
        w: u32,
        h: u32,
        color: png::ColorType,
        depth: png::BitDepth,
        datos: &[u8],
        ajustes: impl FnOnce(&mut png::Encoder<&mut Vec<u8>>),
        trozos: &[(&[u8; 4], Vec<u8>)],
    ) -> Vec<u8> {
        let mut s = Vec::new();
        {
            let mut e = png::Encoder::new(&mut s, w, h);
            e.set_color(color);
            e.set_depth(depth);
            ajustes(&mut e);
            let mut wr = e.write_header().unwrap();
            for (tipo, d) in trozos {
                wr.write_chunk(png::chunk::ChunkType(**tipo), d).unwrap();
            }
            wr.write_image_data(datos).unwrap();
            wr.finish().unwrap();
        }
        s
    }

    /// Un perfil ICC con cabecera válida para libpng: longitud, 'acsp',
    /// espacio RGB, clase monitor, PCS XYZ y una etiqueta dentro del perfil.
    fn icc_falso() -> Vec<u8> {
        let mut v: Vec<u8> = (0..600u32).map(|i| (i * 7 % 251) as u8).collect();
        v[0..4].copy_from_slice(&600u32.to_be_bytes());
        v[8] = 4;
        v[12..16].copy_from_slice(b"mntr");
        v[16..20].copy_from_slice(b"RGB ");
        v[20..24].copy_from_slice(b"XYZ ");
        v[36..40].copy_from_slice(b"acsp");
        v[64..68].copy_from_slice(&0u32.to_be_bytes());
        v[128..132].copy_from_slice(&1u32.to_be_bytes());
        v[132..136].copy_from_slice(b"desc");
        v[136..140].copy_from_slice(&144u32.to_be_bytes());
        v[140..144].copy_from_slice(&100u32.to_be_bytes());
        v
    }

    fn exif_falso() -> Vec<u8> {
        let mut v = b"MM\0*\0\0\0\x08".to_vec();
        v.extend((0..90u8).map(|i| i.wrapping_mul(3)));
        v
    }

    const XMP: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF/></x:xmpmeta>"#;

    fn jpeg(
        w: u32,
        h: u32,
        rgb: &[u8],
        ajustes: impl FnOnce(&mut mozjpeg::Compress),
        marcas: bool,
    ) -> Vec<u8> {
        let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        c.set_size(w as usize, h as usize);
        c.set_quality(85.0);
        ajustes(&mut c);
        let mut c = c.start_compress(Vec::new()).unwrap();
        if marcas {
            let mut exif = b"Exif\0\0".to_vec();
            exif.extend(exif_falso());
            c.write_marker(mozjpeg::Marker::APP(1), &exif);
            let mut xmp = b"http://ns.adobe.com/xap/1.0/\0".to_vec();
            xmp.extend(XMP.as_bytes());
            c.write_marker(mozjpeg::Marker::APP(1), &xmp);
            // El ICC en dos trozos y desordenados, que es lo que tiene que
            // saber recomponer el lector.
            let icc = icc_falso();
            let (a, b) = icc.split_at(250);
            for (seq, trozo) in [(2u8, b), (1u8, a)] {
                let mut m = b"ICC_PROFILE\0".to_vec();
                m.push(seq);
                m.push(2);
                m.extend(trozo);
                c.write_marker(mozjpeg::Marker::APP(2), &m);
            }
        }
        c.write_scanlines(rgb).unwrap();
        c.finish().unwrap()
    }

    fn zlib(d: &[u8]) -> Vec<u8> {
        miniz_oxide::deflate::compress_to_vec_zlib(d, 6)
    }

    fn webp_de(w: u32, h: u32, rgba: &[u8], sin_perdida: bool) -> Vec<u8> {
        // SAFETY: búfer propio; la salida se libera con WebPFree.
        unsafe {
            let mut out = std::ptr::null_mut();
            let n = if sin_perdida {
                w::WebPEncodeLosslessRGBA(
                    rgba.as_ptr(),
                    w as i32,
                    h as i32,
                    (w * 4) as i32,
                    &mut out,
                )
            } else {
                w::WebPEncodeRGBA(
                    rgba.as_ptr(),
                    w as i32,
                    h as i32,
                    (w * 4) as i32,
                    80.0,
                    &mut out,
                )
            };
            let v = std::slice::from_raw_parts(out, n).to_vec();
            w::WebPFree(out as *mut _);
            v
        }
    }

    /// `alfa`: None sin alfa, Some(2) alfa normal, Some(1) alfa asociado
    /// (premultiplicado), que el lector de cwebp deshace.
    fn tiff_de(w: u32, h: u32, datos: &[u8], alfa: Option<u16>) -> Vec<u8> {
        use tiff::encoder::{TiffEncoder, colortype};
        use tiff::tags::Tag;
        let mut s = std::io::Cursor::new(Vec::new());
        let mut e = TiffEncoder::new(&mut s).unwrap();
        match alfa {
            Some(tipo) => {
                let mut img = e.new_image::<colortype::RGBA8>(w, h).unwrap();
                img.encoder().write_tag(Tag::ExtraSamples, tipo).unwrap();
                img.write_data(datos).unwrap();
            }
            None => e.write_image::<colortype::RGB8>(w, h, datos).unwrap(),
        }
        s.into_inner()
    }

    fn caso(nombre: &str, datos: Vec<u8>) -> Caso {
        Caso {
            nombre: nombre.into(),
            datos,
            previas: vec![],
        }
    }

    pub fn generar() -> Vec<Caso> {
        use png::{BitDepth::*, ColorType::*};
        let (w, h, f) = foto();
        let (gw, gh, g) = foto_grande();
        let alfa = con_alfa(gw, gh, &g);
        let mut v = vec![
            caso(
                "foto.webp",
                std::fs::read(raiz().join("pruebas/corpus/foto.webp")).unwrap(),
            ),
            caso("rgb.png", png(gw, gh, Rgb, Eight, &rgb(&g), |_| {}, &[])),
            caso("rgba.png", png(gw, gh, Rgba, Eight, &alfa, |_| {}, &[])),
            caso(
                "gris.png",
                png(gw, gh, Grayscale, Eight, &gris(&g), |_| {}, &[]),
            ),
            caso(
                "gris-alfa.png",
                png(
                    w,
                    h,
                    GrayscaleAlpha,
                    Eight,
                    &gris(&f)
                        .iter()
                        .enumerate()
                        .flat_map(|(i, &x)| [x, (i % 256) as u8])
                        .collect::<Vec<_>>(),
                    |_| {},
                    &[],
                ),
            ),
            caso(
                "16bits.png",
                png(
                    w,
                    h,
                    Rgb,
                    Sixteen,
                    &rgb(&f)
                        .iter()
                        .flat_map(|&x| [x, x ^ 0x5a])
                        .collect::<Vec<_>>(),
                    |_| {},
                    &[],
                ),
            ),
            caso(
                "gamma-1.png",
                png(
                    gw,
                    gh,
                    Rgb,
                    Eight,
                    &rgb(&g),
                    |e| e.set_source_gamma(png::ScaledFloat::from_scaled(100_000)),
                    &[],
                ),
            ),
            caso(
                "gamma-1-alfa.png",
                png(
                    gw,
                    gh,
                    Rgba,
                    Eight,
                    &alfa,
                    |e| e.set_source_gamma(png::ScaledFloat::from_scaled(100_000)),
                    &[],
                ),
            ),
            caso(
                "gamma-045.png",
                png(
                    w,
                    h,
                    Rgb,
                    Eight,
                    &rgb(&f),
                    |e| e.set_source_gamma(png::ScaledFloat::from_scaled(45_455)),
                    &[],
                ),
            ),
            caso(
                "srgb.png",
                png(
                    w,
                    h,
                    Rgb,
                    Eight,
                    &rgb(&f),
                    |e| e.set_source_srgb(png::SrgbRenderingIntent::Perceptual),
                    &[],
                ),
            ),
            caso("yuv.yuv", Vec::new()), // se rellena abajo
        ];

        // Paleta con transparencia.
        let paleta: Vec<u8> = (0..64u32)
            .flat_map(|i| [(i * 4) as u8, (255 - i * 3) as u8, (i * 17 % 256) as u8])
            .collect();
        let trns: Vec<u8> = (0..64u32)
            .map(|i| if i < 8 { 0 } else { (i * 4) as u8 })
            .collect();
        let indices: Vec<u8> = (0..w * h).map(|i| ((i / 3 + i / w) % 64) as u8).collect();
        v.push(caso(
            "paleta.png",
            png(
                w,
                h,
                Indexed,
                Eight,
                &indices,
                |e| {
                    e.set_palette(paleta.clone());
                    e.set_trns(trns.clone());
                },
                &[],
            ),
        ));

        // Metadatos en todas las formas que lee pngdec.
        let mut iccp = b"perfil\0\0".to_vec();
        iccp.extend(zlib(&icc_falso()));
        let mut crudo = format!("\nexif\n{:8}\n", exif_falso().len());
        for (i, b) in exif_falso().iter().enumerate() {
            crudo += &format!("{b:02x}");
            if i % 36 == 35 {
                crudo.push('\n');
            }
        }
        crudo.push('\n');
        let mut texto_crudo = b"Raw profile type exif\0".to_vec();
        texto_crudo.extend(crudo.as_bytes());
        v.push(caso(
            "metadatos.png",
            png(
                w,
                h,
                Rgb,
                Eight,
                &rgb(&f),
                |e| {
                    e.add_itxt_chunk("XML:com.adobe.xmp".into(), XMP.into())
                        .unwrap()
                },
                &[(b"iCCP", iccp), (b"tEXt", texto_crudo)],
            ),
        ));
        // El mismo, con el perfil roto: libpng lo tira y cwebp no lo copia.
        let mut iccp_roto = b"roto\0\0".to_vec();
        let mut roto = icc_falso();
        roto[36..40].copy_from_slice(b"xxxx");
        iccp_roto.extend(zlib(&roto));
        v.push(caso(
            "icc-roto.png",
            png(w, h, Rgb, Eight, &rgb(&f), |_| {}, &[(b"iCCP", iccp_roto)]),
        ));
        v.push(caso(
            "exif.png",
            png(
                w,
                h,
                Rgba,
                Eight,
                &con_alfa(w, h, &f),
                |_| {},
                &[(b"eXIf", exif_falso())],
            ),
        ));

        let (rg, rf) = (rgb(&g), rgb(&f));
        v.push(caso("foto.jpg", jpeg(gw, gh, &rg, |_| {}, false)));
        v.push(caso("metadatos.jpg", jpeg(w, h, &rf, |_| {}, true)));
        v.push(caso(
            "444.jpg",
            jpeg(
                w,
                h,
                &rf,
                |c| c.set_chroma_sampling_pixel_sizes((1, 1), (1, 1)),
                false,
            ),
        ));
        v.push(caso(
            "progresivo.jpg",
            jpeg(gw, gh, &rg, |c| c.set_progressive_mode(), false),
        ));
        {
            let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_GRAYSCALE);
            c.set_size(w as usize, h as usize);
            let mut c = c.start_compress(Vec::new()).unwrap();
            c.write_scanlines(&gris(&f)).unwrap();
            v.push(caso("gris.jpg", c.finish().unwrap()));
        }

        v.push(caso("rgb.tif", tiff_de(gw, gh, &rg, None)));
        let alfa_chica = con_alfa(w, h, &f);
        v.push(caso("rgba.tif", tiff_de(w, h, &alfa_chica, Some(2))));
        let premultiplicada: Vec<u8> = alfa_chica
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let a = p[3] as u32;
                [
                    (p[0] as u32 * a / 255) as u8,
                    (p[1] as u32 * a / 255) as u8,
                    (p[2] as u32 * a / 255) as u8,
                    p[3],
                ]
            })
            .collect();
        v.push(caso(
            "rgba-asociado.tif",
            tiff_de(w, h, &premultiplicada, Some(1)),
        ));

        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        ppm.extend(&rf);
        v.push(caso("foto.ppm", ppm));
        let mut pgm = format!("P5\n{w} {h}\n255\n").into_bytes();
        pgm.extend(gris(&f));
        v.push(caso("gris.pgm", pgm));
        let mut pam =
            format!("P7\nWIDTH {w}\nHEIGHT {h}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n")
                .into_bytes();
        pam.extend(con_alfa(w, h, &f));
        v.push(caso("alfa.pam", pam));

        v.push(caso("perdida.webp", webp_de(gw, gh, &g, false)));
        v.push(caso("alfa-perdida.webp", webp_de(gw, gh, &alfa, false)));
        v.push(caso(
            "alfa-sin-perdida.webp",
            webp_de(w, h, &con_alfa(w, h, &f), true),
        ));

        // YUV 4:2:0 crudo, de tamaño impar.
        let (yw, yh) = (w - 1, h - 3);
        let yuv: Vec<u8> = (0..(yw * yh + 2 * yw.div_ceil(2) * yh.div_ceil(2)))
            .map(|i| (i * 13 % 251) as u8)
            .collect();
        let i = v.iter().position(|c| c.nombre == "yuv.yuv").unwrap();
        v[i] = Caso {
            nombre: "yuv.yuv".into(),
            datos: yuv,
            previas: vec!["-s".into(), yw.to_string(), yh.to_string()],
        };
        v
    }
}
