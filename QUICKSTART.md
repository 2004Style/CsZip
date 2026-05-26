# ⚡ Guía de Inicio Rápido (Quick Start) - CsZip

Esta guía te permitirá configurar, compilar y comenzar a usar `cszip` en menos de 5 minutos en cualquier sistema operativo.

---

## 🚀 1. Configurar Entorno de Desarrollo (1 minuto)

Clona el proyecto y ejecuta el script de configuración correspondiente a tu sistema para verificar que tienes todas las dependencias instaladas.

**En Linux / macOS (POSIX):**
```bash
# Dar permisos de ejecución
chmod +x scripts/*.sh install.sh

# Ejecutar script de entorno
./scripts/dev.sh
```

**En Windows (PowerShell):**
```powershell
# Ejecutar script de entorno
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

Este paso validará tu compilador Rust/Cargo, formateará el código, correrá los lints con Clippy e iniciará la suite de pruebas automáticamente.

---

## 🛠️ 2. Compilar Release y Empaquetar (1 minuto)

Una vez verificado el entorno, genera el ejecutable optimizado y el empaquetado de distribución.

**En Linux / macOS (POSIX):**
```bash
./scripts/build.sh
```
*Esto generará el paquete `dist/cszip-<os>-amd64.tar.gz` junto con su firma SHA-256.*

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1
```
*Esto generará el paquete `dist/cszip-windows-amd64.zip` junto con su firma SHA-256.*

---

## 💾 3. Instalación Local (30 segundos)

Instala el binario resultante de forma limpia bajo los directorios de usuario estándar (PATH).

**En Linux / macOS (POSIX):**
```bash
./install.sh
```

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

---

## 📋 4. Comandos Básicos en 1 Minuto

Una vez instalado, abre una nueva terminal y prueba los siguientes flujos de trabajo principales:

### Compresión y Descompresión en Formato Propio `.cz` (Compresión Real LZ77+Huffman)
```bash
# 1. Crear un archivo de prueba
echo "Lorem ipsum dolor sit amet, consectetur adipiscing elit..." > prueba.txt

# 2. Comprimir usando el algoritmo nativo
cszip compress prueba.txt -o comprimido.cz

# 3. Mostrar la información física del archivo .cz
cszip info comprimido.cz

# 4. Descomprimir el archivo restaurando el original
cszip decompress comprimido.cz -o restaurado.txt
```

### Trabajar con Formatos Comunes `.zip` y `.rar` (Soporte Transparente)
```bash
# Comprimir un archivo directamente a formato ZIP estándar
cszip compress prueba.txt -o backup.zip

# Descomprimir un archivo ZIP nativamente
cszip decompress backup.zip -o extraido_zip.txt

# Extraer un archivo RAR (delegan en 'unrar' de tu sistema de forma automática)
cszip decompress archivo_rar.rar -o extraido_rar.txt
```

### Verificación de Integridad
```bash
# Comprobar la firma y checksums de bloques en archivos .cz
cszip verify comprimido.cz
```
