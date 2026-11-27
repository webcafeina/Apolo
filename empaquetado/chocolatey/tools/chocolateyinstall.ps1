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

# El marcador del gestor: un fichero «gestor» junto a apolo-app.exe que dice «chocolatey». Con él,
# Apolo no se actualiza por su cuenta (lo hace choco upgrade) y su aviso de versión nueva lo dice.
# Si algo falla aquí, Apolo queda instalado igual: solo se avisaría de más.
try {
  $dir = $null
  $clave = @(Get-UninstallRegistryKey -SoftwareName 'Apolo*') | Select-Object -First 1
  if ($clave -and $clave.InstallLocation) { $dir = $clave.InstallLocation -replace '"', '' }
  if (-not $dir -or -not (Test-Path (Join-Path $dir 'apolo-app.exe'))) { $dir = Join-Path $env:LOCALAPPDATA 'Apolo' }
  if (Test-Path (Join-Path $dir 'apolo-app.exe')) {
    Set-Content -Path (Join-Path $dir 'gestor') -Value 'chocolatey' -Encoding ascii -NoNewline
  } else {
    Write-Warning "No se encuentra apolo-app.exe: sin marcador del gestor."
  }
} catch {
  Write-Warning "Sin marcador del gestor: $_"
}
