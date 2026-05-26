#!/usr/bin/env bash
# scripts/build.sh - Compila y empaqueta CsZip para distribución (Multi-OS)
set -euo pipefail

# Colores para salida agradable
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;37m'

echo -e "${BLUE}=== Proceso de Compilación de Release para CsZip ===${NC}"

# 1. Asegurar pruebas limpias
echo -e "${BLUE}Ejecutando pruebas para asegurar que el código es correcto...${NC}"
if ! cargo test --quiet; then
    echo -e "${RED}Error: Las pruebas fallaron. Cancela la compilación de release.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Pruebas exitosas.${NC}"

# 2. Compilar binario en modo release
echo -e "${BLUE}Compilando binario de release optimizado...${NC}"
cargo build --release

# Detectar OS y Arquitectura
OS_RAW="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS_RAW" in
    linux) OS="linux" ;;
    darwin) OS="macos" ;;
    msys*|mingw*|cygwin*|windows*) OS="windows" ;;
    *) OS="unknown" ;;
esac

ARCH_RAW="$(uname -m)"
case "$ARCH_RAW" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) ARCH="$ARCH_RAW" ;;
esac

APP_NAME="cszip"
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
RELEASE_NAME="${APP_NAME}-${OS}-${ARCH}"

# Determinar nombre del ejecutable y empaquetado
BINARY_SRC="target/release/${APP_NAME}"
if [ "$OS" = "windows" ]; then
    BINARY_SRC="${BINARY_SRC}.exe"
    ARCHIVE_NAME="${RELEASE_NAME}.zip"
else
    ARCHIVE_NAME="${RELEASE_NAME}.tar.gz"
fi

if [ ! -f "$BINARY_SRC" ]; then
    echo -e "${RED}Error: No se encontró el binario en: ${BINARY_SRC}${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Binario compilado para ${OS}/${ARCH}.${NC}"

# 3. Preparar directorio de distribución
echo -e "${BLUE}Preparando archivos para el empaquetado...${NC}"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${RELEASE_NAME}"

# Limpiar compilaciones anteriores
rm -rf "$DIST_DIR"
mkdir -p "${STAGE_DIR}/bin"
mkdir -p "${STAGE_DIR}/share/${APP_NAME}"

# Copiar binario y recursos principales
cp "$BINARY_SRC" "${STAGE_DIR}/bin/"
[ -f "README.md" ] && cp "README.md" "${STAGE_DIR}/"
[ -f "LICENSE" ] && cp "LICENSE" "${STAGE_DIR}/"
[ -f "ARCHITECTURE.md" ] && cp "ARCHITECTURE.md" "${STAGE_DIR}/share/${APP_NAME}/"

# 4. Crear archivo comprimido
echo -e "${BLUE}Empaquetando en ${ARCHIVE_NAME}...${NC}"
cd "$DIST_DIR"

if [ "$OS" = "windows" ] && command -v zip >/dev/null 2>&1; then
    zip -r "$ARCHIVE_NAME" "$RELEASE_NAME"
else
    # Por defecto usamos tar.gz para Linux/macOS y como fallback en Windows
    tar -czf "$ARCHIVE_NAME" "$RELEASE_NAME"
fi
cd ..

# 5. Generar suma de verificación SHA-256
echo -e "${BLUE}Generando suma de verificación SHA-256...${NC}"
if command -v sha256sum &> /dev/null; then
    cd "$DIST_DIR"
    sha256sum "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
    cd ..
elif command -v shasum &> /dev/null; then
    cd "$DIST_DIR"
    shasum -a 256 "$ARCHIVE_NAME" > "${ARCHIVE_NAME}.sha256"
    cd ..
elif command -v openssl &> /dev/null; then
    cd "$DIST_DIR"
    # Utilizar openssl como alternativa
    openssl dgst -sha256 "$ARCHIVE_NAME" | cut -d ' ' -f 2 > "${ARCHIVE_NAME}.sha256"
    cd ..
else
    echo -e "${YELLOW}Advertencia: No se encontró herramienta para calcular la suma de verificación.${NC}"
fi

# Limpiar directorio temporal de empaquetado
rm -rf "$STAGE_DIR"

echo -e "\n${GREEN}=====================================================${NC}"
echo -e "${GREEN}✓ ¡Proceso de compilación y empaquetamiento completado!${NC}"
echo -e "Artefacto creado: ${BLUE}${DIST_DIR}/${ARCHIVE_NAME}${NC}"
if [ -f "${DIST_DIR}/${ARCHIVE_NAME}.sha256" ]; then
    echo -e "Suma SHA-256:     ${BLUE}$(cat ${DIST_DIR}/${ARCHIVE_NAME}.sha256)${NC}"
fi
echo -e "Listo para ser distribuido o descargado con install.sh o install.ps1."
echo -e "${GREEN}=====================================================${NC}"
