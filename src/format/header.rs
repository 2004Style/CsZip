//! Estructura y parseo del header global de archivos CsZip
//!
//! El header global tiene 16 bytes y contiene la configuración
//! del archivo comprimido.

use crate::error::{Error, ErrorKind, Result};
use crate::format::constants::*;
use std::io::{Read, Write};

/// Header global de un archivo CsZip (16 bytes)
///
/// ```text
/// Offset  Tamaño  Campo               Tipo
/// ──────  ──────  ─────────────────  ──────────
/// 0-1     2       Magic Number       u16 BE
/// 2       1       Version Major      u8
/// 3       1       Version Minor      u8
/// 4       1       Flags              u8
/// 5       1       Compression Algo   u8
/// 6-7     2       Block Size Log2    u16 BE
/// 8-9     2       Max Expansion %    u16 BE
/// 10-11   2       Reserved           u16 BE
/// 12-15   4       Header Checksum    u32 BE
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Magic number (0x435A o 0x5A43)
    pub magic: u16,
    /// Versión mayor del formato
    pub version_major: u8,
    /// Versión menor del formato
    pub version_minor: u8,
    /// Flags de configuración
    pub flags: u8,
    /// Algoritmo de compresión usado
    pub compression_algo: u8,
    /// Log₂ del tamaño de bloque
    pub block_size_log2: u16,
    /// Máximo porcentaje de expansión permitido
    pub max_expansion: u16,
    /// Bytes reservados para uso futuro
    pub reserved: u16,
    /// Checksum del header
    pub checksum: u32,
}

impl Header {
    /// Tamaño del header en bytes
    pub const SIZE: usize = FILE_HEADER_SIZE;

    /// Crear un nuevo header con valores por defecto
    ///
    /// # Argumentos
    ///
    /// * `compression_algo` - Algoritmo de compresión (0-14)
    /// * `block_size_log2` - Log₂ del tamaño de bloque (9-30)
    /// * `max_expansion` - Máximo % de expansión (100-5000)
    ///
    /// # Errores
    ///
    /// Retorna error si los parámetros están fuera de rango.
    pub fn new(compression_algo: u8, block_size_log2: u16, max_expansion: u16) -> Result<Self> {
        Self::with_flags(compression_algo, block_size_log2, max_expansion, 0)
    }

    /// Crear un nuevo header con flags personalizados
    pub fn with_flags(
        compression_algo: u8,
        block_size_log2: u16,
        max_expansion: u16,
        flags: u8,
    ) -> Result<Self> {
        // Validar algoritmo
        if compression_algo == ALGO_EXPERIMENTAL {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                "Algoritmo experimental (15) no soportado",
            ));
        }
        if compression_algo > ALGO_EXPERIMENTAL {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                format!("Algoritmo {} desconocido", compression_algo),
            ));
        }

        // Validar tamaño de bloque
        if !(MIN_BLOCK_SIZE_LOG2..=MAX_BLOCK_SIZE_LOG2).contains(&block_size_log2) {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!(
                    "Block size log2 {} fuera de rango [{}, {}]",
                    block_size_log2, MIN_BLOCK_SIZE_LOG2, MAX_BLOCK_SIZE_LOG2
                ),
            ));
        }

        // Validar expansión
        if !(MIN_EXPANSION..=MAX_EXPANSION).contains(&max_expansion) {
            return Err(Error::new(
                ErrorKind::InvalidExpansionLimit,
                format!(
                    "Max expansion {} fuera de rango [{}, {}]",
                    max_expansion, MIN_EXPANSION, MAX_EXPANSION
                ),
            ));
        }

        // Validar flags (bits 4-7 deben ser 0)
        if (flags & FLAG_RESERVED_MASK) != 0 {
            return Err(Error::new(
                ErrorKind::InvalidBlockType,
                "Flags contienen bits reservados",
            ));
        }

        Ok(Self {
            magic: MAGIC_PRIMARY,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags,
            compression_algo,
            block_size_log2,
            max_expansion,
            reserved: 0,
            checksum: 0,
        })
    }

    /// Crear header con valores por defecto para algoritmo STORE
    pub fn default_store() -> Self {
        Self {
            magic: MAGIC_PRIMARY,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags: 0,
            compression_algo: ALGO_STORE,
            block_size_log2: DEFAULT_BLOCK_SIZE_LOG2,
            max_expansion: DEFAULT_EXPANSION,
            reserved: 0,
            checksum: 0,
        }
    }

    /// Leer header desde un stream
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::SIZE];
        reader.read_exact(&mut buf).map_err(|e| {
            Error::new(
                ErrorKind::UnexpectedEof,
                format!("No se pudo leer header: {}", e),
            )
        })?;

        Self::from_bytes(&buf)
    }

    /// Parsear header desde bytes
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);

        // Validar magic number
        if magic != MAGIC_PRIMARY && magic != MAGIC_ALT {
            return Err(Error::new(
                ErrorKind::InvalidMagicNumber,
                format!("Magic number 0x{:04X} inválido", magic),
            ));
        }

        let version_major = bytes[2];
        let version_minor = bytes[3];

        // Validar versión
        if version_major > VERSION_MAJOR {
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

        // Validar algoritmo
        if compression_algo == ALGO_EXPERIMENTAL {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                "Algoritmo experimental (15) no soportado",
            ));
        }

        // Validar tamaño de bloque
        if !(MIN_BLOCK_SIZE_LOG2..=MAX_BLOCK_SIZE_LOG2).contains(&block_size_log2) {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!("Block size log2 {} fuera de rango", block_size_log2),
            ));
        }

        // Validar expansión
        if !(MIN_EXPANSION..=MAX_EXPANSION).contains(&max_expansion) {
            return Err(Error::new(
                ErrorKind::InvalidExpansionLimit,
                format!("Max expansion {} fuera de rango", max_expansion),
            ));
        }

        // Validar flags (bits 4-7 deben ser 0)
        if (flags & FLAG_RESERVED_MASK) != 0 {
            return Err(Error::new(
                ErrorKind::InvalidBlockType,
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

    /// Serializar header a bytes
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

    /// Escribir header a un stream
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(&self.to_bytes())
            .map_err(|e| Error::new(ErrorKind::Io, format!("Error escribiendo header: {}", e)))
    }

    /// Obtener tamaño de bloque en bytes
    #[inline]
    pub fn block_size(&self) -> usize {
        1 << self.block_size_log2
    }

    /// ¿Usa CRC-64 en lugar de CRC-32?
    #[inline]
    pub fn uses_crc64(&self) -> bool {
        (self.flags & FLAG_USE_CRC64) != 0
    }

    /// ¿Tiene metadata extra?
    #[inline]
    pub fn has_extra_metadata(&self) -> bool {
        (self.flags & FLAG_HAS_EXTRA_METADATA) != 0
    }

    /// Obtener tamaño del checksum de bloque en bytes
    #[inline]
    pub fn block_checksum_size(&self) -> usize {
        if self.uses_crc64() {
            CRC64_SIZE
        } else {
            CRC32_SIZE
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::default_store()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_new_valid() {
        let header = Header::new(ALGO_STORE, 15, 1000).unwrap();
        assert_eq!(header.magic, MAGIC_PRIMARY);
        assert_eq!(header.version_major, VERSION_MAJOR);
        assert_eq!(header.compression_algo, ALGO_STORE);
        assert_eq!(header.block_size_log2, 15);
        assert_eq!(header.max_expansion, 1000);
    }

    #[test]
    fn test_header_invalid_block_size() {
        // Muy pequeño
        assert!(Header::new(ALGO_STORE, 5, 1000).is_err());
        // Muy grande (> 16)
        assert!(Header::new(ALGO_STORE, 17, 1000).is_err());
    }

    #[test]
    fn test_header_invalid_expansion() {
        // Muy pequeño
        assert!(Header::new(ALGO_STORE, 15, 50).is_err());
        // Muy grande
        assert!(Header::new(ALGO_STORE, 15, 6000).is_err());
    }

    #[test]
    fn test_header_invalid_algorithm() {
        // Experimental
        assert!(Header::new(ALGO_EXPERIMENTAL, 15, 1000).is_err());
    }

    #[test]
    fn test_header_roundtrip() {
        let original = Header::new(ALGO_STORE, 15, 1000).unwrap();
        let bytes = original.to_bytes();
        let parsed = Header::from_bytes(&bytes).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_header_block_size() {
        let header = Header::new(ALGO_STORE, 15, 1000).unwrap();
        assert_eq!(header.block_size(), 32768); // 32 KiB
    }

    #[test]
    fn test_header_crc_flags() {
        let header = Header::with_flags(ALGO_STORE, 15, 1000, FLAG_USE_CRC64).unwrap();
        assert!(header.uses_crc64());
        assert_eq!(header.block_checksum_size(), 8);

        let header = Header::new(ALGO_STORE, 15, 1000).unwrap();
        assert!(!header.uses_crc64());
        assert_eq!(header.block_checksum_size(), 4);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut bytes = Header::default().to_bytes();
        bytes[0] = 0xFF;
        bytes[1] = 0xFF;
        assert!(Header::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_header_unsupported_version() {
        let mut bytes = Header::default().to_bytes();
        bytes[2] = 99; // Version mayor muy alta
        assert!(Header::from_bytes(&bytes).is_err());
    }
}
