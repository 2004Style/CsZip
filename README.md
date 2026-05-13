# CsZip

Compresor de archivos sin pérdida, rápido y seguro, escrito en Rust.
Formato binario propio `.cz` inspirado en XZ Utils y Zstd.

```
cszip compress datos.bin         # datos.bin → datos.bin.cz
cszip decompress datos.bin.cz    # datos.bin.cz → datos.bin
cszip verify datos.bin.cz        # verificar integridad
cszip info datos.bin.cz          # ver metadata
```

---

## Características

- **Rápido** — compresión ~500 MiB/s, descompresión ~600 MiB/s (STORE)
- **Seguro** — CRC-32, CRC-64, ADLER-32 en cada bloque; protección anti-zip-bomb
- **Streaming** — procesa archivos de cualquier tamaño sin cargar todo en memoria
- **Multiplataforma** — Linux, macOS (Intel + Apple Silicon), Windows

## Instalar

### Binarios pre-compilados

Descarga desde [GitHub Releases](https://github.com/tu-usuario/cszip/releases/latest):

| Sistema | Archivo |
|---------|---------|
| Linux x86_64 | `cszip-linux-x86_64.tar.gz` |
| Linux x86_64 (estático) | `cszip-linux-x86_64-musl.tar.gz` |
| macOS Intel | `cszip-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `cszip-macos-aarch64.tar.gz` |
| Windows x86_64 | `cszip-windows-x86_64.zip` |

```bash
# Linux/macOS
curl -LO https://github.com/tu-usuario/cszip/releases/latest/download/cszip-linux-x86_64.tar.gz
tar -xzf cszip-linux-x86_64.tar.gz
sudo mv cszip /usr/local/bin/
```

### Compilar desde fuente

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # instalar Rust
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
# binario en target/release/cszip
```

Compilación optimizada para tu CPU:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

Guía completa: [docs/INSTALL.md](docs/INSTALL.md)

---

## Uso

### Comandos

| Comando | Alias | Descripción |
|---------|-------|-------------|
| `cszip compress <archivo>` | `cszip c` | Comprimir archivo |
| `cszip decompress <archivo.cz>` | `cszip d` | Descomprimir archivo |
| `cszip verify <archivo.cz>` | `cszip v` | Verificar integridad |
| `cszip info <archivo.cz>` | `cszip i` | Mostrar información |
| `cszip list <archivo.cz>` | `cszip l` | Listar bloques |

### Opciones de compresión

```bash
cszip compress -l 9 archivo.txt          # nivel máximo (0-9)
cszip compress -o backup.cz archivo.txt  # nombre de salida personalizado
cszip compress -f archivo.txt            # sobrescribir si existe
cszip compress --crc64 datos.bin         # usar CRC-64 en vez de CRC-32
```

### Opciones de descompresión

```bash
cszip decompress -o salida.txt archivo.cz  # nombre de salida
cszip decompress -f archivo.cz             # sobrescribir
cszip decompress --no-verify archivo.cz    # saltar verificación (más rápido)
```

### Inspección

```bash
cszip info archivo.cz       # resumen del archivo
cszip info -d archivo.cz    # detalle por bloque
cszip list -v archivo.cz    # tabla de bloques con ratios
```

Manual completo: [docs/USAGE.md](docs/USAGE.md)

---

## Formato .cz

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

| Campo | Valor |
|-------|-------|
| Extensión | `.cz` |
| Endianness | Big-endian |
| Bloque máximo | 64 KB |
| Checksums | CRC-32, CRC-64, ADLER-32 |
| Algoritmos | STORE (0), LZ77+Huffman (1), LZ4 (2), LZMA (3), DEFLATE (4) |

Especificación completa: [FORMAT.md](FORMAT.md)

---

## Arquitectura

```
src/
├── lib.rs          API pública
├── main.rs         CLI (clap)
├── error.rs        Tipos de error con códigos numéricos
├── utils.rs        Helpers (format_size, throughput, etc.)
├── cli.rs          Módulo CLI (args, commands, progress)
├── cli/
│   ├── args.rs     Argumentos y subcomandos
│   ├── commands.rs Lógica de cada comando
│   └── progress.rs Barra de progreso
├── codec.rs        Módulo codecs (Algorithm, CompressionLevel)
├── codec/
│   ├── compressor.rs   Motor de compresión
│   ├── decompressor.rs Motor de descompresión
│   ├── lz77.rs         LZ77 con ventana deslizante
│   ├── huffman.rs      Codificación Huffman
│   └── filters.rs      Filtros (delta, RLE, MTF, BWT)
├── format.rs       Módulo formato (re-exports)
├── format/
│   ├── header.rs    File Header (16 bytes)
│   ├── block.rs     Block Header (12 bytes) + Footer (12 bytes)
│   ├── checksum.rs  CRC-32, CRC-64, ADLER-32
│   └── constants.rs Valores fijos del formato
├── io.rs           Módulo I/O (re-exports)
└── io/
    ├── reader.rs    CzReader — leer archivos .cz
    ├── writer.rs    CzWriter — crear archivos .cz
    └── streaming.rs API de streaming con progreso
```

---

## Desarrollo

```bash
cargo test --all           # 334 tests
cargo bench                # benchmarks
cargo clippy --all-targets # linter
cargo fmt --check          # formato
cargo doc --no-deps --open # documentación API
```

---

## Roadmap

- [x] Formato binario con verificación de integridad
- [x] Compresión/descompresión STORE
- [x] CLI completa (compress, decompress, verify, info, list)
- [x] Streaming para archivos grandes
- [x] Suite de 334 tests
- [ ] LZ77 + Huffman (compresión real)
- [ ] LZ4-style (ultra-rápido)
- [ ] SIMD / multi-thread
- [ ] LZMA, DEFLATE
- [ ] Publicación en crates.io

---

## CI/CD (GitHub Actions)

Al pushear un tag `v*` (ej: `git tag v0.1.1 && git push origin v0.1.1`), GitHub Actions automáticamente:

1. Compila binarios para Linux, macOS y Windows
2. Ejecuta tests en cada plataforma
3. Crea un Release en GitHub con todos los binarios
4. Genera checksums SHA-256
5. Publica en crates.io (solo releases estables)

Ver [docs/RELEASING.md](docs/RELEASING.md) para detalles.

---

## Documentación

| Documento | Descripción |
|-----------|-------------|
| [docs/INSTALL.md](docs/INSTALL.md) | Instalación (Linux/macOS/Windows) |
| [docs/USAGE.md](docs/USAGE.md) | Manual de uso completo |
| [docs/RELEASING.md](docs/RELEASING.md) | Crear releases con binarios |
| [docs/TESTING.md](docs/TESTING.md) | Testing y desarrollo local |
| [FORMAT.md](FORMAT.md) | Especificación del formato binario |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Arquitectura del código |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Guía para desarrolladores |
| [docs/API.md](docs/API.md) | Documentación de la API |
| [docs/BENCHMARKS.md](docs/BENCHMARKS.md) | Benchmarks |

---

## Licencia

MIT — ver [LICENSE](LICENSE).
