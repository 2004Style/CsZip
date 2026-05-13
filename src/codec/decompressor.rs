//! Implementación del descompresor
//!
//! Proporciona la interfaz de descompresión para todos los algoritmos soportados.

use std::io::{Read, Write};

use crate::error::{Error, ErrorKind};
use crate::format::checksum::{Adler32, Crc32, Crc64};
use crate::format::constants;

use super::Algorithm;

/// Descompresor de datos
pub struct Decompressor {
    algorithm: Algorithm,
    verify_checksum: bool,
}

impl Decompressor {
    /// Crea un nuevo descompresor para el algoritmo especificado
    pub fn new(algorithm: Algorithm) -> Self {
        Decompressor {
            algorithm,
            verify_checksum: true,
        }
    }

    /// Crea un descompresor STORE
    pub fn store() -> Self {
        Decompressor::new(Algorithm::Store)
    }

    /// Configura si se debe verificar el checksum
    pub fn with_checksum_verification(mut self, verify: bool) -> Self {
        self.verify_checksum = verify;
        self
    }

    /// Obtiene el algoritmo configurado
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Descomprime datos desde un reader hacia un writer
    ///
    /// Retorna el resultado de la descompresión con checksums calculados
    pub fn decompress<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
        expected_size: Option<u64>,
    ) -> Result<DecompressResult, Error> {
        if !self.algorithm.is_implemented() {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                format!(
                    "Algoritmo {} no está implementado aún",
                    self.algorithm.name()
                ),
            ));
        }

        match self.algorithm {
            Algorithm::Store => self.decompress_store(reader, writer, expected_size),
            _ => unreachable!(),
        }
    }

    /// Descomprime un bloque de datos en memoria
    pub fn decompress_block(&self, input: &[u8]) -> Result<DecompressResult, Error> {
        if !self.algorithm.is_implemented() {
            return Err(Error::new(
                ErrorKind::UnsupportedAlgorithm,
                format!(
                    "Algoritmo {} no está implementado aún",
                    self.algorithm.name()
                ),
            ));
        }

        match self.algorithm {
            Algorithm::Store => self.decompress_block_store(input),
            _ => unreachable!(),
        }
    }

    /// Descomprime y verifica contra un checksum CRC-32
    pub fn decompress_and_verify_crc32<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
        expected_crc32: u32,
        expected_size: Option<u64>,
    ) -> Result<DecompressResult, Error> {
        let result = self.decompress(reader, writer, expected_size)?;

        if self.verify_checksum && result.crc32 != expected_crc32 {
            return Err(Error::new(
                ErrorKind::ChecksumMismatch,
                format!(
                    "CRC-32 no coincide: esperado 0x{:08X}, calculado 0x{:08X}",
                    expected_crc32, result.crc32
                ),
            ));
        }

        Ok(result)
    }

    /// Descomprime y verifica contra un checksum CRC-64
    pub fn decompress_and_verify_crc64<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
        expected_crc64: u64,
        expected_size: Option<u64>,
    ) -> Result<DecompressResult, Error> {
        let result = self.decompress(reader, writer, expected_size)?;

        if self.verify_checksum && result.crc64 != expected_crc64 {
            return Err(Error::new(
                ErrorKind::ChecksumMismatch,
                format!(
                    "CRC-64 no coincide: esperado 0x{:016X}, calculado 0x{:016X}",
                    expected_crc64, result.crc64
                ),
            ));
        }

        Ok(result)
    }

    /// Implementación STORE: copia directa sin descompresión
    fn decompress_store<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
        expected_size: Option<u64>,
    ) -> Result<DecompressResult, Error> {
        let mut buffer = [0u8; constants::DEFAULT_BUFFER_SIZE];
        let mut total_read = 0u64;
        let mut total_written = 0u64;

        let mut crc32 = Crc32::new();
        let mut crc64 = Crc64::new();
        let mut adler = Adler32::new();

        loop {
            // Si conocemos el tamaño esperado, limitar la lectura
            let max_read = if let Some(expected) = expected_size {
                let remaining = expected.saturating_sub(total_read);
                if remaining == 0 {
                    break;
                }
                std::cmp::min(remaining as usize, buffer.len())
            } else {
                buffer.len()
            };

            let bytes_read = reader.read(&mut buffer[..max_read]).map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error leyendo datos: {}", e))
            })?;

            if bytes_read == 0 {
                break;
            }

            let data = &buffer[..bytes_read];

            // Actualizar checksums
            crc32.update(data);
            crc64.update(data);
            adler.update(data);

            // Escribir datos
            writer.write_all(data).map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error escribiendo datos: {}", e))
            })?;

            total_read += bytes_read as u64;
            total_written += bytes_read as u64;
        }

        // Verificar tamaño si se especificó
        if let Some(expected) = expected_size {
            if total_written != expected {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Tamaño descomprimido incorrecto: esperado {}, obtenido {}",
                        expected, total_written
                    ),
                ));
            }
        }

        Ok(DecompressResult {
            compressed_size: total_read,
            decompressed_size: total_written,
            crc32: crc32.finalize(),
            crc64: crc64.finalize(),
            adler32: adler.finalize(),
        })
    }

    /// Implementación STORE para bloques en memoria
    fn decompress_block_store(&self, input: &[u8]) -> Result<DecompressResult, Error> {
        let crc32 = Crc32::compute(input);
        let crc64 = Crc64::compute(input);
        let adler32 = Adler32::compute(input);

        Ok(DecompressResult {
            compressed_size: input.len() as u64,
            decompressed_size: input.len() as u64,
            crc32,
            crc64,
            adler32,
        })
    }
}

impl Default for Decompressor {
    fn default() -> Self {
        Decompressor::store()
    }
}

/// Resultado de una operación de descompresión
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompressResult {
    /// Tamaño de los datos comprimidos leídos
    pub compressed_size: u64,
    /// Tamaño de los datos descomprimidos
    pub decompressed_size: u64,
    /// Checksum CRC-32 calculado
    pub crc32: u32,
    /// Checksum CRC-64 calculado
    pub crc64: u64,
    /// Checksum ADLER-32 calculado
    pub adler32: u32,
}

impl DecompressResult {
    /// Verifica si el CRC-32 coincide con el esperado
    pub fn verify_crc32(&self, expected: u32) -> bool {
        self.crc32 == expected
    }

    /// Verifica si el CRC-64 coincide con el esperado
    pub fn verify_crc64(&self, expected: u64) -> bool {
        self.crc64 == expected
    }

    /// Verifica si el ADLER-32 coincide con el esperado
    pub fn verify_adler32(&self, expected: u32) -> bool {
        self.adler32 == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Compressor;
    use std::io::Cursor;

    #[test]
    fn test_decompressor_store_empty() {
        let decompressor = Decompressor::store();
        let input: &[u8] = &[];
        let mut output = Vec::new();

        let result = decompressor
            .decompress(&mut Cursor::new(input), &mut output, None)
            .unwrap();

        assert_eq!(result.compressed_size, 0);
        assert_eq!(result.decompressed_size, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_decompressor_store_data() {
        let decompressor = Decompressor::store();
        let input = b"Hello, CsZip decompression!";
        let mut output = Vec::new();

        let result = decompressor
            .decompress(&mut Cursor::new(input.as_slice()), &mut output, None)
            .unwrap();

        assert_eq!(result.compressed_size, input.len() as u64);
        assert_eq!(result.decompressed_size, input.len() as u64);
        assert_eq!(output, input);
    }

    #[test]
    fn test_decompressor_store_with_expected_size() {
        let decompressor = Decompressor::store();
        let input = b"Exact size test";
        let mut output = Vec::new();

        let result = decompressor
            .decompress(
                &mut Cursor::new(input.as_slice()),
                &mut output,
                Some(input.len() as u64),
            )
            .unwrap();

        assert_eq!(result.decompressed_size, input.len() as u64);
    }

    #[test]
    fn test_decompressor_store_size_mismatch() {
        let decompressor = Decompressor::store();
        let input = b"Short";
        let mut output = Vec::new();

        let result = decompressor.decompress(
            &mut Cursor::new(input.as_slice()),
            &mut output,
            Some(100), // Esperamos 100 bytes pero solo hay 5
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_roundtrip_store() {
        let original = b"Roundtrip test data for CsZip compression system!";

        // Comprimir
        let compressor = Compressor::store();
        let mut compressed = Vec::new();
        let compress_result = compressor
            .compress(&mut Cursor::new(original.as_slice()), &mut compressed)
            .unwrap();

        // Descomprimir
        let decompressor = Decompressor::store();
        let mut decompressed = Vec::new();
        let decompress_result = decompressor
            .decompress(&mut Cursor::new(&compressed), &mut decompressed, None)
            .unwrap();

        // Verificar
        assert_eq!(decompressed, original);
        assert_eq!(compress_result.crc32, decompress_result.crc32);
        assert_eq!(compress_result.crc64, decompress_result.crc64);
    }

    #[test]
    fn test_decompressor_verify_crc32_success() {
        let decompressor = Decompressor::store();
        let input = b"123456789";
        let expected_crc32 = 0xCBF43926;
        let mut output = Vec::new();

        let result = decompressor
            .decompress_and_verify_crc32(
                &mut Cursor::new(input.as_slice()),
                &mut output,
                expected_crc32,
                None,
            )
            .unwrap();

        assert_eq!(result.crc32, expected_crc32);
    }

    #[test]
    fn test_decompressor_verify_crc32_failure() {
        let decompressor = Decompressor::store();
        let input = b"123456789";
        let wrong_crc32 = 0xDEADBEEF;
        let mut output = Vec::new();

        let result = decompressor.decompress_and_verify_crc32(
            &mut Cursor::new(input.as_slice()),
            &mut output,
            wrong_crc32,
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ChecksumMismatch);
    }

    #[test]
    fn test_decompressor_skip_verification() {
        let decompressor = Decompressor::store().with_checksum_verification(false);
        let input = b"123456789";
        let wrong_crc32 = 0xDEADBEEF;
        let mut output = Vec::new();

        // No debe fallar aunque el CRC sea incorrecto
        let result = decompressor.decompress_and_verify_crc32(
            &mut Cursor::new(input.as_slice()),
            &mut output,
            wrong_crc32,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_decompressor_unsupported_algorithm() {
        let decompressor = Decompressor::new(Algorithm::Lzma);
        let input = b"test";
        let mut output = Vec::new();

        let result = decompressor.decompress(&mut Cursor::new(input.as_slice()), &mut output, None);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnsupportedAlgorithm);
    }

    #[test]
    fn test_decompress_result_verify() {
        let result = DecompressResult {
            compressed_size: 100,
            decompressed_size: 100,
            crc32: 0xCBF43926,
            crc64: 0x995DC9BBDF1939FA,
            adler32: 0x091E01DE,
        };

        assert!(result.verify_crc32(0xCBF43926));
        assert!(!result.verify_crc32(0xDEADBEEF));
        assert!(result.verify_crc64(0x995DC9BBDF1939FA));
        assert!(!result.verify_crc64(0));
    }

    #[test]
    fn test_decompressor_large_data() {
        let decompressor = Decompressor::store();
        // Generar 100KB de datos
        let input: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let mut output = Vec::new();

        let result = decompressor
            .decompress(&mut Cursor::new(&input), &mut output, None)
            .unwrap();

        assert_eq!(result.compressed_size, 100_000);
        assert_eq!(result.decompressed_size, 100_000);
        assert_eq!(output, input);
    }
}
