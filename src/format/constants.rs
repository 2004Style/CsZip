//! Constantes del formato CsZip
//!
//! Define todos los valores fijos utilizados en el formato binario.

// ============================================================================
// MAGIC NUMBERS
// ============================================================================

/// Magic number principal: "CZ" en ASCII (C=0x43, Z=0x5A)
pub const MAGIC_PRIMARY: u16 = 0x435A;

/// Magic number alternativo: "ZC" (para detectar endianness)
pub const MAGIC_ALT: u16 = 0x5A43;

// ============================================================================
// VERSIÓN DEL FORMATO
// ============================================================================

/// Versión mayor actual del formato
pub const VERSION_MAJOR: u8 = 1;

/// Versión menor actual del formato
pub const VERSION_MINOR: u8 = 0;

// ============================================================================
// LÍMITES DE TAMAÑO DE BLOQUE
// ============================================================================

/// Mínimo log₂ del tamaño de bloque (512 bytes)
pub const MIN_BLOCK_SIZE_LOG2: u16 = 9;

/// Máximo log₂ del tamaño de bloque (64 KiB, límite por u16 en block header)
pub const MAX_BLOCK_SIZE_LOG2: u16 = 16;

/// Valor por defecto log₂ del tamaño de bloque (32 KiB)
pub const DEFAULT_BLOCK_SIZE_LOG2: u16 = 15;

/// Alias para compatibilidad
pub const DEFAULT_BLOCK_SIZE_EXP: u8 = DEFAULT_BLOCK_SIZE_LOG2 as u8;

/// Tamaño de bloque por defecto en bytes (32 KiB)
pub const DEFAULT_BLOCK_SIZE: usize = 1 << DEFAULT_BLOCK_SIZE_LOG2;

/// Tamaño de buffer por defecto para I/O
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

// ============================================================================
// LÍMITES DE EXPANSIÓN
// ============================================================================

/// Mínimo porcentaje de expansión permitido (100% = 1x)
pub const MIN_EXPANSION: u16 = 100;

/// Máximo porcentaje de expansión permitido (5000% = 50x)
pub const MAX_EXPANSION: u16 = 5000;

/// Valor por defecto de expansión (1000% = 10x)
pub const DEFAULT_EXPANSION: u16 = 1000;

// ============================================================================
// ALGORITMOS DE COMPRESIÓN
// ============================================================================

/// Almacenamiento sin compresión
pub const ALGO_STORE: u8 = 0;

/// Alias para compatibilidad
pub const ALGORITHM_STORE: u8 = ALGO_STORE;

/// LZ77 + Huffman (referencia)
pub const ALGO_LZ77_HUFFMAN: u8 = 1;

/// Alias para compatibilidad
pub const ALGORITHM_LZ77_HUFFMAN: u8 = ALGO_LZ77_HUFFMAN;

/// Estilo LZ4 (rápido)
pub const ALGO_LZ4_STYLE: u8 = 2;

/// Alias para compatibilidad
pub const ALGORITHM_LZ4: u8 = ALGO_LZ4_STYLE;

/// Estilo LZMA (fuerte)
pub const ALGO_LZMA_STYLE: u8 = 3;

/// Alias para compatibilidad
pub const ALGORITHM_LZMA: u8 = ALGO_LZMA_STYLE;

/// Compatible DEFLATE (RFC 1951)
pub const ALGO_DEFLATE: u8 = 4;

/// Alias para compatibilidad
pub const ALGORITHM_DEFLATE: u8 = ALGO_DEFLATE;

/// Algoritmo experimental (no soportado)
pub const ALGO_EXPERIMENTAL: u8 = 15;

// ============================================================================
// TIPOS DE BLOQUE
// ============================================================================

/// Bloque de datos comprimidos
pub const BLOCK_TYPE_DATA: u8 = 0;

/// Bloque de metadata
pub const BLOCK_TYPE_METADATA: u8 = 1;

/// Bloque incompleto (error)
pub const BLOCK_TYPE_INCOMPLETE: u8 = 2;

/// Reservado para uso futuro
pub const BLOCK_TYPE_RESERVED: u8 = 3;

// ============================================================================
// TAMAÑOS DE ESTRUCTURAS
// ============================================================================

/// Tamaño del header global en bytes
pub const FILE_HEADER_SIZE: usize = 16;

/// Alias para compatibilidad
pub const HEADER_SIZE: usize = FILE_HEADER_SIZE;

/// Tamaño del header de bloque en bytes
pub const BLOCK_HEADER_SIZE: usize = 12;

/// Tamaño del footer de archivo en bytes
pub const FILE_FOOTER_SIZE: usize = 12;

/// Alias para compatibilidad
pub const FOOTER_SIZE: usize = FILE_FOOTER_SIZE;

/// Tamaño de CRC-32 en bytes
pub const CRC32_SIZE: usize = 4;

/// Tamaño de CRC-64 en bytes
pub const CRC64_SIZE: usize = 8;

// ============================================================================
// MARCADORES
// ============================================================================

/// Marcador de inicio del footer (0xFE)
pub const FILE_FOOTER_MARKER: u8 = 0xFE;

// ============================================================================
// FLAGS DEL HEADER
// ============================================================================

/// Flag: tiene metadata extra después del footer
pub const FLAG_HAS_EXTRA_METADATA: u8 = 0x01;

/// Flag: usar CRC-64 en lugar de CRC-32
pub const FLAG_USE_CRC64: u8 = 0x02;

/// Máscara de flags reservados (bits 4-7 deben ser 0)
pub const FLAG_RESERVED_MASK: u8 = 0xF0;

// ============================================================================
// NIVELES DE COMPRESIÓN
// ============================================================================

/// Nivel mínimo de compresión
pub const MIN_COMPRESSION_LEVEL: u8 = 0;

/// Nivel máximo de compresión
pub const MAX_COMPRESSION_LEVEL: u8 = 9;

/// Nivel por defecto de compresión
pub const DEFAULT_COMPRESSION_LEVEL: u8 = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_numbers() {
        // Verificar que los bytes corresponden a "CZ"
        let bytes = MAGIC_PRIMARY.to_be_bytes();
        assert_eq!(bytes[0], b'C');
        assert_eq!(bytes[1], b'Z');
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_block_size_range() {
        assert!(MIN_BLOCK_SIZE_LOG2 < MAX_BLOCK_SIZE_LOG2);
        assert!(DEFAULT_BLOCK_SIZE_LOG2 >= MIN_BLOCK_SIZE_LOG2);
        assert!(DEFAULT_BLOCK_SIZE_LOG2 <= MAX_BLOCK_SIZE_LOG2);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_expansion_range() {
        assert!(MIN_EXPANSION < MAX_EXPANSION);
        assert!(DEFAULT_EXPANSION >= MIN_EXPANSION);
        assert!(DEFAULT_EXPANSION <= MAX_EXPANSION);
    }

    #[test]
    fn test_header_sizes() {
        assert_eq!(FILE_HEADER_SIZE, 16);
        assert_eq!(BLOCK_HEADER_SIZE, 12);
        assert_eq!(FILE_FOOTER_SIZE, 12);
    }
}
