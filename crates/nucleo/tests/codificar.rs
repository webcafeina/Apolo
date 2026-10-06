//! Lo básico, sin cwebp de referencia: que sin pérdida devuelva los mismos
//! píxeles, que con pérdida salga un WebP válido del tamaño pedido, y que la
//! cancelación cancele.

use apolo_nucleo::entrada::{self, Lectura};
use apolo_nucleo::webp::{self, Extras, OpcionesWebp, Redimension};
use apolo_nucleo::{Error, cwebp};
use libwebp_sys as w;

fn foto() -> entrada::Imagen {
    let ruta =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus/foto.webp");
    entrada::leer(&std::fs::read(ruta).unwrap(), Lectura::default()).unwrap()
}

fn decodificar(datos: &[u8]) -> (i32, i32, Vec<u8>) {
    let (mut a, mut h) = (0, 0);
    // SAFETY: búfer propio; la salida se libera con WebPFree.
    unsafe {
        let p = w::WebPDecodeRGBA(datos.as_ptr(), datos.len(), &mut a, &mut h);
        assert!(!p.is_null(), "no es un WebP válido");
        let v = std::slice::from_raw_parts(p, (a * h * 4) as usize).to_vec();
        w::WebPFree(p as *mut _);
        (a, h, v)
    }
}

#[test]
fn sin_perdida_conserva_los_pixeles() {
    let img = foto();
    let original = webp::codificar(
        &img,
        &cwebp::leer(&["-lossless"]).unwrap().opciones,
        Extras::default(),
        None,
    )
    .unwrap();
    let (_, _, a) = decodificar(&original.datos);
    let otra = webp::codificar(
        &img,
        &cwebp::leer(&["-z", "9"]).unwrap().opciones,
        Extras::default(),
        None,
    )
    .unwrap();
    let (_, _, b) = decodificar(&otra.datos);
    assert_eq!(a, b);
}

#[test]
fn con_perdida_y_redimension() {
    let img = foto();
    let op = OpcionesWebp {
        redimension: Some(Redimension { ancho: 50, alto: 0 }),
        ..Default::default()
    };
    let r = webp::codificar(&img, &op, Extras::default(), None).unwrap();
    let (a, h, _) = decodificar(&r.datos);
    assert_eq!((a, h), (50, 50));
    assert_eq!(r.estadisticas.bytes as usize, r.datos.len());
}

#[test]
fn cancelar() {
    let img = foto();
    let mut cancelar = |_p: i32| false;
    let r = webp::codificar(
        &img,
        &OpcionesWebp::default(),
        Extras::default(),
        Some(&mut cancelar),
    );
    assert!(matches!(r, Err(Error::Cancelado)));
}

#[test]
fn las_opciones_se_guardan_en_json() {
    let op = cwebp::leer(&["-preset", "photo", "-q", "82", "-metadata", "icc"])
        .unwrap()
        .opciones;
    let json = serde_json::to_string(&op).unwrap();
    let vuelta: OpcionesWebp = serde_json::from_str(&json).unwrap();
    assert_eq!(op, vuelta);
    // Un preset a medias en JSON se completa con los valores por defecto.
    let parcial: OpcionesWebp = serde_json::from_str(r#"{"calidad": 60}"#).unwrap();
    assert_eq!(parcial.calidad, 60.0);
    assert_eq!(parcial.metodo, OpcionesWebp::default().metodo);
}

#[test]
fn enderezar_gira_y_deja_el_exif_a_1() {
    let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../pruebas/corpus/orientacion-6.png");
    let datos = std::fs::read(ruta).unwrap();
    // Sin enderezar, como cwebp: 64×32.
    let o = cwebp::leer(&["-metadata", "exif"]).unwrap();
    let r = cwebp::ejecutar(&o, &datos, None).unwrap();
    assert_eq!((r.ancho, r.alto), (64, 32));
    // Enderezada: 32×64, la mitad roja arriba y la azul abajo, y el EXIF que
    // se copia dice orientación 1.
    let o = cwebp::leer(&["-metadata", "exif", "-apolo_enderezar", "-lossless"]).unwrap();
    let r = cwebp::ejecutar(&o, &datos, None).unwrap();
    assert_eq!((r.ancho, r.alto), (32, 64));
    let (_, _, px) = decodificar(&r.datos);
    let arriba = &px[..4];
    let abajo = &px[px.len() - 4..];
    assert!(
        arriba[0] > 200 && arriba[2] < 60,
        "arriba tiene que ser rojo: {arriba:?}"
    );
    assert!(
        abajo[2] > 200 && abajo[0] < 60,
        "abajo tiene que ser azul: {abajo:?}"
    );
    let img = entrada::leer(&r.datos, Lectura::default()).unwrap();
    assert_eq!(
        apolo_nucleo::orientacion::leer(img.metadatos.exif.as_deref()),
        1
    );
    assert!(!cwebp::es_equivalente(&o.opciones));
}
