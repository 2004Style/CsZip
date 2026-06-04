#!/usr/bin/env sh
# install.sh - Instalador universal para CsZip (Linux, macOS, Windows Git Bash)
set -eu

APP_NAME="cszip"
REPO="user/CsZip"
BIN_DIR="${HOME}/.local/bin"
DATA_DIR="${HOME}/.local/share/${APP_NAME}"
TMP_DIR=""

# Colores para salida agradable
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;37m'

cleanup() {
  if [ -n "${TMP_DIR:-}" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT

echo "=== Instalador para ${APP_NAME} ==="

# 1. Validar sistema operativo
OS_RAW="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS_RAW" in
  linux) OS="linux"; BIN_NAME="${APP_NAME}" ;;
  darwin) OS="macos"; BIN_NAME="${APP_NAME}" ;;
  msys*|mingw*|cygwin*|windows*) OS="windows"; BIN_NAME="${APP_NAME}.exe" ;;
  *) echo "${RED}Error: Sistema operativo no soportado: ${OS_RAW}${NC}"; exit 1 ;;
esac

# 2. Validar arquitectura
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "${RED}Error: Arquitectura no soportada: ${ARCH}${NC}"; exit 1 ;;
esac

# 3. Validar comandos necesarios
for cmd in mkdir cp chmod; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "${RED}Error: Comando requerido no encontrado: ${cmd}${NC}"
    exit 1
  fi
done

# Crear directorios de destino
mkdir -p "$BIN_DIR"
mkdir -p "$DATA_DIR"

TMP_DIR="$(mktemp -d)"

if [ "$OS" = "windows" ]; then
  ARCHIVE_NAME="${APP_NAME}-${OS}-${ARCH}.zip"
else
  ARCHIVE_NAME="${APP_NAME}-${OS}-${ARCH}.tar.gz"
fi

LOCAL_ARCHIVE="dist/${ARCHIVE_NAME}"

# 4. Obtener el archivo
if [ -f "$LOCAL_ARCHIVE" ]; then
  echo "✓ Detectado archivo de compilación local: ${LOCAL_ARCHIVE}"
  echo "Instalando desde artefacto local..."
  cp "$LOCAL_ARCHIVE" "${TMP_DIR}/${ARCHIVE_NAME}"
else
  if ! command -v curl >/dev/null 2>&1; then
    echo "${RED}Error: Se requiere 'curl' para descargar el paquete de internet.${NC}"
    echo "Alternativamente, compila el proyecto localmente primero ejecutando: ./scripts/build.sh"
    exit 1
  fi

  URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE_NAME}"
  echo "Descargando ${ARCHIVE_NAME} de GitHub..."
  echo "URL: $URL"
  if ! curl -fsSL "$URL" -o "${TMP_DIR}/${ARCHIVE_NAME}"; then
    echo "${RED}Error: No se pudo descargar el paquete desde GitHub.${NC}"
    echo "Puedes instalar el paquete compilándolo localmente:"
    echo "  ./scripts/build.sh"
    echo "  ./install.sh"
    exit 1
  fi
fi

# 5. Descomprimir e instalar
echo "Descomprimiendo artefacto..."
if [ "$OS" = "windows" ] && command -v unzip >/dev/null 2>&1; then
  unzip -q "${TMP_DIR}/${ARCHIVE_NAME}" -d "$TMP_DIR"
else
  tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"
fi

EXTRACTED_DIR="${TMP_DIR}/${APP_NAME}-${OS}-${ARCH}"

if [ ! -d "$EXTRACTED_DIR" ]; then
  echo "${RED}Error: El directorio extraído no coincide con el formato esperado.${NC}"
  exit 1
fi

echo "Copiando binario a ${BIN_DIR}..."
cp "${EXTRACTED_DIR}/bin/${BIN_NAME}" "${BIN_DIR}/${BIN_NAME}"
chmod 755 "${BIN_DIR}/${BIN_NAME}"

echo "Copiando recursos adicionales a ${DATA_DIR}..."
if [ -d "${EXTRACTED_DIR}/share/${APP_NAME}" ]; then
  cp -R "${EXTRACTED_DIR}/share/${APP_NAME}/." "$DATA_DIR/"
fi
if [ -f "${EXTRACTED_DIR}/LICENSE" ]; then
  cp "${EXTRACTED_DIR}/LICENSE" "$DATA_DIR/"
fi
if [ -f "${EXTRACTED_DIR}/README.md" ]; then
  cp "${EXTRACTED_DIR}/README.md" "$DATA_DIR/"
fi

# 6. Verificar PATH
echo ""
echo -e "${GREEN}✓ ¡${APP_NAME} se ha instalado correctamente!${NC}"
echo "Ubicación del binario: ${BIN_DIR}/${BIN_NAME}"
echo "Ubicación de recursos: ${DATA_DIR}"

case ":${PATH}:" in
  *:"${BIN_DIR}":*) ;;
  *)
    echo -e "${YELLOW}Advertencia: ${BIN_DIR} no está en tu variable PATH.${NC}"
    if [ "$OS" = "windows" ]; then
      echo "Para ejecutar '${APP_NAME}' desde cualquier lugar, añade '${BIN_DIR}' a las variables de entorno de tu PATH de Windows."
    else
      echo "Añade la siguiente línea a tu ~/.bashrc o ~/.zshrc:"
      echo -e "  ${BLUE}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
      echo "Luego, recarga: source ~/.bashrc"
    fi
    ;;
esac

echo ""
echo "Ejemplos de uso rápido:"
echo -e "  Comprimir archivo a .cz:  ${BLUE}${APP_NAME} compress archivo.txt${NC}"
echo -e "  Comprimir a .zip:         ${BLUE}${APP_NAME} compress archivo.txt -o archivo.zip${NC}"
echo -e "  Descomprimir:             ${BLUE}${APP_NAME} decompress archivo.cz${NC}"
echo ""
