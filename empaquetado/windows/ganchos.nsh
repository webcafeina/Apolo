; Ganchos del instalador NSIS (ADR 0023).
;
; Con un agente abierto (Claude Desktop, Cursor…), apolo-mcp.exe está en marcha y
; Windows no deja reemplazarlo: la actualización fallaría o dejaría el de antes.
; Se cierra antes de copiar. El agente pierde la conexión un momento y, al volver
; a arrancarlo, ya es el nuevo. Cierra todos los apolo-mcp.exe (también el de una
; copia de la CLI): es el mismo programa, y cada agente lo vuelve a lanzar.

!macro NSIS_HOOK_PREINSTALL
  nsExec::Exec 'taskkill /F /T /IM apolo-mcp.exe'
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'taskkill /F /T /IM apolo-mcp.exe'
  Pop $0
!macroend
