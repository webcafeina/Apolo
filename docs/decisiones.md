# Decisiones

Última actualización: **2026-10-06**

Una ficha por decisión no trivial, en [adr/](adr/). Las que se superan **no se borran**: se marcan y
se quedan, porque saber qué se pensaba antes explica por qué el código es como es.

Cada ficha lleva Contexto, Decisión, Alternativas descartadas, Consecuencias y Verificación. La
última sección es la que más se agradece meses después: dice qué se comprobó de verdad y qué no.

Las diez primeras salieron de la sesión de planteamiento del 2026-10-06, en cuatro rondas de preguntas
con el cliente.

| # | Decisión | Fecha | Estado |
|---|---|---|---|
| [0001](adr/0001-tauri-y-rust.md) | Tauri 2 y Rust; la interfaz en React como Esfinge | 2026-10-06 | aceptada |
| [0002](adr/0002-libwebp-enlazada.md) | libwebp enlazada, y la misma salida que `cwebp` byte a byte | 2026-10-06 | aceptada |
| [0003](adr/0003-alcance-squoosh.md) | WebP entero, y el resto de Squoosh | 2026-10-06 | aceptada |
| [0004](adr/0004-estudio-y-lotes.md) | Estudio y lotes, con los presets en el centro | 2026-10-06 | aceptada |
| [0005](adr/0005-extras.md) | Redimensionar, reducir paleta, la CLI y el mapa de diferencias | 2026-10-06 | aceptada |
| [0006](adr/0006-gplv3-y-repositorio-publico.md) | Repositorio público con licencia GPLv3 | 2026-10-06 | aceptada |
| [0007](adr/0007-espanol-con-i18n.md) | En español, con i18n desde el primer día | 2026-10-06 | aceptada |
| [0008](adr/0008-aspecto-del-sistema.md) | Apariencia del sistema; tokens desde Rust con prueba de contraste | 2026-10-06 | aceptada |
| [0009](adr/0009-sin-firmar.md) | Sin firmar, por ahora | 2026-10-06 | aceptada |
| [0010](adr/0010-plataformas.md) | Seis objetivos, con ARM | 2026-10-06 | aceptada |
| [0011](adr/0011-leer-como-cwebp.md) | Leer las imágenes como cwebp, con sus manías; sin orientación EXIF | 2026-10-06 | aceptada · matizada por la 0012 |
| [0012](adr/0012-enderezar-como-opcion.md) | Enderezar según la orientación EXIF, como opción apagada por defecto | 2026-10-06 | aceptada · hecha en la entrega 2 |
| [0013](adr/0013-el-estudio-por-dentro.md) | El Estudio: un servicio con dos transportes (Tauri y HTTP), píxeles crudos en canvas, generaciones por imagen | 2026-10-06 | aceptada |
| [0014](adr/0014-versiones-y-publicacion.md) | Versiones 0.x como pre-release en borrador, con la CLI y las sumas en la Release | 2026-10-06 | aceptada |
| [0015](adr/0015-identidad-nativa-con-marca.md) | Nativa con marca, como Esfinge hoy; entrega «Identidad» antes de Lotes; sin licencia en el `.dmg` | 2026-10-06 | aceptada · sin hacer |
