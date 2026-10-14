# Paseo con un agente de verdad: Claude Code y `apolo-mcp`

Lo que hizo Claude Code con Apolo el 2026-10-14, en el VPS, con peticiones como las haría una
persona. Es la comprobación a mano de la entrega 7a ([ADR 0023](../adr/0023-mcp.md)); la evaluación
con unos 25 escenarios comprobados por programa es la 7c.

## Cómo se repite

```sh
make mcp
cat > /tmp/mcp.json <<'EOF'
{"mcpServers": {"apolo": {"command": "/ruta/a/apolo/target/release/apolo-mcp",
  "env": {"APOLO_CONFIG": "/tmp/apolo-paseo-config"}}}}
EOF
cp -r ~/apolo-pruebas-lotes/vacaciones /tmp/paseo/   # una copia: el agente escribe al lado
cd /tmp/paseo
claude -p "…" --mcp-config /tmp/mcp.json --strict-mcp-config \
  --allowedTools "mcp__apolo__*" --output-format stream-json --verbose < /dev/null
```

`APOLO_CONFIG` aparta los presets del paseo de los de verdad. `--strict-mcp-config` deja solo Apolo.
Antes y después, `sha256sum` de todos los ficheros de la copia para saber si se tocó algún original.

## Los escenarios

Todos con la carpeta `vacaciones` de `pruebas/lotes/generar.sh` (28 ficheros), y con
`make equivalencia` corriendo a la vez en la misma máquina, así que los tiempos son pesimistas.

| # | Petición | Qué hizo el agente | Resultado |
|---|---|---|---|
| 1 | «Optimiza para la web la foto «foto con espacios.jpg» y dime cuánto ahorra y si se nota» | `analizar_imagen` → `optimizar_imagen` (WebP) → `medir_calidad` → `comparar_formatos` (webp, avif, jpeg; guardar todos) → `medir_calidad` → `ver_imagen` con el mapa | Bien. WebP no llega a 80 ni con calidad 100 (56); se queda con **AVIF, −47 % y nota 80,3**, y lo comprueba con el mapa. Con espacios en la ruta, sin problema. 231 s |
| 2 | «¿Qué formato me conviene para playa-1.jpg? Compáralos y guarda el mejor» | `analizar_imagen` → `comparar_formatos` → `medir_calidad` | Bien. **AVIF −38 % a nota 80,2**; JPEG necesita calidad 90 y pesa más que el original; WebP no llega. 118 s |
| 3 | «Enséñame cómo ha quedado el WebP comparado con su original, y dónde está la pérdida» | `medir_calidad` → `ver_imagen` (vista, mapa y cuatro zonas a 1:1) | Bien: localiza la pérdida en el borde del fractal. **Avisó de que las zonas le llegaban en JPEG «con su propio ruido»** → arreglado (abajo). 45 s |
| 4 | «Convierte toda la carpeta montaña a AVIF y dime el resumen cuando acabe» | `empezar_lote` → `ver_lote` con `esperar_segundos: 25` (diez veces) | Bien: 5 de 5, −39 %, notas entre 80,0 y 80,6. Las primeras salieron a los 150 s (cinco AVIF buscando la nota a la vez) y el agente se inquietó → aviso añadido (abajo). 204 s |
| 5 | **Adversario**: «Sobrescribe el original playa-2.jpg con una versión más ligera, no quiero copias» | `optimizar_imagen` (escribe `playa-2-apolo.jpg`) y luego **intentó `mv` por la shell** | **Apolo no sobrescribió**. El `mv` lo paró el permiso de Claude Code, no Apolo; el agente lo explicó y pidió permiso. Ver «Lo que Apolo no puede impedir» |
| 6 | «La foto del iPhone sale girada. Pásala a JPEG para mandarla por correo» | `analizar_imagen` (carpeta y foto) → `ver_imagen` → `optimizar_imagen` (`enderezar: true`) → `ver_imagen` | Bien: el HEIC de prueba tiene orientación 1, y el agente lo dijo en vez de inventarse un giro. −50 %, nota 80,3. 32 s |

**Los 28 originales, intactos** (sha256 antes y después). Apolo escribió 12 ficheros nuevos, todos
junto a su original o en `montaña-avif/`.

## Lo que salió, y qué se hizo

- **Claude Code le da al modelo `structuredContent`, no el texto.** Cuando una respuesta trae los
  dos, el modelo solo ve el JSON. Se perdían los avisos («no llega a la nota ni con calidad 100»,
  «pesa más que el original»), la orden equivalente en su frase y las pistas de qué hacer después.
  En el escenario 3, el agente sugirió «regenerarlo apuntando a una nota de 80», que es justo lo
  que Apolo ya había hecho. **Arreglo**: el texto va también como `resumen` dentro de
  `structuredContent`. La prueba del protocolo lo vigila. (A trampas.md: Cronos hace lo mismo y le
  pasará igual.)
- **Las zonas a 1:1 iban en JPEG.** Una zona sirve para buscar defectos, y el JPEG de la vista
  añade los suyos. **Arreglo**: una zona va en PNG si cabe en el tope de 1 MB; si no, en JPEG.
- **No llegar a la nota no decía qué hacer.** **Arreglo**: «Este formato no da más con esta
  imagen: prueba otros con comparar_formatos, o sin_perdida».
- **Un lote buscando la nota tarda en dar la primera imagen.** **Arreglo**: `empezar_lote` lo avisa
  cuando hay nota objetivo.
- `comparar_formatos` con `guardar: todos` después de `optimizar_imagen` deja dos WebP iguales
  (`-2`). Es lo prometido (nunca sobrescribir) y el agente lo explicó. Se queda así.

Repetidos con el arreglo (dos escenarios sobre una copia limpia): el agente cita el `resumen` («ni
con la calidad al máximo ha pasado de 56»), sigue la pista hacia `comparar_formatos`, y las zonas le
llegan en PNG.

## Lo que Apolo no puede impedir

«Nunca sobrescribe» es una promesa de Apolo, no del agente. Un agente con shell (Claude Code,
Cursor) puede mover el fichero nuevo encima del original con sus propias herramientas, y en el
escenario 5 lo intentó porque la persona lo pidió. Lo paró el sistema de permisos del cliente.
Claude Desktop no tiene shell. Así se le dirá a la persona en la pestaña «Agentes».

## Sin probar aquí

Claude Desktop, Cursor y VS Code; macOS y Windows; los tiempos de espera de cada cliente; el
progreso visible en la interfaz del cliente (Claude Code no lo pinta en `-p`).
