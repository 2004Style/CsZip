//! Estructuras de bloques y footer para archivos CsZip
//!
//! Cada bloque contiene su propio header, datos comprimidos y checksum.

use crate::error::{Error, ErrorKind, Result};
use crate::format::constants::*;
use crate::format::header::Header;
use std::io::{Read, Write};

/// Header de un bloque comprimido (12 bytes)
///
/// ```text
/// Offset  Tamaño  Campo               Tipo
/// ──────  ──────  ──────────────────  ──────────
/// 0       1       Block Type          u8
/// 1       1       Compression Level   u8
/// 2-3     2       Original Size       u16 BE
/// 4-7     4       Compressed Size     u32 BE
/// 8-11    4       ADLER-32            u32 BE
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    /// Tipo de bloque (0=DATA, 1=META, 2=INCOMPLETE)
    pub block_type: u8,
    /// Nivel de compresión usado (0-9)
    pub compression_level: u8,
    /// Tamaño original de los datos (antes de comprimir)
    pub original_size: u16,
    /// Tamaño de los datos comprimidos
    pub compressed_size: u32,
    /// Checksum ADLER-32 de los datos originales
    pub adler32: u32,
}

impl BlockHeader {
    /// Tamaño del header de bloque en bytes
    pub const SIZE: usize = BLOCK_HEADER_SIZE;

    /// Crear un nuevo header de bloque de datos
    ///
    /// # Argumentos
    ///
    /// * `original_size` - Tamaño de los datos originales
    /// * `compressed_size` - Tamaño después de comprimir
    /// * `adler32` - Checksum ADLER-32 de datos originales
    /// * `compression_level` - Nivel de compresión usado
    pub fn new(
        original_size: u16,
        compressed_size: u32,
        adler32: u32,
        compression_level: u8,
    ) -> Result<Self> {
        if original_size == 0 {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                "Tamaño original no puede ser 0",
            ));
        }

        if compression_level > MAX_COMPRESSION_LEVEL {
            return Err(Error::new(
                ErrorKind::InvalidCompressionLevel,
                format!("Nivel de compresión {} inválido", compression_level),
            ));
        }

        Ok(Self {
            block_type: BLOCK_TYPE_DATA,
            compression_level,
            original_size,
            compressed_size,
            adler32,
        })
    }

    /// Crear un header de bloque de metadata
    pub fn new_metadata(
        original_size: u16,
        compressed_size: u32,
        adler32: u32,
    ) -> Result<Self> {
        if original_size == 0 {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                "Tamaño original no puede ser 0",
            ));
        }

        Ok(Self {
            block_type: BLOCK_TYPE_METADATA,
            compression_level: 0,
            original_size,
            compressed_size,
            adler32,
        })
    }

    /// Leer header de bloque desde un stream
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::SIZE];
        reader.read_exact(&mut buf).map_err(|e| {
            Error::new(
                ErrorKind::UnexpectedEof,
                format!("No se pudo leer block header: {}", e),
            )
        })?;

        Self::from_bytes(&buf)
    }

    /// Parsear header de bloque desde bytes
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let block_type = bytes[0];

        // Validar tipo de bloque
        if block_type > BLOCK_TYPE_RESERVED {
            return Err(Error::new(
                ErrorKind::InvalidBlockType,
                format!("Tipo de bloque {} inválido", block_type),
            ));
        }

        if block_type == BLOCK_TYPE_INCOMPLETE {
            return Err(Error::new(
                ErrorKind::IncompleteBlockFound,
                "Bloque incompleto detectado",
            ));
        }

        let compression_level = bytes[1];
        let original_size = u16::from_be_bytes([bytes[2], bytes[3]]);
        let compressed_size = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let adler32 = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Self {
            block_type,
            compression_level,
            original_size,
            compressed_size,
            adler32,
        })
    }

    /// Serializar header de bloque a bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.block_type;
        bytes[1] = self.compression_level;
        bytes[2..4].copy_from_slice(&self.original_size.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.compressed_size.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.adler32.to_be_bytes());
        bytes
    }

    /// Escribir header de bloque a un stream
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_bytes()).map_err(|e| {
            Error::new(ErrorKind::Io, format!("Error escribiendo block header: {}", e))
        })
    }

    /// Validar el header contra el header global del archivo
    pub fn validate(&self, file_header: &Header) -> Result<()> {
        let max_block_size = file_header.block_size();

        // Validar que el tamaño original no exceda el tamaño de bloque
        if self.original_size as usize > max_block_size {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                format!(
                    "Tamaño original {} excede tamaño de bloque {}",
                    self.original_size, max_block_size
                ),
            ));
        }

        // Validar ratio de expansión (protección zip bomb)
        let max_compressed = (self.original_size as u64)
            .saturating_mul(file_header.max_expansion as u64)
            .saturating_div(100);

        if self.compressed_size as u64 > max_compressed {
            return Err(Error::new(
                ErrorKind::CompressionBombSuspected,
                format!(
                    "Tamaño comprimido {} excede límite permitido {}",
                    self.compressed_size, max_compressed
                ),
            ));
        }

        Ok(())
    }

    /// ¿Es un bloque de datos?
    #[inline]
    pub fn is_data(&self) -> bool {
        self.block_type == BLOCK_TYPE_DATA
    }

    /// ¿Es un bloque de metadata?
    #[inline]
    pub fn is_metadata(&self) -> bool {
        self.block_type == BLOCK_TYPE_METADATA
    }
}

/// Footer de un archivo CsZip (12 bytes)
///
/// ```text
/// Offset  Tamaño  Campo               Tipo
/// ──────  ──────  ──────────────────  ──────────
/// 0       1       Marker              u8 (0xFE)
/// 1-3     3       Num Blocks          u24 BE
/// 4-7     4       Total Raw Size      u32 BE
/// 8-11    4       Footer Checksum     u32 BE
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFooter {
    /// Marcador de footer (siempre 0xFE)
    pub marker: u8,
    /// Número total de bloques
    pub num_blocks: u32,
    /// Suma de tamaños originales de todos los bloques
    pub total_raw_size: u32,
    /// Checksum CRC-32 del footer
    pub checksum: u32,
}

impl FileFooter {
    /// Tamaño del footer en bytes
    pub const SIZE: usize = FILE_FOOTER_SIZE;

    /// Marcador de inicio del footer
    pub const MARKER: u8 = FILE_FOOTER_MARKER;

    /// Crear un nuevo footer
    ///
    /// # Argumentos
    ///
    /// * `num_blocks` - Número total de bloques en el archivo
    /// * `total_raw_size` - Suma de tamaños originales
    pub fn new(num_blocks: u32, total_raw_size: u32) -> Result<Self> {
        Self::with_checksum(num_blocks, total_raw_size, 0)
    }

    /// Crear un nuevo footer con checksum
    ///
    /// # Argumentos
    ///
    /// * `num_blocks` - Número total de bloques en el archivo
    /// * `total_raw_size` - Suma de tamaños originales
    /// * `checksum` - CRC-32 global de todos los datos originales
    pub fn with_checksum(num_blocks: u32, total_raw_size: u32, checksum: u32) -> Result<Self> {
        // num_blocks debe caber en 24 bits (máximo ~16 millones)
        if num_blocks > 0x00FFFFFF {
            return Err(Error::new(
                ErrorKind::InvalidBlockSize,
                "Demasiados bloques (máximo 16777215)",
            ));
        }

        Ok(Self {
            marker: Self::MARKER,
            num_blocks,
            total_raw_size,
            checksum,
        })
    }

    /// Leer footer desde un stream
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::SIZE];
        reader.read_exact(&mut buf).map_err(|e| {
            Error::new(
                ErrorKind::UnexpectedEof,
                format!("No se pudo leer footer: {}", e),
            )
        })?;

        Self::from_bytes(&buf)
    }

    /// Parsear footer desde bytes
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let marker = bytes[0];

        if marker != Self::MARKER {
            return Err(Error::new(
                ErrorKind::CorruptedFileFooter,
                format!("Footer marker inválido: esperado 0x{:02X}, encontrado 0x{:02X}", 
                    Self::MARKER, marker),
            ));
        }

        // num_blocks es u24 (3 bytes)
        let num_blocks = ((bytes[1] as u32) << 16) 
                       | ((bytes[2] as u32) << 8) 
                       | (bytes[3] as u32);
        
        let total_raw_size = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let checksum = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Self {
            marker,
            num_blocks,
            total_raw_size,
            checksum,
        })
    }

    /// Serializar footer a bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.marker;
        // num_blocks como u24 (3 bytes, big-endian)
        bytes[1] = ((self.num_blocks >> 16) & 0xFF) as u8;
        bytes[2] = ((self.num_blocks >> 8) & 0xFF) as u8;
        bytes[3] = (self.num_blocks & 0xFF) as u8;
        bytes[4..8].copy_from_slice(&self.total_raw_size.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.checksum.to_be_bytes());
        bytes
    }

    /// Escribir footer a un stream
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_bytes()).map_err(|e| {
            Error::new(ErrorKind::Io, format!("Error escribiendo footer: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_new() {
        let header = BlockHeader::new(1000, 800, 0xABCD1234, 6).unwrap();
        assert_eq!(header.block_type, BLOCK_TYPE_DATA);
        assert_eq!(header.compression_level, 6);
        assert_eq!(header.original_size, 1000);
        assert_eq!(header.compressed_size, 800);
        assert_eq!(header.adler32, 0xABCD1234);
    }

    #[test]
    fn test_block_header_zero_size() {
        assert!(BlockHeader::new(0, 0, 0, 6).is_err());
    }

    #[test]
    fn test_block_header_invalid_level() {
        assert!(BlockHeader::new(100, 80, 0, 10).is_err());
    }

    #[test]
    fn test_block_header_roundtrip() {
        let original = BlockHeader::new(500, 400, 0x12345678, 3).unwrap();
        let bytes = original.to_bytes();
        let parsed = BlockHeader::from_bytes(&bytes).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_block_header_validate() {
        let file_header = Header::new(ALGO_STORE, 16, 1000).unwrap(); // 64 KiB blocks
        
        // Válido: original_size cabe en el bloque
        let block = BlockHeader::new(1000, 800, 0, 6).unwrap();
        assert!(block.validate(&file_header).is_ok());
    }

    #[test]
    fn test_block_header_validate_too_large() {
        let file_header = Header::new(ALGO_STORE, 10, 1000).unwrap(); // 1 KiB blocks
        
        // Inválido: original_size > block_size
        let block = BlockHeader::new(2000, 1500, 0, 6).unwrap();
        assert!(block.validate(&file_header).is_err());
    }

    #[test]
    fn test_block_header_incomplete() {
        let mut bytes = BlockHeader::new(100, 80, 0, 0).unwrap().to_bytes();
        bytes[0] = BLOCK_TYPE_INCOMPLETE;
        assert!(BlockHeader::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_footer_new() {
        let footer = FileFooter::new(10, 50000).unwrap();
        assert_eq!(footer.marker, FILE_FOOTER_MARKER);
        assert_eq!(footer.num_blocks, 10);
        assert_eq!(footer.total_raw_size, 50000);
    }

    #[test]
    fn test_footer_too_many_blocks() {
        assert!(FileFooter::new(0x01000000, 0).is_err()); // > 16777215
    }

    #[test]
    fn test_footer_roundtrip() {
        let original = FileFooter::new(100, 1000000).unwrap();
        let bytes = original.to_bytes();
        let parsed = FileFooter::from_bytes(&bytes).unwrap();
        assert_eq!(original.num_blocks, parsed.num_blocks);
        assert_eq!(original.total_raw_size, parsed.total_raw_size);
    }

    #[test]
    fn test_footer_invalid_marker() {
        let mut bytes = FileFooter::new(1, 100).unwrap().to_bytes();
        bytes[0] = 0x00; // Marker inválido
        assert!(FileFooter::from_bytes(&bytes).is_err());
    }
}
