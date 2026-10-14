//! `apartar_salida` (ADR 0023): después de llamarla, lo que una biblioteca
//! de C escriba con printf va a stderr, y por la copia que devuelve solo sale
//! lo que se escriba en ella. La prueba se relanza a sí misma para tener una
//! salida estándar propia que mirar.

use std::io::Write;
use std::process::Command;

use apolo_avifjxl::{Herramienta, apartar_salida, ejecutar};

fn main() {
    if std::env::var_os("APOLO_PRUEBA_HIJA").is_some() {
        let mut protocolo = apartar_salida().unwrap();
        // avifenc imprime su versión con printf, por el fd 1.
        ejecutar(Herramienta::Avifenc, &["--version".into()]);
        println!("y un println! de Rust");
        protocolo.write_all(b"{\"jsonrpc\":\"2.0\"}\n").unwrap();
        return;
    }
    let s = Command::new(std::env::current_exe().unwrap())
        .env("APOLO_PRUEBA_HIJA", "1")
        .output()
        .unwrap();
    assert!(s.status.success(), "{s:?}");
    let salida = String::from_utf8_lossy(&s.stdout);
    let errores = String::from_utf8_lossy(&s.stderr);
    assert_eq!(
        salida, "{\"jsonrpc\":\"2.0\"}\n",
        "por stdout solo el protocolo"
    );
    assert!(
        errores.contains("Version"),
        "avifenc escribe en stderr: {errores:?}"
    );
    assert!(errores.contains("println!"), "{errores:?}");
    println!("apartar_salida: bien");
}
