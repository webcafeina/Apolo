//! Las rutas que llegan de un agente (ADR 0023): con `~`, como `file://`,
//! entre comillas, con barras de Windows. Siempre salen absolutas.

use std::path::{Path, PathBuf};

use apolo_nucleo::{tr, trs};

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
        return Err(
            ErrorMcp::nuevo("ruta_vacia", trs!("Falta la ruta.", "The path is missing."))
                .con_pista(trs!(
                    "Pasa la ruta absoluta del fichero o la carpeta (se admite ~).",
                    "Pass the absolute path of the file or folder (~ is accepted)."
                )),
        );
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
                trs!(
                    "No se sabe cuál es la carpeta personal.",
                    "The home folder is unknown."
                ),
            )
            .con_pista(trs!(
                "Usa la ruta absoluta, sin ~.",
                "Use the absolute path, without ~."
            )));
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
            tr!(
                "«{texto}» es una ruta relativa, y aquí no se sabe a qué carpeta se refiere.",
                "“{texto}” is a relative path, and here it's unknown which folder it refers to."
            ),
        )
        .con_pista(trs!(
            "Usa la ruta absoluta (se admite ~ para la carpeta personal).",
            "Use the absolute path (~ is accepted for the home folder)."
        ))),
    }
}

/// Una ruta para enseñar, con `~` en lugar de la carpeta personal. Así no sale
/// el nombre de la cuenta del sistema, que puede no ser el de la persona y hacía
/// sospechar al agente («Apolo corre con la cuenta alvaro…»; prueba guiada de la
/// v0.8.2). `resolver` acepta `~`, así que la ruta sigue valiendo.
pub fn con_casa(ruta: &Path) -> String {
    con_casa_en(ruta, dirs::home_dir().as_deref())
}

pub fn con_casa_en(ruta: &Path, casa: Option<&Path>) -> String {
    match casa.and_then(|c| ruta.strip_prefix(c).ok()) {
        Some(resto) if resto.as_os_str().is_empty() => "~".into(),
        Some(resto) => format!("~{}{}", std::path::MAIN_SEPARATOR, resto.display()),
        None => ruta.display().to_string(),
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
    fn la_carpeta_personal_se_enseña_como_virgulilla() {
        let casa = Path::new("/Users/alvaro");
        assert_eq!(
            con_casa_en(
                Path::new("/Users/alvaro/Library/Application Support/Apolo/presets"),
                Some(casa)
            ),
            "~/Library/Application Support/Apolo/presets"
        );
        assert_eq!(
            con_casa_en(Path::new("/Applications/Apolo.app"), Some(casa)),
            "/Applications/Apolo.app"
        );
        assert_eq!(con_casa_en(Path::new("/Users/alvaro"), Some(casa)), "~");
        // Y se puede volver a usar.
        assert_eq!(
            resolver_con("~/Library/x", Some(casa), None).unwrap(),
            Path::new("/Users/alvaro/Library/x")
        );
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
