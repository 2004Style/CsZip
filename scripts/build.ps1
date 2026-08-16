# scripts/build.ps1 - Compila y empaqueta CsZip en Windows para distribución
# Ejecución: powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1

Write-Host "=== Proceso de Compilación de Release para CsZip (Windows) ===" -ForegroundColor Blue

# 1. Asegurar pruebas limpias
Write-Host "Ejecutando pruebas..." -ForegroundColor Blue
cargo test --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Las pruebas fallaron. Cancelando la compilación de release." -ForegroundColor Red
    Exit 1
}
Write-Host "✓ Pruebas exitosas." -ForegroundColor Green

# 2. Compilar binario de release
Write-Host "Compilando binario de release optimizado..." -ForegroundColor Blue
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Falló la compilación de release." -ForegroundColor Red
    Exit 1
}

# Detectar Arquitectura
$arch = $env:PROCESSOR_ARCHITECTURE
switch -regex ($arch) {
    "AMD64" { $archName = "amd64" }
    "ARM64" { $archName = "arm64" }
    default { $archName = "amd64" } # Default a amd64
}

$version = Select-String -Path .\Cargo.toml -Pattern '^version = ' | Select-Object -First 1 | ForEach-Object { $_.Matches.Value -replace '^version = "', '' -replace '"$', '' }
if (-not $version) { $version = "0.0.1" }

$releaseName = "cszip-windows-$archName"
$archiveName = "$releaseName.zip"
$binaryPath = ".\target\release\cszip.exe"

if (-not (Test-Path $binaryPath)) {
    Write-Host "Error: No se encontró el binario compilado en: $binaryPath" -ForegroundColor Red
    Exit 1
}
Write-Host "✓ Binario compilado para windows/$archName." -ForegroundColor Green

# 3. Preparar directorio de distribución
Write-Host "Preparando archivos para el empaquetado..." -ForegroundColor Blue
$distDir = ".\dist"
$stageDir = Join-Path $distDir $releaseName

if (Test-Path $distDir) { Remove-Item -Recurse -Force $distDir }
$null = New-Item -ItemType Directory -Path (Join-Path $stageDir "bin")
$null = New-Item -ItemType Directory -Path (Join-Path $stageDir "share\cszip")

# Copiar archivos
Copy-Item $binaryPath -Destination (Join-Path $stageDir "bin\")
if (Test-Path ".\README.md") { Copy-Item ".\README.md" -Destination $stageDir }
if (Test-Path ".\LICENSE") { Copy-Item ".\LICENSE" -Destination $stageDir }
if (Test-Path ".\ARCHITECTURE.md") { Copy-Item ".\ARCHITECTURE.md" -Destination (Join-Path $stageDir "share\cszip\") }

# 4. Crear archivo comprimido .zip
Write-Host "Empaquetando en $archiveName..." -ForegroundColor Blue
$archiveOutPath = Join-Path $distDir $archiveName
Compress-Archive -Path $stageDir -DestinationPath $archiveOutPath

# 5. Generar hash de verificación SHA-256
Write-Host "Generando firma de verificación SHA-256..." -ForegroundColor Blue
$hashResult = Get-FileHash -Path $archiveOutPath -Algorithm SHA256
$hashString = $hashResult.Hash.ToLower() + "  " + $archiveName
$hashString | Out-File -FilePath (Join-Path $distDir "$archiveName.sha256") -Encoding ascii

# Limpiar directorio temporal
Remove-Item -Recurse -Force $stageDir

Write-Host "`n=====================================================" -ForegroundColor Green
Write-Host "✓ ¡Compilación y empaquetamiento completados!" -ForegroundColor Green
Write-Host "Artefacto creado: $archiveOutPath" -ForegroundColor Blue
Write-Host "Suma SHA-256:     $($hashResult.Hash.ToLower())" -ForegroundColor Blue
Write-Host "=====================================================" -ForegroundColor Green
