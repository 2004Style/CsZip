# scripts/dev.ps1 - Prepara el entorno de desarrollo para CsZip en Windows
# Ejecución: powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1

Write-Host "=== Configuración del Entorno de Desarrollo para CsZip (Windows) ===" -ForegroundColor Blue

# 1. Verificar si Rust y Cargo están instalados
$cargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
$rustcCheck = Get-Command rustc -ErrorAction SilentlyContinue

if (-not $cargoCheck -or -not $rustcCheck) {
    Write-Host "Rust o Cargo no están instalados o no se encuentran en el PATH." -ForegroundColor Yellow
    Write-Host "Por favor descarga e instala Rust desde: https://rustup.rs/"
    Exit 1
}

$rustVersion = rustc --version
Write-Host "✓ Rust y Cargo detectados: $rustVersion" -ForegroundColor Green

# 2. Verificar Git
$gitCheck = Get-Command git -ErrorAction SilentlyContinue
if (-not $gitCheck) {
    Write-Host "Git no detectado. Se requiere Git para el desarrollo." -ForegroundColor Yellow
    Write-Host "Puedes instalarlo ejecutando en cmd/PowerShell:"
    Write-Host "  winget install -e --id Git.Git"
    Exit 1
}
Write-Host "✓ Git detectado." -ForegroundColor Green

# 3. Verificar unrar (para soporte de extracción RAR)
$unrarCheck = Get-Command unrar -ErrorAction SilentlyContinue
if (-not $unrarCheck) {
    Write-Host "Advertencia: 'unrar' no está en tu variable PATH de Windows." -ForegroundColor Yellow
    Write-Host "Para habilitar el soporte de extracción de archivos .rar, instálalo usando:"
    Write-Host "  winget install -e --id RARLab.UnRAR"
    Write-Host "O vía Chocolatey:"
    Write-Host "  choco install unrar"
} else {
    Write-Host "✓ unrar detectado." -ForegroundColor Green
}

# 4. Instalar componentes recomendados
Write-Host "Instalando componentes recomendados (clippy y rustfmt)..." -ForegroundColor Blue
rustup component add clippy rustfmt

# 5. Compilar dependencias
Write-Host "Compilando el proyecto en modo debug..." -ForegroundColor Blue
cargo build

# 6. Formatear y verificar lints de código
Write-Host "Verificando formato de código..." -ForegroundColor Blue
$fmtResult = cargo fmt -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "El formato del código no cumple las directrices. Ejecutando cargo fmt automático..." -ForegroundColor Yellow
    cargo fmt
    Write-Host "✓ Código formateado automáticamente." -ForegroundColor Green
} else {
    Write-Host "✓ Formato de código correcto." -ForegroundColor Green
}

Write-Host "Ejecutando clippy (linter)..." -ForegroundColor Blue
cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "Clippy falló con advertencias críticas. Ejecutando sin -D warnings..." -ForegroundColor Yellow
    cargo clippy --all-targets
} else {
    Write-Host "✓ Clippy ejecutado exitosamente." -ForegroundColor Green
}

# 7. Ejecutar suite de pruebas
Write-Host "Ejecutando pruebas unitarias y de integración..." -ForegroundColor Blue
cargo test

Write-Host "`n=====================================================" -ForegroundColor Green
Write-Host "✓ ¡Entorno de desarrollo configurado y verificado!" -ForegroundColor Green
Write-Host "Puedes compilar el binario para release en Windows ejecutando:"
Write-Host "  powershell .\scripts\build.ps1"
Write-Host "=====================================================" -ForegroundColor Green
