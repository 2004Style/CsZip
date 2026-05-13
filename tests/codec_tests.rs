//! Tests del módulo codec
//!
//! Prueba compresores, descompresores y algoritmos

use std::io::Cursor;
use cszip::codec::{Algorithm, CompressionLevel, Compressor, Decompressor};

// ============================================================================
// Tests de Algorithm
// ============================================================================

mod algorithm_tests {
    use super::*;
    
    #[test]
    fn test_algorithm_from_id() {
        assert!(Algorithm::from_id(0).is_ok());
        assert_eq!(Algorithm::from_id(0).unwrap(), Algorithm::Store);
        
        // Algoritmos no implementados pero válidos
        assert!(Algorithm::from_id(1).is_ok());
        assert!(Algorithm::from_id(2).is_ok());
        assert!(Algorithm::from_id(3).is_ok());
        assert!(Algorithm::from_id(4).is_ok());
        
        // Algoritmos inválidos
        assert!(Algorithm::from_id(5).is_err());
        assert!(Algorithm::from_id(255).is_err());
    }
    
    #[test]
    fn test_algorithm_id() {
        assert_eq!(Algorithm::Store.id(), 0);
        assert_eq!(Algorithm::Lz77Huffman.id(), 1);
        assert_eq!(Algorithm::Lz4.id(), 2);
        assert_eq!(Algorithm::Lzma.id(), 3);
        assert_eq!(Algorithm::Deflate.id(), 4);
    }
    
    #[test]
    fn test_algorithm_name() {
        assert!(!Algorithm::Store.name().is_empty());
        assert!(!Algorithm::Lz77Huffman.name().is_empty());
        assert!(!Algorithm::Lz4.name().is_empty());
        assert!(!Algorithm::Lzma.name().is_empty());
        assert!(!Algorithm::Deflate.name().is_empty());
    }
    
    #[test]
    fn test_algorithm_is_implemented() {
        assert!(Algorithm::Store.is_implemented());
        // Los demás no están implementados aún
        assert!(!Algorithm::Lz77Huffman.is_implemented());
        assert!(!Algorithm::Lz4.is_implemented());
        assert!(!Algorithm::Lzma.is_implemented());
        assert!(!Algorithm::Deflate.is_implemented());
    }
    
    #[test]
    fn test_algorithm_try_from() {
        let algo: Result<Algorithm, _> = 0u8.try_into();
        assert!(algo.is_ok());
        assert_eq!(algo.unwrap(), Algorithm::Store);
        
        let invalid: Result<Algorithm, _> = 100u8.try_into();
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_algorithm_into_u8() {
        let id: u8 = Algorithm::Store.into();
        assert_eq!(id, 0);
        
        let id: u8 = Algorithm::Deflate.into();
        assert_eq!(id, 4);
    }
}

// ============================================================================
// Tests de CompressionLevel
// ============================================================================

mod compression_level_tests {
    use super::*;
    
    #[test]
    fn test_compression_level_new() {
        for level in 0..=9 {
            let result = CompressionLevel::new(level);
            assert!(result.is_ok(), "Level {} should be valid", level);
        }
    }
    
    #[test]
    fn test_compression_level_invalid() {
        assert!(CompressionLevel::new(10).is_err());
        assert!(CompressionLevel::new(255).is_err());
    }
    
    #[test]
    fn test_compression_level_value() {
        let level = CompressionLevel::new(5).unwrap();
        assert_eq!(level.value(), 5);
        
        let level = CompressionLevel::new(9).unwrap();
        assert_eq!(level.value(), 9);
    }
    
    #[test]
    fn test_compression_level_constants() {
        assert_eq!(CompressionLevel::MIN.value(), 0);
        assert_eq!(CompressionLevel::MAX.value(), 9);
        assert_eq!(CompressionLevel::DEFAULT.value(), 6);
    }
    
    #[test]
    fn test_compression_level_default() {
        let default = CompressionLevel::default();
        assert_eq!(default.value(), 6);
    }
    
    #[test]
    fn test_compression_level_try_from() {
        let level: Result<CompressionLevel, _> = 5u8.try_into();
        assert!(level.is_ok());
        
        let invalid: Result<CompressionLevel, _> = 15u8.try_into();
        assert!(invalid.is_err());
    }
}

// ============================================================================
// Tests de Compressor
// ============================================================================

mod compressor_tests {
    use super::*;
    
    #[test]
    fn test_compressor_new() {
        let compressor = Compressor::new(Algorithm::Store, CompressionLevel::DEFAULT);
        assert_eq!(compressor.algorithm(), Algorithm::Store);
        assert_eq!(compressor.level().value(), 6);
    }
    
    #[test]
    fn test_compressor_store() {
        let compressor = Compressor::store();
        assert_eq!(compressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_compressor_with_crc64() {
        let compressor = Compressor::store().with_crc64(true);
        // Verificar que no falla al crear
        assert_eq!(compressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_compressor_default() {
        let compressor = Compressor::default();
        assert_eq!(compressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_compress_store_empty() {
        let compressor = Compressor::store();
        let result = compressor.compress_block(&[]);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.original_size, 0);
        assert_eq!(result.compressed_size, 0);
    }
    
    #[test]
    fn test_compress_store_data() {
        let compressor = Compressor::store();
        let data = b"Hello, CsZip! This is a test.";
        
        let result = compressor.compress_block(data).unwrap();
        
        // STORE no comprime, tamaño debería ser igual
        assert_eq!(result.original_size, data.len() as u64);
        assert_eq!(result.compressed_size, data.len() as u64);
        assert!(result.crc32 != 0, "CRC should be computed");
    }
    
    #[test]
    fn test_compress_store_binary() {
        let compressor = Compressor::store();
        let data: Vec<u8> = (0..=255).collect();
        
        let result = compressor.compress_block(&data).unwrap();
        
        assert_eq!(result.original_size, 256);
        assert_eq!(result.compressed_size, 256);
    }
    
    #[test]
    fn test_compress_stream() {
        let compressor = Compressor::store();
        let data = b"Stream compression test data";
        let mut reader = Cursor::new(data.as_slice());
        let mut writer = Vec::new();
        
        let result = compressor.compress(&mut reader, &mut writer).unwrap();
        
        assert_eq!(result.original_size, data.len() as u64);
        assert_eq!(writer.len(), data.len());
    }
    
    #[test]
    fn test_compress_large_data() {
        let compressor = Compressor::store();
        // 1 MB de datos
        let data: Vec<u8> = (0..1024*1024).map(|i| (i % 256) as u8).collect();
        
        let result = compressor.compress_block(&data).unwrap();
        
        assert_eq!(result.original_size, 1024*1024);
        assert!(result.crc32 != 0);
    }
    
    #[test]
    fn test_compress_ratio() {
        let compressor = Compressor::store();
        let data = b"test";
        
        let result = compressor.compress_block(data).unwrap();
        
        // STORE ratio es 1.0
        assert_eq!(result.ratio(), 1.0);
    }
    
    #[test]
    fn test_compress_unsupported_algorithm() {
        // Los algoritmos no implementados deberían fallar
        let compressor = Compressor::new(Algorithm::Lz77Huffman, CompressionLevel::DEFAULT);
        let data = b"test";
        
        let result = compressor.compress_block(data);
        assert!(result.is_err());
    }
}

// ============================================================================
// Tests de Decompressor
// ============================================================================

mod decompressor_tests {
    use super::*;
    
    #[test]
    fn test_decompressor_new() {
        let decompressor = Decompressor::new(Algorithm::Store);
        assert_eq!(decompressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_decompressor_store() {
        let decompressor = Decompressor::store();
        assert_eq!(decompressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_decompressor_with_verification() {
        let decompressor = Decompressor::store().with_checksum_verification(true);
        assert_eq!(decompressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_decompressor_default() {
        let decompressor = Decompressor::default();
        assert_eq!(decompressor.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_decompress_store_empty() {
        let decompressor = Decompressor::store();
        let result = decompressor.decompress_block(&[]);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.decompressed_size, 0);
    }
    
    #[test]
    fn test_decompress_store_data() {
        let decompressor = Decompressor::store();
        let data = b"Hello, CsZip!";
        
        let result = decompressor.decompress_block(data).unwrap();
        
        assert_eq!(result.decompressed_size, data.len() as u64);
        assert_eq!(result.decompressed_size, data.len() as u64);
    }
    
    #[test]
    fn test_decompress_stream() {
        let decompressor = Decompressor::store();
        let data = b"Stream decompression test";
        let mut reader = Cursor::new(data.as_slice());
        let mut writer = Vec::new();
        
        let result = decompressor.decompress(&mut reader, &mut writer, None).unwrap();
        
        assert_eq!(writer, data);
        assert_eq!(result.decompressed_size, data.len() as u64);
    }
    
    #[test]
    fn test_decompress_with_expected_size() {
        let decompressor = Decompressor::store();
        let data = b"Fixed size test";
        let mut reader = Cursor::new(data.as_slice());
        let mut writer = Vec::new();
        
        let result = decompressor.decompress(
            &mut reader, 
            &mut writer, 
            Some(data.len() as u64)
        ).unwrap();
        
        assert_eq!(result.decompressed_size, data.len() as u64);
    }
    
    #[test]
    fn test_decompress_unsupported_algorithm() {
        let decompressor = Decompressor::new(Algorithm::Deflate);
        let data = b"test";
        
        let result = decompressor.decompress_block(data);
        assert!(result.is_err());
    }
}

// ============================================================================
// Tests de roundtrip compresión/descompresión
// ============================================================================

mod roundtrip_tests {
    use super::*;
    
    #[test]
    fn test_roundtrip_store_small() {
        let original = b"Small test data";
        roundtrip_verify(original);
    }
    
    #[test]
    fn test_roundtrip_store_empty() {
        let original = b"";
        roundtrip_verify(original);
    }
    
    #[test]
    fn test_roundtrip_store_binary() {
        let original: Vec<u8> = (0..=255).collect();
        roundtrip_verify(&original);
    }
    
    #[test]
    fn test_roundtrip_store_repetitive() {
        let original: Vec<u8> = vec![b'A'; 10000];
        roundtrip_verify(&original);
    }
    
    #[test]
    fn test_roundtrip_store_large() {
        let original: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        roundtrip_verify(&original);
    }
    
    fn roundtrip_verify(original: &[u8]) {
        let compressor = Compressor::store();
        let decompressor = Decompressor::store();
        
        // Comprimir
        let mut compressed = Vec::new();
        let mut reader = Cursor::new(original);
        compressor.compress(&mut reader, &mut compressed).unwrap();
        
        // Descomprimir
        let mut decompressed = Vec::new();
        let mut reader = Cursor::new(&compressed);
        decompressor.decompress(&mut reader, &mut decompressed, None).unwrap();
        
        assert_eq!(decompressed, original, "Roundtrip should preserve data");
    }
    
    #[test]
    fn test_roundtrip_block_api() {
        let compressor = Compressor::store();
        let decompressor = Decompressor::store();
        
        let original = b"Block API roundtrip test with some longer content to verify";
        
        let _compress_result = compressor.compress_block(original).unwrap();
        
        // Para STORE, los datos comprimidos son los mismos que el original
        let decompress_result = decompressor.decompress_block(original).unwrap();
        
        assert_eq!(decompress_result.decompressed_size, original.len() as u64);
    }
    
    #[test]
    fn test_roundtrip_with_crc_verification() {
        let compressor = Compressor::store();
        let decompressor = Decompressor::store().with_checksum_verification(true);
        
        let original = b"Data with CRC verification enabled";
        
        let compress_result = compressor.compress_block(original).unwrap();
        let crc = compress_result.crc32;
        
        // Verificar CRC en descompresión
        let mut reader = Cursor::new(original);
        let mut writer = Vec::new();
        
        let result = decompressor.decompress_and_verify_crc32(
            &mut reader,
            &mut writer,
            crc,
            Some(original.len() as u64)
        );
        
        assert!(result.is_ok());
        assert_eq!(writer, original);
    }
    
    #[test]
    fn test_crc_mismatch_detection() {
        let decompressor = Decompressor::store().with_checksum_verification(true);
        
        let data = b"Test data for CRC mismatch";
        let mut reader = Cursor::new(data.as_slice());
        let mut writer = Vec::new();
        
        // CRC incorrecto
        let wrong_crc = 0x12345678u32;
        
        let result = decompressor.decompress_and_verify_crc32(
            &mut reader,
            &mut writer,
            wrong_crc,
            None
        );
        
        assert!(result.is_err(), "Should fail with wrong CRC");
    }
}
