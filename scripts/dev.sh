#!/usr/bin/env bash
# scripts/dev.sh - Configura el entorno de desarrollo para CsZip (Multi-OS & Multi-Distro)
set -euo pipefail

# Colores para salida agradable
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;37m' # No Color

echo -e "${BLUE}=== Configuración del Entorno de Desarrollo para CsZip ===${NC}"

# Variables globales para sistema detectado
OS_TYPE="unknown"
DISTRO_ID="unknown"
DISTRO_LIKE="unknown"

detect_os() {
    OS="$(uname -s)"
    case "$OS" in
        Linux*)
            OS_TYPE="linux"
            if [ -f /etc/os-release ]; then
                # Leer variables de os-release sin exportar todo
                DISTRO_ID="$(grep '^ID=' /etc/os-release | cut -d= -f2 | tr -d '"')"
                DISTRO_LIKE="$(grep '^ID_LIKE=' /etc/os-release | cut -d= -f2 | tr -d '"' || echo "")"
            else
                DISTRO_ID="unknown"
                DISTRO_LIKE="unknown"
            fi
            ;;
        Darwin*)
            OS_TYPE="macos"
            DISTRO_ID="macos"
            DISTRO_LIKE="macos"
            ;;
        MSYS*|MINGW*|CYGWIN*)
            OS_TYPE="windows"
            DISTRO_ID="windows"
            DISTRO_LIKE="windows"
            ;;
        *)
            OS_TYPE="unknown"
            DISTRO_ID="unknown"
            DISTRO_LIKE="unknown"
            ;;
    esac
}

detect_os
echo -e "${GREEN}✓ Sistema operativo detectado: ${OS_TYPE} (${DISTRO_ID})${NC}"

# Ejecutar comandos de paquete con privilegios adecuados
run_pkg_mgr() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        echo -e "${YELLOW}Advertencia: Se requieren privilegios de root pero 'sudo' no está disponible.${NC}"
        echo -e "Ejecuta manualmente como root: $*"
        exit 1
    fi
}

install_deps() {
    case "$OS_TYPE" in
        linux)
            # Detección de gestor de paquetes de Linux
            if [ "$DISTRO_ID" = "ubuntu" ] || [ "$DISTRO_ID" = "debian" ] || [ "$DISTRO_ID" = "linuxmint" ] || [ "$DISTRO_ID" = "kali" ] || [[ "$DISTRO_LIKE" == *"debian"* ]] || [[ "$DISTRO_LIKE" == *"ubuntu"* ]]; then
                echo -e "${BLUE}Detectado sistema basado en Debian/Ubuntu. Instalando dependencias con apt...${NC}"
                run_pkg_mgr apt-get update -y
                run_pkg_mgr apt-get install -y build-essential git curl tar gzip unzip pkg-config || true
                if run_pkg_mgr apt-get install -y unrar; then
                    echo -e "${GREEN}✓ 'unrar' instalado.${NC}"
                elif run_pkg_mgr apt-get install -y unrar-free; then
                    echo -e "${GREEN}✓ 'unrar-free' instalado (compatibilidad básica).${NC}"
                else
                    echo -e "${YELLOW}Advertencia: No se pudo instalar 'unrar'. Soporte RAR limitado.${NC}"
                fi
            elif [ "$DISTRO_ID" = "fedora" ] || [ "$DISTRO_ID" = "rhel" ] || [ "$DISTRO_ID" = "centos" ] || [[ "$DISTRO_LIKE" == *"rhel"* ]] || [[ "$DISTRO_LIKE" == *"fedora"* ]]; then
                echo -e "${BLUE}Detectado sistema basado en Fedora/RHEL. Instalando dependencias con dnf...${NC}"
                run_pkg_mgr dnf groupinstall -y "Development Tools" || true
                run_pkg_mgr dnf install -y git curl tar gzip unzip || true
                if run_pkg_mgr dnf install -y unrar; then
                    echo -e "${GREEN}✓ 'unrar' instalado.${NC}"
                else
                    echo -e "${YELLOW}Advertencia: No se pudo instalar 'unrar'. Soporte RAR limitado.${NC}"
                fi
            elif [ "$DISTRO_ID" = "arch" ] || [ "$DISTRO_ID" = "manjaro" ] || [[ "$DISTRO_LIKE" == *"arch"* ]]; then
                echo -e "${BLUE}Detectado sistema basado en Arch Linux. Instalando dependencias con pacman...${NC}"
                run_pkg_mgr pacman -S --needed --noconfirm base-devel git curl tar gzip unzip unrar || true
            else
                echo -e "${YELLOW}Distribución Linux no reconocida automáticamente. Asegúrate de tener instalados: git, curl, tar, gzip, unrar y herramientas de compilación C.${NC}"
            fi
            ;;
        macos)
            echo -e "${BLUE}Detectado macOS. Verificando Xcode Command Line Tools...${NC}"
            if ! xcode-select -p >/dev/null 2>&1; then
                echo -e "${BLUE}Instalando Xcode Command Line Tools...${NC}"
                xcode-select --install || true
                echo -e "${YELLOW}Por favor, completa la instalación visual de Xcode Tools y vuelve a ejecutar este script.${NC}"
                exit 1
            fi
            if command -v brew >/dev/null 2>&1; then
                echo -e "${BLUE}Instalando dependencias adicionales con Homebrew...${NC}"
                brew install git curl unrar || true
            else
                echo -e "${YELLOW}Homebrew no detectado. Por favor, instala 'git', 'curl' y 'unrar' de forma manual si no están disponibles.${NC}"
            fi
            ;;
        windows)
            echo -e "${BLUE}Detectado entorno Windows (Git Bash/MSYS). Verificando herramientas...${NC}"
            if ! command -v git >/dev/null 2>&1; then
                echo -e "${RED}Error: Git no está en tu PATH de Windows.${NC}"
                exit 1
            fi
            if ! command -v unrar >/dev/null 2>&1; then
                echo -e "${YELLOW}Advertencia: 'unrar' no está en tu PATH. Puedes instalarlo usando 'winget install RARLab.UnRAR' o 'choco install unrar'.${NC}"
            fi
            echo -e "${BLUE}Nota: Para Windows nativo, también puedes ejecutar el script nativo 'scripts/dev.ps1' desde PowerShell.${NC}"
            ;;
        *)
            echo -e "${YELLOW}Sistema operativo no soportado directamente para instalación automática.${NC}"
            ;;
    esac
}

# Ejecutar la lógica de dependencias
install_deps

# Instalar o verificar Rust
if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    echo -e "${BLUE}Instalando Rust a través de rustup.rs...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Cargar variables de entorno en la sesión actual
    if [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi
else
    echo -e "${GREEN}✓ Rust y Cargo detectados.${NC} ($(rustc --version))"
fi

# Instalar componentes recomendados
if command -v rustup >/dev/null 2>&1; then
    echo -e "${BLUE}Verificando componentes clippy y rustfmt...${NC}"
    rustup component add clippy rustfmt || true
fi

# Compilar proyecto en modo debug
echo -e "${BLUE}Compilando el proyecto en modo debug...${NC}"
cargo build

# Formato de código
echo -e "${BLUE}Verificando formato de código...${NC}"
if ! cargo fmt -- --check; then
    echo -e "${YELLOW}El código no cumple con el formato estándar. Reformateando...${NC}"
    cargo fmt
    echo -e "${GREEN}✓ Formato corregido.${NC}"
else
    echo -e "${GREEN}✓ Formato correcto.${NC}"
fi

# Linter Clippy
echo -e "${BLUE}Ejecutando clippy (linter)...${NC}"
cargo clippy --all-targets -- -D warnings || cargo clippy --all-targets

# Ejecutar pruebas
echo -e "${BLUE}Ejecutando pruebas...${NC}"
cargo test

echo -e "\n${GREEN}=====================================================${NC}"
echo -e "${GREEN}✓ ¡Entorno de desarrollo configurado y verificado!${NC}"
echo -e "Puedes compilar el binario para release usando: ${BLUE}./scripts/build.sh${NC}"
echo -e "O en Windows PowerShell usando: ${BLUE}.\scripts\build.ps1${NC}"
echo -e "${GREEN}=====================================================${NC}"
