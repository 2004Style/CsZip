//! Módulo de codecs de compresión/descompresión
//!
//! Implementa los algoritmos de compresión soportados por el formato CsZip.
//!
//! # Algoritmos soportados
//!
//! | ID | Nombre | Descripción |
//! |----|--------|-------------|
//! | 0  | STORE  | Sin compresión (copia directa) |
//! | 1  | LZ77+Huffman | Compresión con ventana deslizante + Huffman |
//! | 2  | LZ4    | Compresión LZ4 rápida |
//! | 3  | LZMA   | Compresión LZMA de alta ratio |
//! | 4  | DEFLATE | Compresión DEFLATE (zlib) |

pub mod compressor;
pub mod decompressor;
pub mod filters;
pub mod huffman;
pub mod lz77;

pub use compressor::Compressor;
pub use decompressor::Decompressor;
pub use filters::{Filter, FilterType};
pub use huffman::{HuffmanDecoder, HuffmanEncoder};
pub use lz77::{Lz77Compressor, Lz77Decompressor, Lz77Config, Lz77Token};

use crate::error::{Error, ErrorKind};
use crate::format::constants;

/// Algoritmo de compresión
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Algorithm {
    /// Sin compresión (copia directa)
    Store = 0,
    /// LZ77 con codificación Huffman
    Lz77Huffman = 1,
    /// LZ4 rápido
    Lz4 = 2,
    /// LZMA alta compresión
    Lzma = 3,
    /// DEFLATE (zlib compatible)
    Deflate = 4,
}

impl Algorithm {
    /// Crea un algoritmo desde su identificador numérico
    pub fn from_id(id: u8) -> Result<Self, Error> {
        match id {
            constants::ALGORITHM_STORE => Ok(Algorithm::Store),
            constants::ALGORITHM_LZ77_HUFFMAN => Ok(Algorithm::Lz77Huffman),
            constants::ALGORITHM_LZ4 => Ok(Algorithm::Lz4),
            constants::ALGORITHM_LZMA => Ok(Algorithm::Lzma),
            constants::ALGORITHM_DEFLATE => Ok(Algorithm::Deflate),
            _ => Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                format!("Algoritmo no soportado: {}", id),
            )),
        }
    }

    /// Retorna el identificador numérico del algoritmo
    pub fn id(&self) -> u8 {
        *self as u8
    }

    /// Nombre legible del algoritmo
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Store => "STORE",
            Algorithm::Lz77Huffman => "LZ77+Huffman",
            Algorithm::Lz4 => "LZ4",
            Algorithm::Lzma => "LZMA",
            Algorithm::Deflate => "DEFLATE",
        }
    }

    /// Indica si el algoritmo está implementado
    pub fn is_implemented(&self) -> bool {
        matches!(self, Algorithm::Store)
    }
}

impl TryFrom<u8> for Algorithm {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_id(value)
    }
}

impl From<Algorithm> for u8 {
    fn from(alg: Algorithm) -> u8 {
        alg.id()
    }
}

/// Nivel de compresión (0-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionLevel(u8);

impl CompressionLevel {
    /// Nivel mínimo de compresión (más rápido)
    pub const MIN: CompressionLevel = CompressionLevel(0);
    /// Nivel por defecto
    pub const DEFAULT: CompressionLevel = CompressionLevel(6);
    /// Nivel máximo de compresión (más lento)
    pub const MAX: CompressionLevel = CompressionLevel(9);

    /// Crea un nivel de compresión validado
    pub fn new(level: u8) -> Result<Self, Error> {
        if level > 9 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Nivel de compresión inválido: {} (debe ser 0-9)", level),
            ));
        }
        Ok(CompressionLevel(level))
    }

    /// Obtiene el valor numérico del nivel
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryFrom<u8> for CompressionLevel {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_from_id() {
        assert_eq!(Algorithm::from_id(0).unwrap(), Algorithm::Store);
        assert_eq!(Algorithm::from_id(1).unwrap(), Algorithm::Lz77Huffman);
        assert_eq!(Algorithm::from_id(2).unwrap(), Algorithm::Lz4);
        assert_eq!(Algorithm::from_id(3).unwrap(), Algorithm::Lzma);
        assert_eq!(Algorithm::from_id(4).unwrap(), Algorithm::Deflate);
        assert!(Algorithm::from_id(5).is_err());
        assert!(Algorithm::from_id(255).is_err());
    }

    #[test]
    fn test_algorithm_roundtrip() {
        for id in 0..=4u8 {
            let alg = Algorithm::from_id(id).unwrap();
            assert_eq!(alg.id(), id);
        }
    }

    #[test]
    fn test_algorithm_names() {
        assert_eq!(Algorithm::Store.name(), "STORE");
        assert_eq!(Algorithm::Lz77Huffman.name(), "LZ77+Huffman");
        assert_eq!(Algorithm::Lz4.name(), "LZ4");
        assert_eq!(Algorithm::Lzma.name(), "LZMA");
        assert_eq!(Algorithm::Deflate.name(), "DEFLATE");
    }

    #[test]
    fn test_algorithm_implemented() {
        assert!(Algorithm::Store.is_implemented());
        assert!(!Algorithm::Lz77Huffman.is_implemented());
        assert!(!Algorithm::Lz4.is_implemented());
        assert!(!Algorithm::Lzma.is_implemented());
        assert!(!Algorithm::Deflate.is_implemented());
    }

    #[test]
    fn test_compression_level() {
        assert!(CompressionLevel::new(0).is_ok());
        assert!(CompressionLevel::new(9).is_ok());
        assert!(CompressionLevel::new(10).is_err());
        assert!(CompressionLevel::new(255).is_err());
    }

    #[test]
    fn test_compression_level_default() {
        assert_eq!(CompressionLevel::default().value(), 6);
    }

    #[test]
    fn test_compression_level_constants() {
        assert_eq!(CompressionLevel::MIN.value(), 0);
        assert_eq!(CompressionLevel::DEFAULT.value(), 6);
        assert_eq!(CompressionLevel::MAX.value(), 9);
    }
}
