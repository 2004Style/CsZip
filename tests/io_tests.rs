//! Tests del módulo io
//!
//! Prueba CzWriter, CzReader y operaciones de archivo

use std::io::{Cursor, Seek, SeekFrom};
use cszip::io::{CzWriter, CzReader};
use cszip::codec::{Algorithm, CompressionLevel};
use cszip::format::constants::*;
use tempfile::TempDir;

// ============================================================================
// Tests de CzWriter
// ============================================================================

mod writer_tests {
    use super::*;
    
    #[test]
    fn test_writer_new() {
        let buffer = Cursor::new(Vec::new());
        let writer = CzWriter::new(buffer);
        
        assert!(writer.is_ok());
    }
    
    #[test]
    fn test_writer_with_options() {
        let buffer = Cursor::new(Vec::new());
        let writer = CzWriter::new_with_options(
            buffer,
            Algorithm::Store,
            CompressionLevel::DEFAULT
        );
        
        assert!(writer.is_ok());
        let writer = writer.unwrap();
        assert_eq!(writer.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_writer_block_size() {
        let buffer = Cursor::new(Vec::new());
        let writer = CzWriter::new(buffer).unwrap();
        
        let block_size = writer.block_size();
        assert!(block_size > 0);
        assert_eq!(block_size, DEFAULT_BLOCK_SIZE);
    }
    
    #[test]
    fn test_writer_algorithm() {
        let buffer = Cursor::new(Vec::new());
        let writer = CzWriter::new(buffer).unwrap();
        
        assert_eq!(writer.algorithm(), Algorithm::Store);
    }
    
    #[test]
    fn test_write_empty_block() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        // Escribir bloque vacío no hace nada (retorna Ok sin escribir)
        let result = writer.write_block(&[]);
        assert!(result.is_ok());
        
        let stats = writer.finish().unwrap();
        // No se escribió ningún bloque porque los datos estaban vacíos
        assert_eq!(stats.block_count, 0);
    }
    
    #[test]
    fn test_write_single_block() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        let data = b"Hello, CsZip!";
        writer.write_block(data).unwrap();
        
        let stats = writer.finish().unwrap();
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, data.len() as u64);
    }
    
    #[test]
    fn test_write_multiple_blocks() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        for i in 0..5 {
            let data = format!("Block number {}", i);
            writer.write_block(data.as_bytes()).unwrap();
        }
        
        let stats = writer.finish().unwrap();
        
        assert_eq!(stats.block_count, 5);
    }
    
    #[test]
    fn test_write_stream() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        let data = b"Stream data for writing test";
        let mut reader = Cursor::new(data.as_slice());
        
        writer.write_stream(&mut reader).unwrap();
        
        let stats = writer.finish().unwrap();
        
        assert!(stats.block_count >= 1);
        assert_eq!(stats.original_size, data.len() as u64);
    }
    
    #[test]
    fn test_write_large_stream() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        // Datos más grandes que un bloque
        let data: Vec<u8> = (0..500_000).map(|i| (i % 256) as u8).collect();
        let mut reader = Cursor::new(&data);
        
        writer.write_stream(&mut reader).unwrap();
        
        let stats = writer.finish().unwrap();
        
        assert!(stats.block_count >= 2, "Should have multiple blocks");
        assert_eq!(stats.original_size, data.len() as u64);
    }
    
    #[test]
    fn test_writer_stats() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CzWriter::new(buffer).unwrap();
        
        let data = b"Stats test data";
        writer.write_block(data).unwrap();
        
        let stats = writer.finish().unwrap();
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, data.len() as u64);
        assert!(stats.compressed_size >= data.len() as u64); // STORE no comprime
        assert!(stats.ratio() >= 0.0);
    }
    
    #[test]
    fn test_writer_finish_produces_valid_format() {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = CzWriter::new(&mut buffer).unwrap();
            writer.write_block(b"Test").unwrap();
            writer.finish().unwrap();
        }
        
        // Verificar estructura del archivo
        buffer.seek(SeekFrom::Start(0)).unwrap();
        
        let data = buffer.into_inner();
        
        // Debe tener al menos header (16) + block header (12) + data + footer (12)
        assert!(data.len() >= 16 + 12 + 4 + 12);
        
        // Verificar magic number
        let magic = u16::from_be_bytes([data[0], data[1]]);
        assert_eq!(magic, MAGIC_PRIMARY);
    }
    
    #[test]
    fn test_create_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.cz");
        
        let writer = CzWriter::create(&path);
        assert!(writer.is_ok());
        
        let mut writer = writer.unwrap();
        writer.write_block(b"File test").unwrap();
        writer.finish().unwrap();
        
        assert!(path.exists());
    }
}

// ============================================================================
// Tests de CzReader
// ============================================================================

mod reader_tests {
    use super::*;
    
    #[allow(dead_code)]
    fn create_test_archive(data: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(data).unwrap();
            writer.finish().unwrap();
        }
        buffer
    }
    
    #[allow(dead_code)]
    fn write_to_buffer(data: &[u8]) -> Cursor<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(&mut cursor).unwrap();
            writer.write_block(data).unwrap();
            writer.finish().unwrap();
        }
        Cursor::new(buffer)
    }
    
    #[test]
    fn test_reader_new() {
        // Crear un archivo válido primero
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(b"test").unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor);
        
        assert!(reader.is_ok());
    }
    
    #[test]
    fn test_reader_header() {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(b"header test").unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor).unwrap();
        
        let header = reader.header();
        assert_eq!(header.magic, MAGIC_PRIMARY);
        assert_eq!(header.version_major, VERSION_MAJOR);
    }
    
    #[test]
    fn test_reader_algorithm() {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(b"algo test").unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor).unwrap();
        
        let algo = reader.algorithm().unwrap();
        assert_eq!(algo, Algorithm::Store);
    }
    
    #[test]
    fn test_reader_block_size() {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(b"block size test").unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor).unwrap();
        
        let block_size = reader.block_size();
        assert_eq!(block_size, DEFAULT_BLOCK_SIZE);
    }
    
    #[test]
    fn test_read_single_block() {
        let original = b"Single block content";
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        
        let block = reader.read_block().unwrap();
        assert!(block.is_some());
        
        let block = block.unwrap();
        assert_eq!(block.data, original);
    }
    
    #[test]
    fn test_read_multiple_blocks() {
        let blocks_data = vec![
            b"Block 0".to_vec(),
            b"Block 1".to_vec(),
            b"Block 2".to_vec(),
        ];
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            for data in &blocks_data {
                writer.write_block(data).unwrap();
            }
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        
        for (i, expected) in blocks_data.iter().enumerate() {
            let block = reader.read_block().unwrap();
            assert!(block.is_some(), "Block {} should exist", i);
            assert_eq!(&block.unwrap().data, expected);
        }
        
        // No más bloques
        let block = reader.read_block().unwrap();
        assert!(block.is_none());
    }
    
    #[test]
    fn test_read_footer() {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(b"footer test 1").unwrap();
            writer.write_block(b"footer test 2").unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        
        let footer = reader.read_footer().unwrap();
        assert_eq!(footer.num_blocks, 2);
    }
    
    #[test]
    fn test_decompress_all() {
        let original = b"Data to decompress completely with all method";
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        
        let mut output = Vec::new();
        let stats = reader.decompress_all(&mut output).unwrap();
        
        assert_eq!(output, original);
        assert_eq!(stats.block_count, 1);
    }
    
    #[test]
    fn test_with_checksum_verification() {
        let original = b"Checksum verification test";
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor).unwrap()
            .with_checksum_verification(true);
        
        // Debería funcionar sin errores
        let header = reader.header();
        assert_eq!(header.magic, MAGIC_PRIMARY);
    }
    
    #[test]
    fn test_reader_invalid_magic() {
        let mut invalid_data = vec![0xFF, 0xFF]; // Magic inválido
        invalid_data.extend_from_slice(&[0u8; 50]);
        
        let cursor = Cursor::new(invalid_data);
        let reader = CzReader::new(cursor);
        
        assert!(reader.is_err());
    }
    
    #[test]
    fn test_reader_truncated_data() {
        let truncated = vec![0x43, 0x5A]; // Solo magic, sin resto del header
        
        let cursor = Cursor::new(truncated);
        let reader = CzReader::new(cursor);
        
        assert!(reader.is_err());
    }
    
    #[test]
    fn test_open_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("read_test.cz");
        
        // Crear archivo
        {
            let mut writer = CzWriter::create(&path).unwrap();
            writer.write_block(b"File content").unwrap();
            writer.finish().unwrap();
        }
        
        // Leer archivo
        let reader = CzReader::open(&path);
        assert!(reader.is_ok());
        
        let mut reader = reader.unwrap();
        let block = reader.read_block().unwrap();
        assert!(block.is_some());
        assert_eq!(block.unwrap().data, b"File content");
    }
}

// ============================================================================
// Tests de roundtrip completo Writer -> Reader
// ============================================================================

mod roundtrip_tests {
    use super::*;
    
    fn roundtrip_test(original: &[u8]) {
        let mut buffer = Vec::new();
        
        // Escribir usando write_stream (divide automáticamente en bloques)
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            let mut reader = Cursor::new(original);
            writer.write_stream(&mut reader).unwrap();
            writer.finish().unwrap();
        }
        
        // Leer usando decompress_all
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        let mut output = Vec::new();
        reader.decompress_all(&mut output).unwrap();
        
        assert_eq!(output, original, "Data should match after roundtrip");
    }
    
    #[test]
    fn test_roundtrip_empty() {
        roundtrip_test(b"");
    }
    
    #[test]
    fn test_roundtrip_small() {
        roundtrip_test(b"Small data");
    }
    
    #[test]
    fn test_roundtrip_medium() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        roundtrip_test(&data);
    }
    
    #[test]
    fn test_roundtrip_large() {
        let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        roundtrip_test(&data);
    }
    
    #[test]
    fn test_roundtrip_all_bytes() {
        let data: Vec<u8> = (0..=255).collect();
        roundtrip_test(&data);
    }
    
    #[test]
    fn test_roundtrip_repetitive() {
        let data = vec![b'X'; 50_000];
        roundtrip_test(&data);
    }
    
    #[test]
    fn test_roundtrip_stream() {
        let original: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
        
        let mut buffer = Vec::new();
        
        // Escribir usando stream
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            let mut reader = Cursor::new(&original);
            writer.write_stream(&mut reader).unwrap();
            writer.finish().unwrap();
        }
        
        // Leer usando decompress_all
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        let mut output = Vec::new();
        reader.decompress_all(&mut output).unwrap();
        
        assert_eq!(output, original);
    }
    
    #[test]
    fn test_roundtrip_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("roundtrip.cz");
        
        let original = b"File roundtrip test content";
        
        // Escribir
        {
            let mut writer = CzWriter::create(&path).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }
        
        // Leer
        let mut reader = CzReader::open(&path).unwrap();
        let mut output = Vec::new();
        reader.decompress_all(&mut output).unwrap();
        
        assert_eq!(output, original);
    }
    
    #[test]
    fn test_roundtrip_multiple_blocks_stream() {
        // Datos lo suficientemente grandes para múltiples bloques
        let original: Vec<u8> = (0..500_000).map(|i| ((i * 7) % 256) as u8).collect();
        
        let mut buffer = Vec::new();
        
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            let mut reader = Cursor::new(&original);
            writer.write_stream(&mut reader).unwrap();
            let stats = writer.finish().unwrap();
            assert!(stats.block_count >= 2, "Should have multiple blocks");
        }
        
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        let mut output = Vec::new();
        let stats = reader.decompress_all(&mut output).unwrap();
        
        assert!(stats.block_count >= 2);
        assert_eq!(output, original);
    }
}

// ============================================================================
// Tests de BlockData
// ============================================================================

mod block_data_tests {
    use super::*;
    
    #[test]
    fn test_block_data_fields() {
        let original = b"Block data test";
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original).unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        let block = reader.read_block().unwrap().unwrap();
        
        assert_eq!(block.data, original);
        assert_eq!(block.original_size as usize, original.len());
        assert!(block.compressed_size > 0);
        assert!(block.index == 0);
    }
}

// ============================================================================
// Tests de estadísticas
// ============================================================================

mod stats_tests {
    use super::*;
    
    #[test]
    fn test_write_stats() {
        let data = b"Stats test data for writing";
        
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = CzWriter::new(cursor).unwrap();
        writer.write_block(data).unwrap();
        let stats = writer.finish().unwrap();
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, data.len() as u64);
        assert!(stats.compressed_size > 0);
        assert!(stats.ratio() >= 0.0);
    }
    
    #[test]
    fn test_read_stats() {
        let data = b"Stats test data for reading";
        
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(data).unwrap();
            writer.finish().unwrap();
        }
        
        let cursor = Cursor::new(buffer);
        let mut reader = CzReader::new(cursor).unwrap();
        let mut output = Vec::new();
        let stats = reader.decompress_all(&mut output).unwrap();
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, data.len() as u64);
    }
}
