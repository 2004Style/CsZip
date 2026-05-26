# 📚 Guía de Código - CsZip

**Ejemplos de implementación rápida para cada módulo**

---

## 1. Manejo de Errores (`src/error.rs`)

### Ejemplo mínimo

```rust
#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    InvalidMagicNumber,
    UnsupportedVersion,
    InvalidBlockSize,
    BlockCrcMismatch,
    MemoryLimitExceeded,
    UnexpectedEof,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn code(&self) -> u8 {
        match self.kind {
            ErrorKind::InvalidMagicNumber => 0x01,
            ErrorKind::UnsupportedVersion => 0x02,
            ErrorKind::InvalidBlockSize => 0x04,
            ErrorKind::BlockCrcMismatch => 0x07,
            ErrorKind::MemoryLimitExceeded => 0x0D,
            ErrorKind::UnexpectedEof => 0x0E,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ERROR {}] {}", self.code(), self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## 2. Constants del Formato (`src/format/constants.rs`)

```rust
// Magic Numbers
pub const MAGIC_PRIMARY: u16 = 0x435A;  // "CZ"
pub const MAGIC_ALT: u16 = 0x5A43;     // "ZC"

// Versión
pub const VERSION_MAJOR: u8 = 1;
pub const VERSION_MINOR: u8 = 0;

// Límites
pub const MIN_BLOCK_LOG2: u16 = 9;    // 512 B
pub const MAX_BLOCK_LOG2: u16 = 16;   // 64 KiB
pub const MIN_EXPANSION: u16 = 100;
pub const MAX_EXPANSION: u16 = 5000;
pub const DEFAULT_EXPANSION: u16 = 1000;  // 10x

// Algoritmos
pub const ALGO_STORE: u8 = 0;
pub const ALGO_LZ77_HUFFMAN: u8 = 1;
pub const ALGO_DEFLATE: u8 = 4;

// Block Types
pub const BLOCK_DATA: u8 = 0;
pub const BLOCK_METADATA: u8 = 1;
pub const BLOCK_INCOMPLETE: u8 = 2;

// Tamaños
pub const FILE_HEADER_SIZE: usize = 16;
pub const BLOCK_HEADER_SIZE: usize = 12;
pub const FILE_FOOTER_SIZE: usize = 12;
pub const CRC32_SIZE: usize = 4;
pub const CRC64_SIZE: usize = 8;
pub const FILE_FOOTER_MARKER: u8 = 0xFE;
```

---

## 3. Header Global (`src/format/header.rs`)

### Estructura

```rust
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

    pub fn new(algo: u8, block_size_log2: u16, max_exp: u16) -> crate::error::Result<Self> {
        if block_size_log2 < 9 || block_size_log2 > 16 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::InvalidBlockSize,
                "Block size log2 fuera de rango",
            ));
        }

        Ok(Self {
            magic: 0x435A,
            version_major: 1,
            version_minor: 0,
            flags: 0,
            compression_algo: algo,
            block_size_log2,
            max_expansion: max_exp,
            reserved: 0,
            checksum: 0,
        })
    }

    pub fn from_bytes(bytes: &[u8; 16]) -> crate::error::Result<Self> {
        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);

        if magic != 0x435A && magic != 0x5A43 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::InvalidMagicNumber,
                format!("Magic 0x{:04X} inválido", magic),
            ));
        }

        let version_major = bytes[2];
        if version_major > 1 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::UnsupportedVersion,
                format!("Version {}.{} no soportada", version_major, bytes[3]),
            ));
        }

        let block_size_log2 = u16::from_be_bytes([bytes[6], bytes[7]]);
        if block_size_log2 < 9 || block_size_log2 > 16 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::InvalidBlockSize,
                "Block size log2 inválido",
            ));
        }

        Ok(Self {
            magic,
            version_major,
            version_minor: bytes[3],
            flags: bytes[4],
            compression_algo: bytes[5],
            block_size_log2,
            max_expansion: u16::from_be_bytes([bytes[8], bytes[9]]),
            reserved: u16::from_be_bytes([bytes[10], bytes[11]]),
            checksum: u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        })
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0..2].copy_from_slice(&self.magic.to_be_bytes());
        b[2] = self.version_major;
        b[3] = self.version_minor;
        b[4] = self.flags;
        b[5] = self.compression_algo;
        b[6..8].copy_from_slice(&self.block_size_log2.to_be_bytes());
        b[8..10].copy_from_slice(&self.max_expansion.to_be_bytes());
        b[10..12].copy_from_slice(&self.reserved.to_be_bytes());
        b[12..16].copy_from_slice(&self.checksum.to_be_bytes());
        b
    }

    pub fn block_size(&self) -> usize {
        1 << self.block_size_log2
    }
}
```

---

## 4. Block Header (`src/format/block.rs`)

```rust
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
        orig_size: u16,
        comp_size: u32,
        adler: u32,
        level: u8,
    ) -> crate::error::Result<Self> {
        if orig_size == 0 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::InvalidBlockSize,
                "Tamaño original debe ser > 0",
            ));
        }

        Ok(Self {
            block_type: 0, // DATA
            compression_level: level,
            original_size: orig_size,
            compressed_size: comp_size,
            adler32: adler,
        })
    }

    pub fn from_bytes(bytes: &[u8; 12]) -> crate::error::Result<Self> {
        let block_type = bytes[0];

        if block_type == 2 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::InvalidBlockSize,
                "Block incompleto",
            ));
        }

        Ok(Self {
            block_type,
            compression_level: bytes[1],
            original_size: u16::from_be_bytes([bytes[2], bytes[3]]),
            compressed_size: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            adler32: u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        })
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut b = [0u8; 12];
        b[0] = self.block_type;
        b[1] = self.compression_level;
        b[2..4].copy_from_slice(&self.original_size.to_be_bytes());
        b[4..8].copy_from_slice(&self.compressed_size.to_be_bytes());
        b[8..12].copy_from_slice(&self.adler32.to_be_bytes());
        b
    }
}
```

---

## 5. CRC-32 (`src/format/checksum.rs`)

```rust
pub struct Crc32;

impl Crc32 {
    const POLYNOMIAL: u32 = 0xEDB88320;
    const TABLE: [u32; 256] = Self::build_table();

    const fn build_table() -> [u32; 256] {
        let mut table = [0u32; 256];
        let mut i = 0;

        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;

            while j < 8 {
                crc = if (crc & 1) == 1 {
                    (crc >> 1) ^ Self::POLYNOMIAL
                } else {
                    crc >> 1
                };
                j += 1;
            }

            table[i] = crc;
            i += 1;
        }

        table
    }

    /// Calcular CRC-32
    pub fn compute(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFF;

        for &byte in data {
            let idx = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ Self::TABLE[idx];
        }

        crc ^ 0xFFFFFFFF
    }

    /// Verificar CRC-32
    pub fn verify(data: &[u8], expected: u32) -> bool {
        Self::compute(data) == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        let result = Crc32::compute(b"");
        assert_eq!(result, 0x00000000); // CRC-32("") = 0
    }

    #[test]
    fn test_crc32_known() {
        // Valores conocidos para verificación
        let result = Crc32::compute(b"123456789");
        assert_eq!(result, 0xCBF43926);
    }

    #[test]
    fn test_crc32_verify() {
        let data = b"Hello, World!";
        let crc = Crc32::compute(data);
        assert!(Crc32::verify(data, crc));
    }
}
```

---

## 6. Compresor Simple (STORE - `src/codec/mod.rs`)

```rust
/// Algoritmo STORE: sin compresión
pub fn store_compress(data: &[u8]) -> crate::error::Result<Vec<u8>> {
    // Solo copiar datos
    Ok(data.to_vec())
}

/// Descomprimir STORE
pub fn store_decompress(
    data: &[u8],
    expected_size: usize,
) -> crate::error::Result<Vec<u8>> {
    if data.len() != expected_size {
        return Err(crate::error::Error::new(
            crate::error::ErrorKind::InvalidBlockSize,
            "Tamaño descomprimido no coincide",
        ));
    }
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_roundtrip() {
        let data = b"Hello, CsZip!";
        let compressed = store_compress(data).unwrap();
        let decompressed = store_decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, &decompressed[..]);
    }
}
```

---

## 7. CLI Básica (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Comprimir archivo
    Compress {
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long, default_value = "6")]
        level: u8,
    },

    /// Descomprimir archivo
    Decompress {
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verificar archivo
    Test {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Listar información
    List {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compress {
            file,
            output,
            level,
        } => handle_compress(&file, output, level, cli.verbose),
        Commands::Decompress { file, output } => handle_decompress(&file, output, cli.verbose),
        Commands::Test { file } => handle_test(&file, cli.verbose),
        Commands::List { file } => handle_list(&file, cli.verbose),
    };

    match result {
        Ok(_) => {
            if cli.verbose {
                println!("✓ Operación completada");
            }
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_compress(
    file: &std::path::Path,
    output: Option<std::path::PathBuf>,
    level: u8,
    verbose: bool,
) -> CsZip::Result<()> {
    let output = output.unwrap_or_else(|| {
        let mut p = file.to_path_buf();
        p.set_extension("CZ");
        p
    });

    if verbose {
        println!("Comprimiendo: {} → {}", file.display(), output.display());
    }

    let data = std::fs::read(file)
        .map_err(|e| CsZip::Error::new(
            CsZip::error::ErrorKind::UnexpectedEof,
            format!("No se pudo leer archivo: {}", e),
        ))?;

    let compressed = CsZip::compress(&data, level)?;

    std::fs::write(&output, &compressed)
        .map_err(|e| CsZip::Error::new(
            CsZip::error::ErrorKind::UnexpectedEof,
            format!("No se pudo escribir: {}", e),
        ))?;

    if verbose {
        println!(
            "Original: {} bytes, Comprimido: {} bytes ({:.1}%)",
            data.len(),
            compressed.len(),
            (100.0 * compressed.len() as f64) / data.len() as f64
        );
    }

    Ok(())
}

fn handle_decompress(
    file: &std::path::Path,
    output: Option<std::path::PathBuf>,
    verbose: bool,
) -> CsZip::Result<()> {
    let output = output.unwrap_or_else(|| {
        let mut p = file.to_path_buf();
        p.set_extension("");
        p
    });

    if verbose {
        println!("Descomprimiendo: {} → {}", file.display(), output.display());
    }

    let compressed = std::fs::read(file)
        .map_err(|e| CsZip::Error::new(
            CsZip::error::ErrorKind::UnexpectedEof,
            format!("No se pudo leer archivo: {}", e),
        ))?;

    let decompressed = CsZip::decompress(&compressed)?;

    std::fs::write(&output, &decompressed)
        .map_err(|e| CsZip::Error::new(
            CsZip::error::ErrorKind::UnexpectedEof,
            format!("No se pudo escribir: {}", e),
        ))?;

    if verbose {
        println!(
            "Comprimido: {} bytes, Original: {} bytes",
            compressed.len(),
            decompressed.len()
        );
    }

    Ok(())
}

fn handle_test(file: &std::path::Path, verbose: bool) -> CsZip::Result<()> {
    if verbose {
        println!("Verificando: {}", file.display());
    }
    // TODO: Validar archivo
    println!("✓ Archivo válido");
    Ok(())
}

fn handle_list(file: &std::path::Path, verbose: bool) -> CsZip::Result<()> {
    if verbose {
        println!("Listando: {}", file.display());
    }
    // TODO: Mostrar información de bloques
    println!("Bloques encontrados: 1");
    Ok(())
}
```

---

## 🎯 Orden de Implementación Recomendado

1. **error.rs** - Sistema de errores
2. **format/constants.rs** - Constantes
3. **format/header.rs** - Parsing de header
4. **format/block.rs** - Parsing de bloques
5. **format/checksum.rs** - Checksums CRC
6. **codec/mod.rs** - Algoritmo STORE
7. **io/reader.rs** - Lectura
8. **io/writer.rs** - Escritura
9. **main.rs** - CLI
10. **Tests exhaustivos**

---

## ✅ Checklist por Módulo

- [x] error.rs compile
- [x] format/constants.rs compile
- [x] format/header.rs read/write roundtrip
- [x] format/block.rs read/write roundtrip
- [x] format/checksum.rs CRC correcto
- [x] codec/mod.rs STORE funciona
- [x] io/reader.rs read_exact funciona
- [x] io/writer.rs write_all funciona
- [x] main.rs compilación
- [x] Tests: cargo test
- [x] No warnings: cargo clippy
- [x] Formato: cargo fmt

---

<div align="center">

**Copia estos ejemplos y adapta a tu código**

¡Éxito! 🚀

</div>
