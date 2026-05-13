//! # CsZip - Sistema de Compresión y Descompresión de Archivos
//!
//! CsZip es una herramienta de compresión sin pérdidas con un formato binario
//! personalizado (extensión `.cz`).
//!
//! ## Características
//!
//! - Formato binario eficiente con compresión por bloques
//! - Múltiples algoritmos de compresión (actualmente STORE implementado)
//! - Verificación de integridad con CRC-32/CRC-64
//! - API de alto nivel para compresión/descompresión
//! - Protección contra zip bombs
//!
//! ## Uso rápido
//!
//! ```rust,no_run
//! use cszip::io::{CzReader, CzWriter};
//! use std::io::Cursor;
//!
//! // Comprimir datos
//! let data = b"Hello, CsZip!";
//! let mut buffer = Vec::new();
//! {
//!     let cursor = Cursor::new(&mut buffer);
//!     let mut writer = CzWriter::new(cursor).unwrap();
//!     writer.write_block(data).unwrap();
//!     writer.finish().unwrap();
//! }
//!
//! // Descomprimir datos
//! let cursor = Cursor::new(&buffer);
//! let mut reader = CzReader::new(cursor).unwrap();
//! let block = reader.read_block().unwrap().unwrap();
//! assert_eq!(block.data, data);
//! ```
//!
//! ## Formato de archivo
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │   File Header (16 bytes)        │
//! │   - Magic: 0x435A ("CZ")        │
//! │   - Version, Algorithm, Flags   │
//! ├─────────────────────────────────┤
//! │   Block 0                       │
//! │   ├── Block Header (12 bytes)   │
//! │   └── Compressed Data           │
//! ├─────────────────────────────────┤
//! │   Block 1 ...                   │
//! ├─────────────────────────────────┤
//! │   File Footer (12 bytes)        │
//! │   - Block count, Size, CRC      │
//! └─────────────────────────────────┘
//! ```

#![warn(missing_docs)]

pub mod cli;
pub mod codec;
pub mod error;
pub mod format;
pub mod io;
pub mod utils;

// Re-exportar tipos principales
pub use codec::{Algorithm, CompressionLevel, Compressor, Decompressor};
pub use error::{Error, ErrorKind};
pub use format::{BlockHeader, FileFooter, Header};
pub use io::{CzReader, CzWriter};

/// Versión de la biblioteca
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Nombre del programa
pub const NAME: &str = "cszip";

/// Extensión de archivo por defecto
pub const EXTENSION: &str = "cz";

/// Comprime un archivo a formato .cz
///
/// # Argumentos
///
/// * `input` - Ruta del archivo a comprimir
/// * `output` - Ruta del archivo de salida (opcional)
///
/// # Ejemplo
///
/// ```rust,no_run
/// use std::path::Path;
/// use cszip::compress_file;
///
/// compress_file(Path::new("data.txt"), Some(Path::new("data.cz"))).unwrap();
/// ```
pub fn compress_file(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
) -> Result<io::writer::WriteStats, Error> {
    use std::fs::File;
    use std::io::BufReader;

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let mut p = input.to_path_buf();
            p.set_extension(EXTENSION);
            p
        }
    };

    let input_file = File::open(input).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error abriendo archivo: {}", e),
        )
    })?;

    let mut reader = BufReader::new(input_file);
    let mut writer = CzWriter::create(&output_path)?;

    writer.write_stream(&mut reader)?;
    writer.finish()
}

/// Descomprime un archivo .cz
///
/// # Argumentos
///
/// * `input` - Ruta del archivo .cz
/// * `output` - Ruta del archivo de salida (opcional)
///
/// # Ejemplo
///
/// ```rust,no_run
/// use std::path::Path;
/// use cszip::decompress_file;
///
/// decompress_file(Path::new("data.cz"), Some(Path::new("data.txt"))).unwrap();
/// ```
pub fn decompress_file(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
) -> Result<io::reader::ReadStats, Error> {
    use std::fs::File;
    use std::io::BufWriter;

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let mut p = input.to_path_buf();
            p.set_extension("");
            if p.extension().is_none() {
                p.set_extension("out");
            }
            p
        }
    };

    let mut reader = CzReader::open(input)?;

    let output_file = File::create(&output_path).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error creando archivo: {}", e),
        )
    })?;

    let mut writer = BufWriter::new(output_file);
    reader.decompress_all(&mut writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "cszip");
    }

    #[test]
    fn test_extension() {
        assert_eq!(EXTENSION, "cz");
    }

    #[test]
    fn test_roundtrip_memory() {
        let original = b"CsZip library test data for roundtrip verification!";

        // Comprimir
        let mut compressed = Vec::new();
        {
            let cursor = Cursor::new(&mut compressed);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }

        // Descomprimir
        let cursor = Cursor::new(&compressed);
        let mut reader = CzReader::new(cursor).unwrap();
        let block = reader.read_block().unwrap().unwrap();

        assert_eq!(block.data, original);
    }
}
