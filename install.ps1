# install.ps1 - Instalador universal para CsZip en Windows (PowerShell)
# Ejecución: powershell -ExecutionPolicy Bypass -File .\install.ps1

Write-Host "=== Instalador para cszip en Windows ===" -ForegroundColor Blue

$appName = "cszip"
$repo = "user/CsZip"
$binDir = Join-Path $env:USERPROFILE ".local\bin"
$dataDir = Join-Path $env:USERPROFILE ".local\share\cszip"
$tmpDir = Join-Path $env:TEMP ([Guid]::NewGuid().ToString())

# Crear directorios
if (-not (Test-Path $binDir)) { $null = New-Item -ItemType Directory -Path $binDir }
if (-not (Test-Path $dataDir)) { $null = New-Item -ItemType Directory -Path $dataDir }

# Detectar Arquitectura
$arch = $env:PROCESSOR_ARCHITECTURE
switch -regex ($arch) {
    "AMD64" { $archName = "amd64" }
    "ARM64" { $archName = "arm64" }
    default { $archName = "amd64" }
}

$archiveName = "cszip-windows-$archName.zip"
$localArchive = ".\dist\$archiveName"

# Resolver y obtener artefacto
$null = New-Item -ItemType Directory -Path $tmpDir
$tmpZipPath = Join-Path $tmpDir $archiveName

if (Test-Path $localArchive) {
    Write-Host "✓ Detectado archivo de compilación local: $localArchive" -ForegroundColor Green
    Write-Host "Instalando desde artefacto local..."
    Copy-Item $localArchive -Destination $tmpZipPath
} else {
    Write-Host "Descargando $archiveName de GitHub..." -ForegroundColor Blue
    $url = "https://github.com/$repo/releases/latest/download/$archiveName"
    Write-Host "URL: $url"
    
    try {
        Invoke-WebRequest -Uri $url -OutFile $tmpZipPath -UseBasicParsing
    } catch {
        Write-Host "Error: No se pudo descargar el paquete desde GitHub." -ForegroundColor Red
        Write-Host "Alternativamente, puedes compilar el proyecto localmente primero ejecutando:"
        Write-Host "  powershell .\scripts\build.ps1"
        Write-Host "  powershell .\install.ps1"
        Remove-Item -Recurse -Force $tmpDir
        Exit 1
    }
}

# Descomprimir e instalar
Write-Host "Descomprimiendo artefacto..." -ForegroundColor Blue
Expand-Archive -Path $tmpZipPath -DestinationPath $tmpDir

$extractedDir = Join-Path $tmpDir "cszip-windows-$archName"
if (-not (Test-Path $extractedDir)) {
    Write-Host "Error: El directorio extraído no coincide con el formato esperado." -ForegroundColor Red
    Remove-Item -Recurse -Force $tmpDir
    Exit 1
}

# Copiar ejecutables y recursos
Write-Host "Copiando binario a $binDir..." -ForegroundColor Blue
Copy-Item (Join-Path $extractedDir "bin\cszip.exe") -Destination (Join-Path $binDir "cszip.exe") -Force

Write-Host "Copiando recursos adicionales a $dataDir..." -ForegroundColor Blue
if (Test-Path (Join-Path $extractedDir "share\cszip")) {
    Copy-Item (Join-Path $extractedDir "share\cszip\*") -Destination $dataDir -Recurse -Force
}
if (Test-Path (Join-Path $extractedDir "LICENSE")) {
    Copy-Item (Join-Path $extractedDir "LICENSE") -Destination $dataDir -Force
}
if (Test-Path (Join-Path $extractedDir "README.md")) {
    Copy-Item (Join-Path $extractedDir "README.md") -Destination $dataDir -Force
}

# Limpiar directorio temporal
Remove-Item -Recurse -Force $tmpDir

Write-Host "`n✓ ¡cszip se ha instalado correctamente!" -ForegroundColor Green
Write-Host "Ubicación del binario: $(Join-Path $binDir 'cszip.exe')" -ForegroundColor Blue
Write-Host "Ubicación de recursos: $dataDir" -ForegroundColor Blue

# Verificar PATH
$pathEnv = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($pathEnv -split ";" -notcontains $binDir) {
    Write-Host "`nAdvertencia: $binDir no está en tu variable PATH de usuario." -ForegroundColor Yellow
    Write-Host "Para poder ejecutar 'cszip' desde cualquier terminal, ejecuta la siguiente instrucción:" -ForegroundColor Blue
    Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `"`$pathEnv;$binDir`", 'User')" -ForegroundColor Green
    Write-Host "Luego, reinicia la terminal."
}

Write-Host "`nEjemplos de uso rápido:"
Write-Host "  Comprimir archivo a .cz:  cszip compress archivo.txt" -ForegroundColor Blue
Write-Host "  Comprimir a .zip:         cszip compress archivo.txt -o archivo.zip" -ForegroundColor Blue
Write-Host "  Descomprimir:             cszip decompress archivo.cz" -ForegroundColor Blue
