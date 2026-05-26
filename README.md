# CsZip

Compresor de archivos sin pérdida, rápido, seguro y altamente portable, escrito en Rust.
Posee un formato binario propio `.cz` (basado en bloques con compresión nativa `LZ77+Huffman`) e integra soporte transparente de compresión/descompresión para archivos `.zip` y descompresión para archivos `.rar`.

```bash
cszip compress datos.bin         # datos.bin → datos.bin.cz (con LZ77+Huffman)
cszip compress datos.bin -o d.zip # datos.bin → d.zip (formato ZIP estándar)
cszip decompress datos.bin.cz    # descomprimir formato .cz
cszip decompress backup.zip      # descomprimir formato .zip nativamente
cszip decompress backup.rar      # extraer formato .rar (delegando en unrar de sistema)
cszip verify datos.bin.cz        # verificar integridad (checksums del formato .cz)
cszip info datos.bin.cz          # ver metadata detallada
```

---

## Características

- **Compresión Nativa** — Algoritmo `LZ77+Huffman` de alto rendimiento incorporado directamente en el núcleo de la herramienta.
- **Formatos Externos** — Integración nativa de lectura/escritura de `.zip` y extracción de `.rar` (delegación segura en `unrar` de sistema).
- **Seguro** — Verificación redundante con CRC-32, CRC-64 y ADLER-32 a nivel de bloque e integridad global; protección activa contra ataques de denegación de servicio (*zip bombs*).
- **Streaming** — E/S bufferizada y procesamiento por bloques que mantiene el uso de memoria constante, permitiendo procesar archivos de gigabytes en hardware limitado.
- **Portabilidad Universal** — Totalmente automatizado para Linux (soporte multiplataforma e instaladores para Ubuntu, Debian, Mint, Kali, Fedora, RHEL y Arch Linux), macOS y Windows.

---

## Instalar

### Método Automatizado por Script (Recomendado)

**En Linux / macOS (POSIX):**
```bash
curl -fsSL https://raw.githubusercontent.com/user/CsZip/main/install.sh | sh
```

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -Command "Invoke-Expression (Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/user/CsZip/main/install.ps1' -UseBasicParsing).Content"
```

### Compilar desde fuente (Desarrolladores)

**En Linux / macOS (POSIX):**
```bash
# Inicializa el entorno e instala dependencias del sistema y de Rust
./scripts/dev.sh

# Compila y empaqueta en dist/
./scripts/build.sh
```

**En Windows (PowerShell):**
```powershell
# Inicializa el entorno de Windows
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1

# Compila y empaqueta para Windows
powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

---

## Uso

### Comandos

| Comando | Alias | Descripción |
|---------|-------|-------------|
| `cszip compress <archivo>` | `cszip c` | Comprimir archivo (autodetecta extensión `.zip` si se especifica `-o`) |
| `cszip decompress <archivo.cz/.zip/.rar>` | `cszip d` | Descomprimir archivo (CZ nativo, ZIP nativo o RAR mediante unrar) |
| `cszip verify <archivo.cz>` | `cszip v` | Verificar integridad del archivo `.cz` |
| `cszip info <archivo.cz>` | `cszip i` | Mostrar información detallada de bloques de archivo `.cz` |
| `cszip list <archivo.cz>` | `cszip l` | Listar tabla de bloques y ratios de compresión |

### Opciones de compresión

```bash
cszip compress -l 9 archivo.txt          # Nivel máximo (0-9)
cszip compress -a lz77-huffman datos.bin # Usar LZ77+Huffman explícitamente (por defecto)
cszip compress -o backup.zip datos.bin   # Comprimir a archivo zip estándar
cszip compress -f archivo.txt            # Sobrescribir si el archivo destino existe
```

### Opciones de descompresión

```bash
cszip decompress archivo.cz -o salida.txt # Nombre de salida personalizado
cszip decompress -f backup.zip            # Sobrescribir archivos al extraer ZIP
cszip decompress --no-verify archivo.cz    # Saltar verificación de checksums (más rápido)
```

Manual completo: [docs/USAGE.md](docs/USAGE.md)

---

## Estructura del Formato .cz

```
┌─────────────────────────────────┐
│ File Header    (16 bytes)       │  Magic 0x435A, versión, algoritmo, flags
├─────────────────────────────────┤
│ Block 0                         │  Block Header (12 bytes) + datos + CRC
│ Block 1                         │
│ ...                             │
├─────────────────────────────────┤
│ File Footer    (12 bytes)       │  Marker 0xFE, nº bloques, tamaño, CRC global
└─────────────────────────────────┘
```

Especificación completa: [FORMAT.md](FORMAT.md)

---

## Arquitectura del Proyecto

```
src/
├── lib.rs          API pública de la librería
├── main.rs         Punto de entrada CLI (clap)
├── error.rs        Definición y códigos de error robustos
├── utils.rs        Funciones de formato, tamaño y utilidades
├── cli/            Módulos para interfaz de línea de comandos
│   └── commands.rs Intercepción y delegación (CZ, ZIP y RAR)
├── codec/          Motores de compresión
│   ├── lz77.rs     Algoritmo LZ77 nativo
│   ├── huffman.rs  Codificador de Huffman nativo
│   ├── compressor.rs   Flujo de compresión de bloques
│   └── decompressor.rs Flujo de descompresión de bloques
└── utils/
    └── archive.rs  Operaciones nativas ZIP y wrapper CLI para unrar (RAR)
```

---

## Roadmap

- [x] Formato binario con verificación de integridad
- [x] Compresión/descompresión STORE
- [x] CLI completa (compress, decompress, verify, info, list)
- [x] Streaming para archivos grandes
- [x] Suite de tests completa (>330 tests)
- [x] Integración de compresión real `LZ77 + Huffman`
- [x] Soporte para formatos externos `.zip` (nativo) y `.rar` (extracción vía unrar)
- [x] Scripts automatizados universales para desarrolladores y usuarios finales (POSIX + PowerShell)
- [ ] Paralelización multi-hilo
- [ ] Compresión estilo LZ4 y LZMA

---

## Documentación

| Documento | Descripción |
|-----------|-------------|
| [docs/INSTALL.md](docs/INSTALL.md) | Guía de instalación en múltiples distribuciones y Windows |
| [docs/USAGE.md](docs/USAGE.md) | Manual de uso completo y comandos para ZIP/RAR |
| [docs/TESTING.md](docs/TESTING.md) | Suite de pruebas y desarrollo local |
| [FORMAT.md](FORMAT.md) | Especificación física del formato binario `.cz` |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Arquitectura modular del código de Rust |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Guía de fases de desarrollo e hitos |

---

## Licencia

MIT — ver [LICENSE](LICENSE).
