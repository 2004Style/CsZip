# Guía de Instalación de CsZip

`cszip` se puede instalar de forma automatizada mediante scripts (recomendado para la mayoría de los usuarios) o compilando manualmente desde el código fuente.

---

## 🚀 Método 1: Instalación Automatizada (Recomendado)

El instalador detectará automáticamente el sistema operativo (Linux, macOS, Windows) y la arquitectura de CPU (AMD64, ARM64) para instalar el binario correcto y configurarlo bajo los directorios de usuario estándar (cumpliendo con la especificación XDG).

### En Linux y macOS (POSIX)

Ejecuta el siguiente comando en tu terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/user/CsZip/main/install.sh | sh
```

- **Ubicación del binario:** `$HOME/.local/bin/cszip`
- **Ubicación de recursos:** `$HOME/.local/share/cszip/`
- **PATH:** Si `$HOME/.local/bin` no está en tu variable de entorno PATH, el instalador te mostrará las instrucciones para agregarlo agregando una línea en tu `~/.bashrc` o `~/.zshrc`.

### En Windows (PowerShell)

Ejecuta la siguiente línea en PowerShell como usuario normal:
```powershell
powershell -ExecutionPolicy Bypass -Command "Invoke-Expression (Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/user/CsZip/main/install.ps1' -UseBasicParsing).Content"
```

- **Ubicación del binario:** `$HOME\.local\bin\cszip.exe`
- **Ubicación de recursos:** `$HOME\.local\share\cszip\`
- **PATH:** El script te dará la instrucción de actualizar tu variable de entorno PATH de usuario si no está configurada.

---

## 🛠️ Método 2: Compilar e Instalar desde Fuente (Desarrolladores)

Si deseas compilar la herramienta tú mismo u optimizarla para tu arquitectura de CPU específica, sigue las siguientes instrucciones.

### 1. Clonar el repositorio
```bash
git clone https://github.com/user/CsZip.git
cd CsZip
```

### 2. Configuración del Entorno de Desarrollo y Dependencias

Ejecuta el script de desarrollo para instalar de forma segura las dependencias necesarias de compilación (como `git`, `curl`, `unrar`, etc.) y configurar los componentes linter (`clippy` y `rustfmt`).

**En Linux (Debian, Ubuntu, Mint, Kali, Fedora, RHEL, Arch Linux) y macOS:**
```bash
chmod +x scripts/*.sh install.sh
./scripts/dev.sh
```
*El script detectará tu distribución de Linux exacta o macOS e invocará a su gestor de paquetes (`apt`, `dnf`, `pacman` o `brew`) solicitando permisos de root sólo si es estrictamente necesario.*

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

### 3. Compilar para Release

**En Linux / macOS:**
```bash
./scripts/build.sh
```
*Esto generará el paquete compilado en `dist/cszip-<os>-<arch>.tar.gz`.*

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1
```
*Esto generará el paquete en `dist/cszip-windows-<arch>.zip`.*

### 4. Ejecutar el Instalador Local

Una vez construido el paquete en el directorio `dist/`, ejecuta el instalador para copiar el binario final local a tu PATH.

**En Linux / macOS:**
```bash
./install.sh
```

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

---

## ❓ Solución de Problemas

### Comando `cszip` no encontrado tras la instalación
- Asegúrate de reiniciar tu terminal o recargar tu perfil de configuración (`source ~/.bashrc` o `source ~/.zshrc` en Linux/macOS).
- En Windows, asegúrate de haber ejecutado la línea que actualiza el PATH de tu entorno de usuario y haber abierto una nueva terminal de PowerShell o CMD.

### Detección de 'unrar' en Windows
- Para extraer archivos `.rar`, `cszip` requiere la utilidad de consola `unrar.exe` en tu PATH. Puedes instalarla fácilmente ejecutando:
  ```powershell
  winget install -e --id RARLab.UnRAR
  ```
