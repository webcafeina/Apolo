# Plantilla: rellenar.py pone la versión y las sumas de los instaladores de la Release. Lo de antes
# de la primera línea en blanco no llega al paquete.

$ErrorActionPreference = 'Stop'

$version = '{{VERSION}}'
$base = "https://github.com/webcafeina/Apolo/releases/download/v$version"

# Win32_Processor.Architecture 12 es ARM64. Chocolatey no tiene un parámetro propio para ARM, así que
# el instalador de ARM64 va como «64 bits» cuando la máquina lo es.
$arm64 = (Get-CimInstance Win32_Processor | Select-Object -First 1).Architecture -eq 12
if ($arm64) {
  $url = "$base/Apolo_${version}_arm64-setup.exe"
  $checksum = '{{SHA_ARM64}}'
} else {
  $url = "$base/Apolo_${version}_x64-setup.exe"
  $checksum = '{{SHA_X64}}'
}

$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  fileType       = 'exe'
  url64bit       = $url
  checksum64     = $checksum
  checksumType64 = 'sha256'
  silentArgs     = '/S'
  validExitCodes = @(0)
  softwareName   = 'Apolo*'
}

Install-ChocolateyPackage @packageArgs
