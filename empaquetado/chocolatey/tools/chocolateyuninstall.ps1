# Desinstala Apolo con el desinstalador de NSIS que deja el propio instalador, en silencio. Lo de
# antes de la primera línea en blanco no llega al paquete.

$ErrorActionPreference = 'Stop'

$claves = @(Get-UninstallRegistryKey -SoftwareName 'Apolo*')

if ($claves.Count -eq 1) {
  $desinstalador = $claves[0].UninstallString -replace '"', ''
  Uninstall-ChocolateyPackage -PackageName $env:ChocolateyPackageName -FileType 'exe' `
    -SilentArgs '/S' -File $desinstalador -ValidExitCodes @(0)
} elseif ($claves.Count -eq 0) {
  Write-Warning "Apolo ya no estaba instalado: no hay nada que desinstalar."
} else {
  Write-Warning "Hay $($claves.Count) instalaciones de Apolo; no se desinstala ninguna para no tocar la que no es:"
  $claves | ForEach-Object { Write-Warning "- $($_.DisplayName)" }
}
