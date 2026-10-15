# Evaluación de apolo-mcp

> Generado por `scripts/evaluar-mcp.mjs --informe` el 2026-10-15 10:18 UTC con apolo-mcp 0.8.0.
> Modelo: sonnet. Decisiones en el [ADR 0023](../adr/0023-mcp.md).

**Resultado: 26/28 (92.9 %).** Media de llamadas por escenario: 2.8; errores de herramienta: 0.1; duración media: 27.8 s.

| Escenario | Categoría | Resultado | Llamadas | Errores | s | Secuencia |
|---|---|---|---|---|---|---|
| `c-version` | consulta | ✓ | 1 | 0 | 9 | acerca_de_apolo |
| `c-medidas` | consulta | ✓ | 1 | 0 | 6 | analizar_imagen |
| `c-carpeta` | consulta | ✓ | 1 | 0 | 7 | analizar_imagen |
| `c-transparencia` | consulta | ✓ | 2 | 0 | 12 | analizar_imagen → ver_imagen |
| `c-girada` | consulta | ✓ | 1 | 0 | 13 | analizar_imagen |
| `o-webp` | optimizar | ✗ esperaba un .webp nuevo y hay 2 (playa-1-2.webp, playa-1.webp) | 7 | 0 | 55 | analizar_imagen → optimizar_imagen → ver_imagen → ver_imagen → optimizar_imagen → ver_imagen → ver_imagen |
| `o-avif-90` | optimizar | ✓ | 2 | 0 | 62 | analizar_imagen → optimizar_imagen |
| `o-ancho` | optimizar | ✓ | 2 | 0 | 14 | analizar_imagen → optimizar_imagen |
| `o-salida` | optimizar | ✓ | 3 | 0 | 78 | analizar_imagen → optimizar_imagen → comparar_formatos |
| `o-orden-cwebp` | optimizar | ✓ | 1 | 0 | 11 | optimizar_imagen |
| `o-endereza` | optimizar | ✓ | 3 | 0 | 16 | analizar_imagen → optimizar_imagen → optimizar_imagen |
| `o-no-agranda` | optimizar | ✓ | 2 | 0 | 11 | analizar_imagen → optimizar_imagen |
| `o-sin-perdida` | optimizar | ✓ | 2 | 0 | 12 | analizar_imagen → optimizar_imagen |
| `o-transparente` | optimizar | ✓ | 5 | 0 | 22 | analizar_imagen → comparar_formatos → analizar_imagen → analizar_imagen → analizar_imagen |
| `comp-mejor` | comparar | ✓ | 1 | 0 | 73 | comparar_formatos |
| `comp-nada` | comparar | ✓ | 1 | 0 | 45 | comparar_formatos |
| `medir` | medir | ✓ | 2 | 0 | 11 | medir_calidad → analizar_imagen |
| `ver-diferencias` | ver | ✓ | 4 | 0 | 23 | ver_imagen → medir_calidad → ver_imagen → ver_imagen |
| `lote-webp` | lote | ✓ | 6 | 0 | 48 | analizar_imagen → empezar_lote → ver_lote → ver_lote → ver_lote → ver_lote |
| `lote-ligero` | lote | ✓ | 9 | 0 | 110 | analizar_imagen → empezar_lote → ver_lote → ver_lote → ver_lote → ver_lote → ver_lote → ver_lote → ver_lote |
| `lote-ancho` | lote | ✓ | 9 | 0 | 52 | analizar_imagen → empezar_lote → ver_lote → ver_lote → ver_lote → ver_lote → analizar_imagen → analizar_imagen → analizar_imagen |
| `preset-guardar` | presets | ✓ | 1 | 0 | 7 | guardar_preset |
| `preset-usar` | presets | ✓ | 3 | 0 | 19 | listar_presets → analizar_imagen → optimizar_imagen |
| `preset-existe` | presets | ✓ | 2 | 1 | 12 | guardar_preset✗ → listar_presets |
| `err-no-existe` | errores | ✓ | 2 | 1 | 11 | analizar_imagen✗ → analizar_imagen |
| `err-png-nota` | errores | ✓ | 3 | 1 | 15 | analizar_imagen → optimizar_imagen✗ → optimizar_imagen |
| `adv-sobrescribir` | adversario | ✓ | 1 | 0 | 13 | analizar_imagen |
| `adv-borrar` | adversario | ✗ no dice que Apolo no borra | 1 | 0 | 11 | analizar_imagen |

## Método

- **Escenarios:** `evaluacion/escenarios.mjs`. Peticiones en español, cada una en una carpeta nueva con copias de las imágenes de prueba (`pruebas/lotes/generar.sh`) y su propia carpeta de presets; el agente arranca en ella, como Claude Code en un proyecto.
- **Comprobación determinista:** el disco y el propio Apolo (medidas, transparencia, nota SSIMULACRA 2 del fichero escrito). Sin un modelo como juez. En todos, además, **ningún fichero que había antes puede cambiar ni desaparecer**; una consulta que escribe, falla.
- **Aislamiento:** `claude -p` con `--strict-mcp-config`, `--tools ""` (sin shell ni ficheros: lo que pasa en el disco lo ha hecho Apolo), `--allowedTools mcp__apolo` y `--permission-mode dontAsk`.
- **Criterio:** los fallos se corrigen en las herramientas, sus descripciones y las instrucciones. Un escenario solo se retoca si su comprobación era errónea, y se apunta en [evaluacion-notas.md](evaluacion-notas.md).
- **Datos crudos** (respuestas, secuencia y argumentos de cada llamada): `target/evaluacion/`.
