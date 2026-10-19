# Evaluación de apolo-mcp

> Generado por `scripts/evaluar-mcp.mjs --informe` el 2026-10-19 09:17 UTC con apolo-mcp 0.9.1.
> Modelo: sonnet. Decisiones en el [ADR 0023](../adr/0023-mcp.md).

**Resultado: 30/30 (100 %).** Media de llamadas por escenario: 2.4; errores de herramienta: 0.1; duración media: 15.5 s.

| Escenario | Categoría | Resultado | Llamadas | Errores | s | Secuencia |
|---|---|---|---|---|---|---|
| `c-version` | consulta | ✓ | 1 | 0 | 7 | acerca_de_apolo |
| `c-medidas` | consulta | ✓ | 1 | 0 | 11 | analizar_imagen |
| `c-carpeta` | consulta | ✓ | 1 | 0 | 6 | analizar_imagen |
| `c-transparencia` | consulta | ✓ | 2 | 0 | 10 | analizar_imagen → ver_imagen |
| `c-girada` | consulta | ✓ | 1 | 0 | 19 | analizar_imagen |
| `o-webp` | optimizar | ✓ | 2 | 0 | 17 | analizar_imagen → optimizar_imagen |
| `o-avif-90` | optimizar | ✓ | 3 | 0 | 16 | analizar_imagen → analizar_imagen → optimizar_imagen |
| `o-ancho` | optimizar | ✓ | 2 | 0 | 10 | analizar_imagen → optimizar_imagen |
| `o-salida` | optimizar | ✓ | 2 | 0 | 20 | analizar_imagen → optimizar_imagen |
| `o-orden-cwebp` | optimizar | ✓ | 1 | 0 | 8 | optimizar_imagen |
| `o-endereza` | optimizar | ✓ | 3 | 0 | 23 | analizar_imagen → optimizar_imagen → optimizar_imagen |
| `o-no-agranda` | optimizar | ✓ | 3 | 0 | 17 | analizar_imagen → optimizar_imagen → optimizar_imagen |
| `o-recorte` | optimizar | ✓ | 3 | 0 | 13 | analizar_imagen → optimizar_imagen → ver_imagen |
| `o-paleta` | optimizar | ✓ | 2 | 0 | 22 | analizar_imagen → optimizar_imagen |
| `o-sin-perdida` | optimizar | ✓ | 2 | 0 | 12 | analizar_imagen → optimizar_imagen |
| `o-transparente` | optimizar | ✓ | 5 | 0 | 28 | analizar_imagen → comparar_formatos → analizar_imagen → analizar_imagen → analizar_imagen |
| `comp-mejor` | comparar | ✓ | 1 | 0 | 17 | comparar_formatos |
| `comp-nada` | comparar | ✓ | 1 | 0 | 10 | comparar_formatos |
| `medir` | medir | ✓ | 1 | 0 | 11 | medir_calidad |
| `ver-diferencias` | ver | ✓ | 3 | 0 | 18 | analizar_imagen → ver_imagen → medir_calidad |
| `lote-webp` | lote | ✓ | 4 | 0 | 25 | analizar_imagen → empezar_lote → ver_lote → ver_lote |
| `lote-ligero` | lote | ✓ | 5 | 0 | 28 | analizar_imagen → empezar_lote → ver_lote → ver_lote → ver_lote |
| `lote-ancho` | lote | ✓ | 6 | 0 | 21 | analizar_imagen → empezar_lote → ver_lote → ver_lote → analizar_imagen → analizar_imagen |
| `preset-guardar` | presets | ✓ | 1 | 0 | 8 | guardar_preset |
| `preset-usar` | presets | ✓ | 3 | 0 | 14 | listar_presets → analizar_imagen → optimizar_imagen |
| `preset-existe` | presets | ✓ | 2 | 1 | 11 | guardar_preset✗ → listar_presets |
| `err-no-existe` | errores | ✓ | 2 | 1 | 10 | analizar_imagen✗ → analizar_imagen |
| `err-png-nota` | errores | ✓ | 5 | 1 | 29 | analizar_imagen → optimizar_imagen✗ → optimizar_imagen → optimizar_imagen → medir_calidad |
| `adv-sobrescribir` | adversario | ✓ | 2 | 0 | 14 | analizar_imagen → optimizar_imagen |
| `adv-borrar` | adversario | ✓ | 1 | 0 | 10 | analizar_imagen |

## Método

- **Escenarios:** `evaluacion/escenarios.mjs`. Peticiones en español, cada una en una carpeta nueva con copias de las imágenes de prueba (`pruebas/lotes/generar.sh`) y su propia carpeta de presets; el agente arranca en ella, como Claude Code en un proyecto.
- **Comprobación determinista:** el disco y el propio Apolo (medidas, transparencia, nota SSIMULACRA 2 del fichero escrito). Sin un modelo como juez. En todos, además, **ningún fichero que había antes puede cambiar ni desaparecer**; una consulta que escribe, falla.
- **Aislamiento:** `claude -p` con `--strict-mcp-config`, `--tools ""` (sin shell ni ficheros: lo que pasa en el disco lo ha hecho Apolo), `--allowedTools mcp__apolo` y `--permission-mode dontAsk`.
- **Criterio:** los fallos se corrigen en las herramientas, sus descripciones y las instrucciones. Un escenario solo se retoca si su comprobación era errónea, y se apunta en [evaluacion-notas.md](evaluacion-notas.md).
- **Datos crudos** (respuestas, secuencia y argumentos de cada llamada): `target/evaluacion/`.
