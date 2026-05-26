//! Tests del módulo error
//!
//! Prueba tipos de error, códigos y manejo de errores

use cszip::error::{Error, ErrorKind};
use std::io;

// ============================================================================
// Tests de ErrorKind
// ============================================================================

mod error_kind_tests {
    use super::*;

    #[test]
    fn test_error_kind_variants() {
        // Verificar que todos los variantes existen
        let _ = ErrorKind::InvalidMagicNumber;
        let _ = ErrorKind::UnsupportedVersion;
        let _ = ErrorKind::UnsupportedAlgorithm;
        let _ = ErrorKind::InvalidBlockSize;
        let _ = ErrorKind::InvalidExpansionLimit;
        let _ = ErrorKind::InvalidBlockType;
        let _ = ErrorKind::BlockCrcMismatch;
        let _ = ErrorKind::CompressionBombSuspected;
        let _ = ErrorKind::IncompleteBlockFound;
        let _ = ErrorKind::CorruptedBlockHeader;
        let _ = ErrorKind::CorruptedFileFooter;
        let _ = ErrorKind::InvalidAdler32Checksum;
        let _ = ErrorKind::MemoryLimitExceeded;
        let _ = ErrorKind::UnexpectedEof;
        let _ = ErrorKind::InvalidCompressionLevel;
        let _ = ErrorKind::Io;
        let _ = ErrorKind::FileNotFound;
        let _ = ErrorKind::FileExists;
        let _ = ErrorKind::IoError;
        let _ = ErrorKind::InvalidData;
        let _ = ErrorKind::ChecksumMismatch;
    }

    #[test]
    fn test_error_kind_clone() {
        let kind = ErrorKind::InvalidMagicNumber;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_error_kind_copy() {
        let kind = ErrorKind::UnsupportedVersion;
        let copied = kind;
        assert_eq!(kind, copied);
    }

    #[test]
    fn test_error_kind_debug() {
        let kind = ErrorKind::InvalidBlockSize;
        let debug_str = format!("{:?}", kind);
        assert!(debug_str.contains("InvalidBlockSize"));
    }

    #[test]
    fn test_error_kind_equality() {
        assert_eq!(ErrorKind::InvalidMagicNumber, ErrorKind::InvalidMagicNumber);
        assert_ne!(ErrorKind::InvalidMagicNumber, ErrorKind::UnsupportedVersion);
    }
}

// ============================================================================
// Tests de Error
// ============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn test_error_new() {
        let error = Error::new(ErrorKind::InvalidMagicNumber, "Bad magic number");

        assert_eq!(error.kind(), ErrorKind::InvalidMagicNumber);
    }

    #[test]
    fn test_error_with_context() {
        let error = Error::new(ErrorKind::FileNotFound, "File not found")
            .with_context("Looking for config.cz");

        assert_eq!(error.kind(), ErrorKind::FileNotFound);
        assert!(error.context().is_some());
        assert!(error.context().unwrap().contains("config.cz"));
    }

    #[test]
    fn test_error_kind_getter() {
        let error = Error::new(ErrorKind::UnsupportedAlgorithm, "Algorithm not implemented");
        assert_eq!(error.kind(), ErrorKind::UnsupportedAlgorithm);
    }

    #[test]
    fn test_error_code() {
        let test_cases = vec![
            (ErrorKind::InvalidMagicNumber, 1),
            (ErrorKind::UnsupportedVersion, 2),
            (ErrorKind::UnsupportedAlgorithm, 3),
            (ErrorKind::InvalidBlockSize, 4),
            (ErrorKind::InvalidExpansionLimit, 5),
            (ErrorKind::InvalidBlockType, 6),
            (ErrorKind::BlockCrcMismatch, 7),
            (ErrorKind::CompressionBombSuspected, 8),
            (ErrorKind::IncompleteBlockFound, 9),
            (ErrorKind::CorruptedBlockHeader, 10),
            (ErrorKind::CorruptedFileFooter, 11),
            (ErrorKind::InvalidAdler32Checksum, 12),
            (ErrorKind::MemoryLimitExceeded, 13),
            (ErrorKind::UnexpectedEof, 14),
            (ErrorKind::InvalidCompressionLevel, 15),
            (ErrorKind::Io, 16),
            (ErrorKind::FileNotFound, 17),
            (ErrorKind::FileExists, 18),
            (ErrorKind::IoError, 19),
            (ErrorKind::InvalidData, 20),
            (ErrorKind::ChecksumMismatch, 21),
        ];

        for (kind, expected_code) in test_cases {
            let error = Error::new(kind, "test");
            assert_eq!(error.code(), expected_code, "Wrong code for {:?}", kind);
        }
    }

    #[test]
    fn test_error_display() {
        let error = Error::new(ErrorKind::InvalidMagicNumber, "Expected 0x435A");
        let display = format!("{}", error);

        assert!(!display.is_empty());
        assert!(display.contains("0x435A"));
    }

    #[test]
    fn test_error_debug() {
        let error = Error::new(ErrorKind::UnsupportedVersion, "Version 99 not supported");
        let debug = format!("{:?}", error);

        assert!(debug.contains("UnsupportedVersion"));
    }

    #[test]
    fn test_error_is_std_error() {
        let error = Error::new(ErrorKind::Io, "IO failure");
        let std_error: &dyn std::error::Error = &error;

        // Debe implementar Display
        let _ = format!("{}", std_error);
    }

    #[test]
    fn test_error_from_io() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File missing");
        let error: Error = io_error.into();

        assert_eq!(error.kind(), ErrorKind::Io);
    }

    #[test]
    fn test_error_context_none() {
        let error = Error::new(ErrorKind::InvalidBlockSize, "Size out of range");
        assert!(error.context().is_none());
    }

    #[test]
    fn test_error_context_string() {
        let error = Error::new(ErrorKind::FileNotFound, "Not found")
            .with_context(String::from("path/to/file.cz"));

        assert_eq!(error.context(), Some("path/to/file.cz"));
    }
}

// ============================================================================
// Tests de Result type alias
// ============================================================================

mod result_tests {
    use super::*;
    use cszip::error::Result;

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(Error::new(ErrorKind::InvalidData, "Invalid"));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_result_map() {
        let result: Result<i32> = Ok(5);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.unwrap(), 10);
    }

    #[test]
    fn test_result_and_then() {
        fn double_if_positive(x: i32) -> Result<i32> {
            if x > 0 {
                Ok(x * 2)
            } else {
                Err(Error::new(ErrorKind::InvalidData, "Must be positive"))
            }
        }

        let result: Result<i32> = Ok(5);
        let chained = result.and_then(double_if_positive);
        assert_eq!(chained.unwrap(), 10);

        let result: Result<i32> = Ok(-5);
        let chained = result.and_then(double_if_positive);
        assert!(chained.is_err());
    }
}

// ============================================================================
// Tests de uso práctico
// ============================================================================

mod practical_tests {
    use super::*;

    fn parse_magic(bytes: &[u8]) -> cszip::error::Result<u16> {
        if bytes.len() < 2 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Need at least 2 bytes",
            ));
        }

        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);

        if magic != 0x435A && magic != 0x5A43 {
            return Err(Error::new(
                ErrorKind::InvalidMagicNumber,
                format!("Expected 0x435A or 0x5A43, got 0x{:04X}", magic),
            ));
        }

        Ok(magic)
    }

    #[test]
    fn test_parse_valid_magic() {
        let result = parse_magic(&[0x43, 0x5A]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x435A);
    }

    #[test]
    fn test_parse_alt_magic() {
        let result = parse_magic(&[0x5A, 0x43]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x5A43);
    }

    #[test]
    fn test_parse_invalid_magic() {
        let result = parse_magic(&[0xFF, 0xFF]);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), ErrorKind::InvalidMagicNumber);
    }

    #[test]
    fn test_parse_too_short() {
        let result = parse_magic(&[0x43]);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), ErrorKind::UnexpectedEof);
    }

    fn validate_compression_level(level: u8) -> cszip::error::Result<u8> {
        if level > 9 {
            return Err(Error::new(
                ErrorKind::InvalidCompressionLevel,
                format!("Level {} exceeds maximum of 9", level),
            ));
        }
        Ok(level)
    }

    #[test]
    fn test_valid_levels() {
        for level in 0..=9 {
            assert!(validate_compression_level(level).is_ok());
        }
    }

    #[test]
    fn test_invalid_levels() {
        for level in 10..=20 {
            assert!(validate_compression_level(level).is_err());
        }
    }

    #[test]
    fn test_error_chain() {
        fn inner_operation() -> cszip::error::Result<()> {
            Err(Error::new(ErrorKind::IoError, "Disk full"))
        }

        fn outer_operation() -> cszip::error::Result<()> {
            inner_operation().map_err(|e| e.with_context("Writing block 5"))
        }

        let result = outer_operation();
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert_eq!(error.kind(), ErrorKind::IoError);
        assert!(error.context().unwrap().contains("block 5"));
    }
}
