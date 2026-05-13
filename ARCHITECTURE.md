# 🏗️ Arquitectura de CsZip - Guía de Implementación

**Versión:** 1.0  
**Última revisión:** 7 Febrero 2026

---

## 📋 Tabla de Contenidos

1. [Estructura de Carpetas](#estructura-de-carpetas)
2. [Módulos Principales](#módulos-principales)
3. [Interfaces Públicas](#interfaces-públicas)
4. [Patrones de Codificación](#patrones-de-codificación)
5. [Gestión de Errores](#gestión-de-errores)
6. [Testing y Validación](#testing-y-validación)

---

## 📁 Estructura de Carpetas

```
CsZip/
├── Cargo.toml              # Configuración del proyecto
├── Cargo.lock              # Lock de dependencias
│
├── src/
│   ├── lib.rs              # Punto de entrada de librería
│   ├── main.rs             # CLI principal
│   │
│   ├── error.rs            # Tipos de error y manejo
│   ├── utils.rs            # Funciones de utilidad
│   │
│   ├── format/             # Defnición del formato
│   │   ├── mod.rs
│   │   ├── header.rs       # Parseo de global header
│   │   ├── block.rs        # Parseo de block header/footer
│   │   ├── checksum.rs     # Cálculo CRC-32/64
│   │   └── constants.rs    # Constantes del formato
│   │
│   ├── codec/              # Compresión/descompresión
│   │   ├── mod.rs
│   │   ├── compressor.rs   # Lógica de compresión
│   │   ├── decompressor.rs # Lógica de descompresión
│   │   ├── algorithm.rs    # Algoritmo base (LZ77, etc.)
│   │   ├── lz77.rs         # Implementación LZ77
│   │   ├── huffman.rs      # Codificación Huffman
│   │   └── filters.rs      # Filtros preprocesamiento
│   │
│   ├── io/                 # Lectura/escritura eficiente
│   │   ├── mod.rs
│   │   ├── reader.rs       # BufReader mejorado
│   │   ├── writer.rs       # BufWriter mejorado
│   │   └── streaming.rs    # Streaming sin buffering completo
│   │
│   └── cli/                # Interfaz de línea de comandos
│       ├── mod.rs
│       ├── commands.rs     # Definición de comandos
│       ├── options.rs      # Parseo de opciones (clap)
│       └── progress.rs     # Barra de progreso
│
├── tests/                  # Tests de integración
│   ├── basic.rs
│   ├── large_files.rs
│   ├── corruption.rs
│   └── performance.rs
│
├── benches/                # Benchmarks
│   ├── compress.rs
│   ├── decompress.rs
│   └── throughput.rs
│
├── fuzz/                   # Fuzzing targets
│   ├── fuzz_targets/
│   │   ├── fuzz_decompressor.rs
│   │   ├── fuzz_compressor.rs
│   │   └── fuzz_header_parser.rs
│   └── artifacts/
│
├── examples/               # Ejemplos de uso
│   ├── basic_compress.rs
│   ├── basic_decompress.rs
│   ├── streaming.rs
│   └── library_api.rs
│
├── docs/                   # Documentación adicional
│   ├── ARCHITECTURE.md     # Este archivo
│   ├── ALGORITHM.md        # Detalles del algoritmo
│   ├── PERFORMANCE.md      # Benchmarks y optimización
│   └── SECURITY.md         # Consideraciones de seguridad
│
├── README.md               # Documentación principal
├── FORMAT.md               # Especificación del formato
├── LICENSE                 # Licencia MIT
└── .gitignore             # Control de versiones
```

---

## 🧩 Módulos Principales

### 1. `lib.rs` - Punto de Entrada de Librería

**Responsabilidad:** Exportar la API pública

````rust
// src/lib.rs

pub mod error;
pub mod format;
pub mod codec;
pub mod io;

// Re-exportar interfaces públicas
pub use codec::{Compressor, Decompressor};
pub use error::{Error, Result};
pub use format::Header;
pub use io::{Reader, Writer};

// Versión de la librería
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Comprimir un buffer en memoria
///
/// # Ejemplos
/// ```
/// let data = b"Hello, CsZip!";
/// let compressed = CsZip::compress(data, 6)?;
/// # Ok(())
/// ```
pub fn compress(data: &[u8], level: u8) -> Result<Vec<u8>> {
    // Implementación de alto nivel
}

/// Descomprimir un buffer en memoria
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    // Implementación de alto nivel
}
````

### 2. `error.rs` - Manejo de Errores

**Responsabilidad:** Definir todos los tipos de error posibles

```rust
// src/error.rs

use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorKind {
    InvalidMagicNumber,
    UnsupportedVersion,
    UnsupportedAlgorithm,
    InvalidBlockSize,
    InvalidExpansionLimit,
    InvalidBlockType,
    BlockCrcMismatch,
    CompressionBombSuspected,
    IncompleteBlockFound,
    CorruptedBlockHeader,
    CorruptedFileFooter,
    InvalidAdler32Checksum,
    MemoryLimitExceeded,
    UnexpectedEof,
    InvalidCompressionLevel,
    ReservedForFutureUse,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    context: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn code(&self) -> u8 {
        match self.kind {
            ErrorKind::InvalidMagicNumber => 0x01,
            ErrorKind::UnsupportedVersion => 0x02,
            // ... resto de códigos
            ErrorKind::ReservedForFutureUse => 0x10,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ERROR {}] {}", self.code(), self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
```

### 3. `format/mod.rs` - Definición del Formato

**Responsabilidad:** Estructuras y constantes del formato .cz

```rust
// src/format/mod.rs

pub mod header;
pub mod block;
pub mod checksum;
pub mod constants;

pub use header::Header;
pub use block::{BlockHeader, BlockFooter};
pub use checksum::{Crc32, Crc64};
pub use constants::*;

#[derive(Debug, Clone)]
pub struct FileFormat {
    pub header: Header,
    pub blocks: Vec<BlockHeader>,
}

impl FileFormat {
    pub fn validate(&self) -> crate::error::Result<()> {
        self.header.validate()?;
        for block in &self.blocks {
            block.validate(&self.header)?;
        }
        Ok(())
    }
}
```

#### 3.1 `format/header.rs`

```rust
// src/format/header.rs

use crate::error::{Error, ErrorKind, Result};
use std::io::{Read, Write};

const MAGIC_NUMBER_PRIMARY: u16 = 0x435A;     // "CZ"
const MAGIC_NUMBER_ALTERNATE: u16 = 0x5A43;  // "ZC"

#[derive(Debug, Clone)]
pub struct Header {
    pub magic: u16,
    pub version_major: u8,
    pub version_minor: u8,
    pub flags: u8,
    pub compression_algo: u8,
    pub block_size_log2: u16,
    pub max_expansion: u16,
    pub reserved: u16,
    pub checksum: u32,
}

impl Header {
    pub const SIZE: usize = 16;

    pub fn new(
        compression_algo: u8,
        block_size_log2: u16,
        max_expansion: u16,
        flags: u8,
    ) -> Result<Self> {
        if block_size_log2 < 9 || block_size_log2 > 16 {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!("Block size log2 {} fuera de rango [9, 16]", block_size_log2),
            ));
        }

        if max_expansion < 100 || max_expansion > 5000 {
            return Err(Error::new(
                ErrorKind::InvalidExpansionLimit,
                format!("Max expansion {} fuera de rango [100, 5000]", max_expansion),
            ));
        }

        Ok(Self {
            magic: MAGIC_NUMBER_PRIMARY,
            version_major: 1,
            version_minor: 0,
            flags,
            compression_algo,
            block_size_log2,
            max_expansion,
            reserved: 0,
            checksum: 0, // Calcular después
        })
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::SIZE];
        reader.read_exact(&mut buf)
            .map_err(|e| Error::new(
                ErrorKind::UnexpectedEof,
                format!("No se pudo leer header: {}", e),
            ))?;

        Self::from_bytes(&buf)
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);

        // Validación de magic number
        if magic != MAGIC_NUMBER_PRIMARY && magic != MAGIC_NUMBER_ALTERNATE {
            return Err(Error::new(
                ErrorKind::InvalidMagicNumber,
                format!("Magic number 0x{:04X} inválido", magic),
            ));
        }

        let version_major = bytes[2];
        let version_minor = bytes[3];

        // Validación de versión
        if version_major > 1 {
            return Err(Error::new(
                ErrorKind::UnsupportedVersion,
                format!("Versión {}.{} no soportada", version_major, version_minor),
            ));
        }

        let flags = bytes[4];
        let compression_algo = bytes[5];
        let block_size_log2 = u16::from_be_bytes([bytes[6], bytes[7]]);
        let max_expansion = u16::from_be_bytes([bytes[8], bytes[9]]);
        let reserved = u16::from_be_bytes([bytes[10], bytes[11]]);
        let checksum = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        // Validación de algoritmo
        if compression_algo == 15 {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                "Algoritmo experimental (15) no soportado",
            ));
        }

        // Validación de block size
        if block_size_log2 < 9 || block_size_log2 > 16 {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!("Block size log2 {} fuera de rango", block_size_log2),
            ));
        }

        // Validación de max expansion
        if max_expansion < 100 || max_expansion > 5000 {
            return Err(Error::new(
                ErrorKind::InvalidExpansionLimit,
                format!("Max expansion {} fuera de rango", max_expansion),
            ));
        }

        // Validación de flags (bits 4-7 deben ser 0)
        if (flags & 0xF0) != 0 {
            return Err(Error::new(
                ErrorKind::ReservedForFutureUse,
                "Flags contienen bits reservados",
            ));
        }

        Ok(Self {
            magic,
            version_major,
            version_minor,
            flags,
            compression_algo,
            block_size_log2,
            max_expansion,
            reserved,
            checksum,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..2].copy_from_slice(&self.magic.to_be_bytes());
        bytes[2] = self.version_major;
        bytes[3] = self.version_minor;
        bytes[4] = self.flags;
        bytes[5] = self.compression_algo;
        bytes[6..8].copy_from_slice(&self.block_size_log2.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.max_expansion.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.reserved.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.checksum.to_be_bytes());
        bytes
    }

    pub fn validate(&self) -> Result<()> {
        // Ya validado en from_bytes, pero existe para ser explícito
        Ok(())
    }

    pub fn block_size(&self) -> usize {
        1 << self.block_size_log2
    }

    pub fn has_crc64(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn has_extra_metadata(&self) -> bool {
        (self.flags & 0x01) != 0
    }
}
```

#### 3.2 `format/block.rs`

```rust
// src/format/block.rs

use crate::error::{Error, ErrorKind, Result};
use crate::format::Header;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub block_type: u8,
    pub compression_level: u8,
    pub original_size: u16,
    pub compressed_size: u32,
    pub adler32: u32,
}

impl BlockHeader {
    pub const SIZE: usize = 12;

    pub fn new(
        original_size: u16,
        compressed_size: u32,
        adler32: u32,
        compression_level: u8,
    ) -> Result<Self> {
        if original_size == 0 {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                "Original size no puede ser 0",
            ));
        }

        Ok(Self {
            block_type: 0,      // DATA
            compression_level,
            original_size,
            compressed_size,
            adler32,
        })
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::SIZE];
        reader.read_exact(&mut buf)
            .map_err(|e| Error::new(
                ErrorKind::UnexpectedEof,
                format!("No se pudo leer block header: {}", e),
            ))?;

        Self::from_bytes(&buf)
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let block_type = bytes[0];

        if block_type > 3 {
            return Err(Error::new(
                ErrorKind::InvalidBlockType,
                format!("Block type {} inválido", block_type),
            ));
        }

        if block_type == 2 {
            return Err(Error::new(
                ErrorKind::IncompleteBlockFound,
                "Bloque incompleto detectado",
            ));
        }

        let compression_level = bytes[1];
        let original_size = u16::from_be_bytes([bytes[2], bytes[3]]);
        let compressed_size = u32::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let adler32 = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Self {
            block_type,
            compression_level,
            original_size,
            compressed_size,
            adler32,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.block_type;
        bytes[1] = self.compression_level;
        bytes[2..4].copy_from_slice(&self.original_size.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.compressed_size.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.adler32.to_be_bytes());
        bytes
    }

    pub fn validate(&self, file_header: &Header) -> Result<()> {
        let max_block_size = file_header.block_size();

        if self.original_size as usize > max_block_size {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!(
                    "Tamaño original {} excede tamaño de bloque {}",
                    self.original_size, max_block_size
                ),
            ));
        }

        let max_compressed = (self.original_size as u32)
            .saturating_mul(file_header.max_expansion as u32)
            .saturating_div(100);

        if self.compressed_size > max_compressed {
            return Err(Error::new(
                ErrorKind::CompressionBombSuspected,
                format!(
                    "Tamaño comprimido {} excede límite {}",
                    self.compressed_size, max_compressed
                ),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileFooter {
    pub marker: u8,
    pub num_blocks: u32,
    pub total_raw_size: u32,
    pub footer_checksum: u32,
}

impl FileFooter {
    pub const SIZE: usize = 12;
    pub const MARKER: u8 = 0xFE;

    pub fn new(num_blocks: u32, total_raw_size: u32) -> Self {
        Self {
            marker: Self::MARKER,
            num_blocks,
            total_raw_size,
            footer_checksum: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.marker;
        bytes[1..4].copy_from_slice(&(self.num_blocks as u32).to_be_bytes()[1..]);
        bytes[4..8].copy_from_slice(&self.total_raw_size.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.footer_checksum.to_be_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        if bytes[0] != Self::MARKER {
            return Err(Error::new(
                ErrorKind::CorruptedFileFooter,
                format!("Footer marker inválido: expected 0xFE, got 0x{:02X}", bytes[0]),
            ));
        }

        let num_blocks =
            (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | (bytes[3] as u32);
        let total_raw_size = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let footer_checksum = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Self {
            marker: Self::MARKER,
            num_blocks,
            total_raw_size,
            footer_checksum,
        })
    }
}
```

#### 3.3 `format/constants.rs`

```rust
// src/format/constants.rs

// Magic Numbers
pub const MAGIC_NUMBER_PRIMARY: u16 = 0x435A;    // "CZ"
pub const MAGIC_NUMBER_ALT: u16 = 0x5A43;      // "ZC"

// Versión
pub const VERSION_MAJOR: u8 = 1;
pub const VERSION_MINOR: u8 = 0;

// Límites
pub const MIN_BLOCK_SIZE_LOG2: u16 = 9;         // 512 bytes
pub const MAX_BLOCK_SIZE_LOG2: u16 = 16;        // 64 KiB
pub const MIN_MAX_EXPANSION: u16 = 100;         // 1x
pub const MAX_MAX_EXPANSION: u16 = 5000;        // 50x

pub const MAX_EXPANSION_DEFAULT: u16 = 1000;    // 10x

// Algoritmos de compresión
pub mod algorithms {
    pub const STORE: u8 = 0;
    pub const LZ77_HUFFMAN: u8 = 1;
    pub const LZ4_STYLE: u8 = 2;
    pub const LZMA_STYLE: u8 = 3;
    pub const DEFLATE_STYLE: u8 = 4;
    pub const RESERVED_5_14: u8 = 5;
    pub const EXPERIMENTAL: u8 = 15;
}

// Bloques
pub const BLOCK_TYPE_DATA: u8 = 0;
pub const BLOCK_TYPE_METADATA: u8 = 1;
pub const BLOCK_TYPE_INCOMPLETE: u8 = 2;
pub const BLOCK_TYPE_RESERVED: u8 = 3;

// Tamaños de header/footer
pub const FILE_HEADER_SIZE: usize = 16;
pub const BLOCK_HEADER_SIZE: usize = 12;
pub const FILE_FOOTER_SIZE: usize = 12;
pub const CRC32_SIZE: usize = 4;
pub const CRC64_SIZE: usize = 8;
```

#### 3.4 `format/checksum.rs`

```rust
// src/format/checksum.rs

pub struct Crc32;
pub struct Crc64;

impl Crc32 {
    const POLYNOMIAL: u32 = 0xEDB88320;

    /// Calcular CRC-32 ISO 3309
    pub fn compute(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFF;

        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                crc = if (crc & 1) == 1 {
                    (crc >> 1) ^ Self::POLYNOMIAL
                } else {
                    crc >> 1
                };
            }
        }

        crc ^ 0xFFFFFFFF
    }

    /// Verificar CRC-32
    pub fn verify(data: &[u8], expected: u32) -> bool {
        Self::compute(data) == expected
    }
}

impl Crc64 {
    const POLYNOMIAL: u64 = 0x42F0E1EBA9EA3693;

    /// Calcular CRC-64 ECMA
    pub fn compute(data: &[u8]) -> u64 {
        let mut crc = 0xFFFFFFFFFFFFFFFF;

        for &byte in data {
            crc ^= (byte as u64) << 56;
            for _ in 0..8 {
                crc = if (crc & 0x8000000000000000) != 0 {
                    (crc << 1) ^ Self::POLYNOMIAL
                } else {
                    crc << 1
                };
            }
        }

        crc ^ 0xFFFFFFFFFFFFFFFF
    }

    /// Verificar CRC-64
    pub fn verify(data: &[u8], expected: u64) -> bool {
        Self::compute(data) == expected
    }
}
```

---

## 🎯 Patrones de Codificación

### 1. Validación Temprana (Fail Fast)

```rust
// ✅ CORRECTO: Validar en el constructor
pub fn new(size: usize) -> Result<Self> {
    if size == 0 {
        return Err(Error::new(
            ErrorKind::InvalidBlockSize,
            "Size cannot be zero",
        ));
    }
    Ok(Self { size })
}

// ❌ INCORRECTO: Validar más tarde
pub fn new(size: usize) -> Self {
    Self { size }
}
pub fn validate(&self) -> Result<()> {
    if self.size == 0 {
        return Err(...);
    }
    Ok(())
}
```

### 2. Evitar Panic en Hot Paths

```rust
// ✅ CORRECTO: Usar Result
pub fn decompress_block(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < MIN_BLOCK_SIZE {
        return Err(Error::new(
            ErrorKind::CorruptedBlockHeader,
            "Block too small",
        ));
    }
    // ...
}

// ❌ INCORRECTO: Usar unwrap/expect
pub fn decompress_block(data: &[u8]) -> Vec<u8> {
    let header = BlockHeader::from_bytes(data[..12].try_into().unwrap()); // BOOM!
    // ...
}
```

### 3. Manejo de Streaming

```rust
// ✅ CORRECTO: Procesar por chunks sin cargar todo
pub fn decompress_file<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    max_memory: usize,
) -> Result<()> {
    let mut total_decompressed = 0;

    loop {
        // Leer bloque
        let mut block_buf = vec![0u8; BLOCK_HEADER_SIZE];
        if reader.read(&mut block_buf)? == 0 {
            break; // EOF
        }

        // Descomprimir
        let decompressed = decompress_block(&block_buf)?;
        total_decompressed += decompressed.len();

        // Validar memoria
        if total_decompressed > max_memory {
            return Err(Error::new(
                ErrorKind::MemoryLimitExceeded,
                "Decompression exceeds memory limit",
            ));
        }

        // Escribir
        writer.write_all(&decompressed)?;
    }

    Ok(())
}

// ❌ INCORRECTO: Cargar todo en memoria
pub fn decompress_file(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    for chunk in data.chunks(BLOCK_SIZE) {
        output.extend(decompress_block(chunk)?);
    }
    Ok(output)
}
```

### 4. Manejo de Errores Contextual

```rust
// ✅ CORRECTO: Agregar contexto a errores
match file_header.validate() {
    Ok(_) => {},
    Err(e) => {
        return Err(e.with_context(format!(
            "Al leer header en offset {}",
            offset
        )));
    }
}

// También se puede hacer:
file_header.validate()
    .map_err(|e| e.with_context(format!("offset: {}", offset)))?;
```

---

## 📊 Gestión de Errores

Todos los errores deben ser capturados y reportados de forma clara.

### Implementar Error Handler

```rust
// src/error_handler.rs

pub trait ErrorHandler {
    fn handle(&self, error: &Error);
}

pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle(&self, error: &Error) {
        eprintln!("{}", error);
    }
}

pub struct LogErrorHandler {
    log_file: std::fs::File,
}

impl ErrorHandler for LogErrorHandler {
    fn handle(&self, error: &Error) {
        // Escribir a log
    }
}
```

---

## 🧪 Testing y Validación

### Estructura de Tests

```
tests/
├── basic.rs              # Tests básicos (compress/decompress)
├── large_files.rs        # Tests con archivos grandes
├── corruption.rs         # Tests de detección de corrupción
└── performance.rs        # Tests de rendimiento

fuzz/
├── fuzz_targets/
│   ├── fuzz_decompressor.rs
│   ├── fuzz_compressor.rs
│   └── fuzz_header_parser.rs
```

### Ejemplo de Test

```rust
// tests/basic.rs

#[test]
fn test_roundtrip_small_file() {
    let original = b"Hello, CsZip! This is a test file.";

    // Comprimir
    let compressed = CsZip::compress(original, 6)
        .expect("Compression failed");

    // Descomprimir
    let decompressed = CsZip::decompress(&compressed)
        .expect("Decompression failed");

    // Verificar
    assert_eq!(original, decompressed.as_slice());
}

#[test]
fn test_detects_corrupted_header() {
    let mut corrupted = vec![0xFF, 0xFF]; // Magic inválido
    corrupted.extend_from_slice(&[0u8; 14]);

    let result = CsZip::decompress(&corrupted);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidMagicNumber);
}

#[test]
fn test_large_file_streaming() {
    let size = 100 * 1024 * 1024; // 100 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    // Usar streaming para no sobrecargar memoria
    let compressed = CsZip::compress(&data, 6)
        .expect("Compression failed");

    let decompressed = CsZip::decompress(&compressed)
        .expect("Decompression failed");

    assert_eq!(data, decompressed);
}
```

---

<div align="center">

**Arquitectura CsZip — Guía Completa de Implementación**

Seguir estos patrones garantiza código seguro, mantenible y auditable.

</div>
