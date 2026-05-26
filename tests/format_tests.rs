//! Tests del módulo format
//!
//! Prueba headers, block headers, footers, checksums y constantes

use cszip::format::checksum::{Adler32, Crc32, Crc64};
use cszip::format::constants::*;
use cszip::format::{BlockHeader, FileFooter, Header};

// Constantes adicionales usadas en tests (definidas localmente si no exportadas)
const BLOCK_TYPE_DATA: u8 = 0;

// ============================================================================
// Tests de constantes
// ============================================================================

mod constants_tests {
    use super::*;

    #[test]
    fn test_magic_numbers() {
        assert_eq!(MAGIC_PRIMARY, 0x435A, "CZ in ASCII");
        assert_eq!(MAGIC_ALT, 0x5A43, "ZC in ASCII (reversed)");
    }

    #[test]
    fn test_version() {
        assert_eq!(VERSION_MAJOR, 1);
        assert_eq!(VERSION_MINOR, 0);
    }

    #[test]
    fn test_block_size_limits() {
        assert!(MIN_BLOCK_SIZE_LOG2 < MAX_BLOCK_SIZE_LOG2);
        assert_eq!(MIN_BLOCK_SIZE_LOG2, 9); // 512 bytes
        assert_eq!(MAX_BLOCK_SIZE_LOG2, 16); // 64 KiB (límite por u16)
        assert!(DEFAULT_BLOCK_SIZE_LOG2 >= MIN_BLOCK_SIZE_LOG2);
        assert!(DEFAULT_BLOCK_SIZE_LOG2 <= MAX_BLOCK_SIZE_LOG2);
    }

    #[test]
    fn test_expansion_limits() {
        assert!(MIN_EXPANSION < MAX_EXPANSION);
        assert_eq!(MIN_EXPANSION, 100); // 1x
        assert_eq!(MAX_EXPANSION, 5000); // 50x
        assert!(DEFAULT_EXPANSION >= MIN_EXPANSION);
        assert!(DEFAULT_EXPANSION <= MAX_EXPANSION);
    }

    #[test]
    fn test_algorithm_constants() {
        assert_eq!(ALGO_STORE, 0);
        assert_eq!(ALGORITHM_STORE, ALGO_STORE); // Alias
    }

    #[test]
    fn test_default_block_size() {
        let expected = 1 << DEFAULT_BLOCK_SIZE_LOG2;
        assert_eq!(DEFAULT_BLOCK_SIZE, expected);
        assert_eq!(DEFAULT_BLOCK_SIZE, 32 * 1024); // 32 KiB
    }
}

// ============================================================================
// Tests de Header
// ============================================================================

mod header_tests {
    use super::*;

    #[test]
    fn test_header_new_default() {
        let header = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, DEFAULT_EXPANSION)
            .expect("Default header should be valid");

        assert_eq!(header.magic, MAGIC_PRIMARY);
        assert_eq!(header.version_major, VERSION_MAJOR);
        assert_eq!(header.version_minor, VERSION_MINOR);
        assert_eq!(header.compression_algo, ALGO_STORE);
        assert_eq!(header.block_size_log2, DEFAULT_BLOCK_SIZE_LOG2);
        assert_eq!(header.max_expansion, DEFAULT_EXPANSION);
    }

    #[test]
    fn test_header_size() {
        assert_eq!(Header::SIZE, 16, "Header should be 16 bytes");
    }

    #[test]
    fn test_header_serialization() {
        let header = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, DEFAULT_EXPANSION)
            .expect("Header creation failed");

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), Header::SIZE);

        // Verificar magic number en los primeros bytes
        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);
        assert_eq!(magic, MAGIC_PRIMARY);
    }

    #[test]
    fn test_header_deserialization() {
        let original = Header::new(ALGO_STORE, 15, 500).expect("Header creation failed");
        let bytes = original.to_bytes();

        let parsed = Header::from_bytes(&bytes).expect("Header parsing failed");

        assert_eq!(parsed.magic, original.magic);
        assert_eq!(parsed.version_major, original.version_major);
        assert_eq!(parsed.version_minor, original.version_minor);
        assert_eq!(parsed.compression_algo, original.compression_algo);
        assert_eq!(parsed.block_size_log2, original.block_size_log2);
        assert_eq!(parsed.max_expansion, original.max_expansion);
    }

    #[test]
    fn test_header_invalid_block_size() {
        // Block size demasiado pequeño
        let result = Header::new(ALGO_STORE, 5, DEFAULT_EXPANSION);
        assert!(result.is_err());

        // Block size demasiado grande
        let result = Header::new(ALGO_STORE, 35, DEFAULT_EXPANSION);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_invalid_expansion() {
        // Expansion demasiado pequeño
        let result = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, 50);
        assert!(result.is_err());

        // Expansion demasiado grande
        let result = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, 6000);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_checksum_initial() {
        let header = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, DEFAULT_EXPANSION)
            .expect("Header creation failed");

        // El checksum inicial es 0 (se calcula al escribir)
        assert_eq!(header.checksum, 0, "Initial checksum should be 0");
    }

    #[test]
    fn test_header_block_size() {
        let header = Header::new(ALGO_STORE, DEFAULT_BLOCK_SIZE_LOG2, DEFAULT_EXPANSION)
            .expect("Header creation failed");

        assert_eq!(header.block_size(), DEFAULT_BLOCK_SIZE);
    }

    #[test]
    fn test_header_from_invalid_bytes() {
        // Magic inválido
        let mut invalid_magic = [0u8; 16];
        invalid_magic[0] = 0xFF;
        invalid_magic[1] = 0xFF;
        assert!(Header::from_bytes(&invalid_magic).is_err());
    }
}

// ============================================================================
// Tests de BlockHeader
// ============================================================================

mod block_header_tests {
    use super::*;

    #[test]
    fn test_block_header_new() {
        // BlockHeader::new(original_size, compressed_size, adler32, compression_level)
        let block = BlockHeader::new(
            1000,       // original_size
            950,        // compressed_size
            0x12345678, // adler32
            5,          // compression_level
        )
        .expect("BlockHeader creation failed");

        assert_eq!(block.block_type, BLOCK_TYPE_DATA);
        assert_eq!(block.compression_level, 5);
        assert_eq!(block.original_size, 1000);
        assert_eq!(block.compressed_size, 950);
        assert_eq!(block.adler32, 0x12345678);
    }

    #[test]
    fn test_block_header_size() {
        assert_eq!(BlockHeader::SIZE, 12, "BlockHeader should be 12 bytes");
    }

    #[test]
    fn test_block_header_serialization() {
        let block = BlockHeader::new(500, 400, 0xAABBCCDD, 6).expect("BlockHeader creation failed");

        let bytes = block.to_bytes();
        assert_eq!(bytes.len(), BlockHeader::SIZE);
    }

    #[test]
    fn test_block_header_roundtrip() {
        let original =
            BlockHeader::new(65535, 60000, 0x11223344, 9).expect("BlockHeader creation failed");

        let bytes = original.to_bytes();
        let parsed = BlockHeader::from_bytes(&bytes).expect("BlockHeader parsing failed");

        assert_eq!(parsed.block_type, original.block_type);
        assert_eq!(parsed.compression_level, original.compression_level);
        assert_eq!(parsed.original_size, original.original_size);
        assert_eq!(parsed.compressed_size, original.compressed_size);
        assert_eq!(parsed.adler32, original.adler32);
    }

    #[test]
    fn test_block_header_invalid_level() {
        // Nivel de compresión > 9 es inválido
        let result = BlockHeader::new(100, 100, 0, 15);
        assert!(result.is_err());
    }
}

// ============================================================================
// Tests de FileFooter
// ============================================================================

mod file_footer_tests {
    use super::*;

    #[test]
    fn test_file_footer_new() {
        let footer = FileFooter::new(10, 50000).expect("Footer creation failed");

        assert_eq!(footer.marker, FILE_FOOTER_MARKER);
        assert_eq!(footer.num_blocks, 10);
        assert_eq!(footer.total_raw_size, 50000);
    }

    #[test]
    fn test_file_footer_size() {
        assert_eq!(FileFooter::SIZE, 12, "FileFooter should be 12 bytes");
    }

    #[test]
    fn test_file_footer_serialization() {
        let footer = FileFooter::new(5, 10000).expect("Footer creation failed");

        let bytes = footer.to_bytes();
        assert_eq!(bytes.len(), FileFooter::SIZE);

        // Primer byte es el marker
        assert_eq!(bytes[0], FILE_FOOTER_MARKER);
    }

    #[test]
    fn test_file_footer_roundtrip() {
        let original = FileFooter::new(100, 1000000).expect("Footer creation failed");

        let bytes = original.to_bytes();
        let parsed = FileFooter::from_bytes(&bytes).expect("Footer parsing failed");

        assert_eq!(parsed.marker, original.marker);
        assert_eq!(parsed.num_blocks, original.num_blocks);
        assert_eq!(parsed.total_raw_size, original.total_raw_size);
        assert_eq!(parsed.checksum, original.checksum);
    }

    #[test]
    fn test_file_footer_zero_blocks() {
        let footer = FileFooter::new(0, 0).expect("Empty footer should be valid");
        assert_eq!(footer.num_blocks, 0);
        assert_eq!(footer.total_raw_size, 0);
    }
}

// ============================================================================
// Tests de Checksums
// ============================================================================

mod checksum_tests {
    use super::*;

    // CRC-32 Tests
    mod crc32_tests {
        use super::*;

        #[test]
        fn test_crc32_empty() {
            let crc = Crc32::compute(&[]);
            assert_eq!(crc, 0, "CRC of empty data should be 0");
        }

        #[test]
        fn test_crc32_known_value() {
            // "123456789" tiene CRC-32 conocido: 0xCBF43926
            let data = b"123456789";
            let crc = Crc32::compute(data);
            assert_eq!(
                crc, 0xCBF43926,
                "CRC-32 of '123456789' should match known value"
            );
        }

        #[test]
        fn test_crc32_incremental() {
            let data = b"Hello, World!";

            // Calcular de una vez
            let crc_once = Crc32::compute(data);

            // Calcular incrementalmente
            let mut crc = Crc32::new();
            crc.update(&data[..5]);
            crc.update(&data[5..]);
            let crc_incremental = crc.finalize();

            assert_eq!(
                crc_once, crc_incremental,
                "Incremental CRC should match single-pass"
            );
        }

        #[test]
        fn test_crc32_verify() {
            let data = b"Test data for CRC verification";
            let expected = Crc32::compute(data);

            assert!(Crc32::verify(data, expected));
            assert!(!Crc32::verify(data, expected ^ 1));
        }

        #[test]
        fn test_crc32_different_data() {
            let crc1 = Crc32::compute(b"data1");
            let crc2 = Crc32::compute(b"data2");

            assert_ne!(crc1, crc2, "Different data should have different CRCs");
        }

        #[test]
        fn test_crc32_default() {
            let crc = Crc32::default();
            assert_eq!(crc.finalize(), 0);
        }
    }

    // CRC-64 Tests
    mod crc64_tests {
        use super::*;

        #[test]
        fn test_crc64_empty() {
            let crc = Crc64::compute(&[]);
            assert_eq!(crc, 0);
        }

        #[test]
        fn test_crc64_incremental() {
            let data = b"Testing CRC-64 incremental computation";

            let crc_once = Crc64::compute(data);

            let mut crc = Crc64::new();
            crc.update(&data[..10]);
            crc.update(&data[10..]);
            let crc_incremental = crc.finalize();

            assert_eq!(crc_once, crc_incremental);
        }

        #[test]
        fn test_crc64_verify() {
            let data = b"CRC-64 verification test";
            let expected = Crc64::compute(data);

            assert!(Crc64::verify(data, expected));
            assert!(!Crc64::verify(data, expected ^ 1));
        }

        #[test]
        fn test_crc64_different_data() {
            let crc1 = Crc64::compute(b"abc");
            let crc2 = Crc64::compute(b"abd");

            assert_ne!(crc1, crc2);
        }
    }

    // ADLER-32 Tests
    mod adler32_tests {
        use super::*;

        #[test]
        fn test_adler32_empty() {
            let adler = Adler32::compute(&[]);
            assert_eq!(adler, 1, "ADLER-32 of empty data should be 1");
        }

        #[test]
        fn test_adler32_known_value() {
            // "Wikipedia" tiene ADLER-32 conocido: 0x11E60398
            let data = b"Wikipedia";
            let adler = Adler32::compute(data);
            assert_eq!(adler, 0x11E60398, "ADLER-32 of 'Wikipedia' should match");
        }

        #[test]
        fn test_adler32_incremental() {
            let data = b"Incremental ADLER test";

            let adler_once = Adler32::compute(data);

            let mut adler = Adler32::new();
            adler.update(&data[..10]);
            adler.update(&data[10..]);
            let adler_incremental = adler.finalize();

            assert_eq!(adler_once, adler_incremental);
        }

        #[test]
        fn test_adler32_verify() {
            let data = b"ADLER verify";
            let expected = Adler32::compute(data);

            assert!(Adler32::verify(data, expected));
            assert!(!Adler32::verify(data, expected ^ 1));
        }

        #[test]
        fn test_adler32_single_byte() {
            let adler = Adler32::compute(&[0x41]); // 'A'
                                                   // A=1, s1=66, s2=66, result = (66 << 16) | 66 = 0x00420042
            assert_eq!(adler, 0x00420042);
        }
    }
}
