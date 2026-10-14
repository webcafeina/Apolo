//! Las rutas que llegan de un agente (ADR 0023): con `~`, como `file://`,
//! entre comillas, con barras de Windows. Siempre salen absolutas.

use std::path::{Path, PathBuf};

use crate::errores::{ErrorMcp, Hecho};

/// Una ruta de un agente, absoluta. Una relativa vale si el directorio de
/// trabajo es de verdad uno (Claude Code arranca en la carpeta del proyecto);
/// con `/` (Claude Desktop en macOS) no: no se sabe a qué se refiere.
pub fn resolver(texto: &str) -> Hecho<PathBuf> {
    resolver_con(
        texto,
        dirs::home_dir().as_deref(),
        std::env::current_dir().ok().as_deref(),
    )
}

pub fn resolver_con(texto: &str, casa: Option<&Path>, actual: Option<&Path>) -> Hecho<PathBuf> {
    let mut t = texto.trim();
    for comilla in ['"', '\'', '`'] {
        if t.len() >= 2 && t.starts_with(comilla) && t.ends_with(comilla) {
            t = &t[1..t.len() - 1];
        }
    }
    if t.is_empty() {
        return Err(ErrorMcp::nuevo("ruta_vacia", "Falta la ruta.")
            .con_pista("Pasa la ruta absoluta del fichero o la carpeta (se admite ~)."));
    }
    let mut ruta: String = match t.strip_prefix("file://") {
        Some(resto) => decodificar_url(resto.strip_prefix("localhost").unwrap_or(resto)),
        None => t.to_string(),
    };
    // file:///C:/… en Windows: sobra la barra de delante.
    if cfg!(windows) && ruta.len() > 2 && ruta.starts_with('/') && ruta.as_bytes()[2] == b':' {
        ruta.remove(0);
    }
    if ruta == "~" || ruta.starts_with("~/") || ruta.starts_with("~\\") {
        let Some(casa) = casa else {
            return Err(ErrorMcp::nuevo(
                "ruta_relativa",
                "No se sabe cuál es la carpeta personal.",
            )
            .con_pista("Usa la ruta absoluta, sin ~."));
        };
        ruta = format!("{}{}", casa.display(), &ruta[1..]);
    }
    let p = PathBuf::from(ruta);
    if p.is_absolute() {
        return Ok(p);
    }
    match actual {
        Some(a) if a.parent().is_some() => Ok(a.join(p)),
        _ => Err(ErrorMcp::nuevo(
            "ruta_relativa",
            format!("«{texto}» es una ruta relativa, y aquí no se sabe a qué carpeta se refiere."),
        )
        .con_pista("Usa la ruta absoluta (se admite ~ para la carpeta personal).")),
    }
}

/// %20 y compañía, de una URL `file://`.
fn decodificar_url(s: &str) -> String {
    let b = s.as_bytes();
    let mut v = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(n) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            v.push(n);
            i += 3;
            continue;
        }
        v.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&v).into_owned()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn las_formas_de_una_ruta() {
        let casa = Path::new("/home/ana");
        let aqui = Path::new("/home/ana/proyecto");
        let r = |t: &str| resolver_con(t, Some(casa), Some(aqui));
        assert_eq!(
            r("~/fotos/a.jpg").unwrap(),
            Path::new("/home/ana/fotos/a.jpg")
        );
        assert_eq!(
            r("\"/tmp/con espacios.png\"").unwrap(),
            Path::new("/tmp/con espacios.png")
        );
        assert_eq!(
            r("file:///tmp/foto%20uno.png").unwrap(),
            Path::new("/tmp/foto uno.png")
        );
        assert_eq!(
            r("fotos/a.jpg").unwrap(),
            Path::new("/home/ana/proyecto/fotos/a.jpg")
        );
        assert_eq!(r("  /abs/b.png  ").unwrap(), Path::new("/abs/b.png"));
    }

    #[test]
    fn una_relativa_sin_carpeta_de_trabajo_es_un_error_con_pista() {
        let e = resolver_con("a.jpg", None, Some(Path::new("/"))).unwrap_err();
        assert_eq!(e.error, "ruta_relativa");
        assert!(e.pista.unwrap().contains("absoluta"));
        assert_eq!(
            resolver_con("", None, None).unwrap_err().error,
            "ruta_vacia"
        );
    }
}
