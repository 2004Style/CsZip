//! Implementación del compresor
//!
//! Proporciona la interfaz de compresión para todos los algoritmos soportados.

use std::io::{Read, Write};

use crate::error::{Error, ErrorKind};
use crate::format::checksum::{Adler32, Crc32, Crc64};
use crate::format::constants;

use super::{Algorithm, CompressionLevel};

/// Compresor de datos
pub struct Compressor {
    algorithm: Algorithm,
    level: CompressionLevel,
    use_crc64: bool,
}

impl Compressor {
    /// Crea un nuevo compresor con el algoritmo y nivel especificados
    pub fn new(algorithm: Algorithm, level: CompressionLevel) -> Self {
        Compressor {
            algorithm,
            level,
            use_crc64: false,
        }
    }

    /// Crea un compresor STORE (sin compresión)
    pub fn store() -> Self {
        Compressor::new(Algorithm::Store, CompressionLevel::MIN)
    }

    /// Configura el uso de CRC-64 en lugar de CRC-32
    pub fn with_crc64(mut self, use_crc64: bool) -> Self {
        self.use_crc64 = use_crc64;
        self
    }

    /// Obtiene el algoritmo configurado
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Obtiene el nivel de compresión configurado
    pub fn level(&self) -> CompressionLevel {
        self.level
    }

    /// Comprime datos desde un reader hacia un writer
    ///
    /// Retorna una tupla (bytes_escritos, checksum)
    pub fn compress<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressResult, Error> {
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
            Algorithm::Store => self.compress_store(reader, writer),
            _ => unreachable!(),
        }
    }

    /// Comprime un bloque de datos en memoria
    pub fn compress_block(&self, input: &[u8]) -> Result<CompressResult, Error> {
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
            Algorithm::Store => self.compress_block_store(input),
            _ => unreachable!(),
        }
    }

    /// Implementación STORE: copia directa sin compresión
    fn compress_store<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<CompressResult, Error> {
        let mut buffer = [0u8; constants::DEFAULT_BUFFER_SIZE];
        let mut total_written = 0u64;
        let mut total_read = 0u64;

        let mut crc32 = Crc32::new();
        let mut crc64 = Crc64::new();
        let mut adler = Adler32::new();

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| {
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

            // Escribir datos sin modificar
            writer.write_all(data).map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error escribiendo datos: {}", e))
            })?;

            total_read += bytes_read as u64;
            total_written += bytes_read as u64;
        }

        Ok(CompressResult {
            original_size: total_read,
            compressed_size: total_written,
            crc32: crc32.finalize(),
            crc64: crc64.finalize(),
            adler32: adler.finalize(),
        })
    }

    /// Implementación STORE para bloques en memoria
    fn compress_block_store(&self, input: &[u8]) -> Result<CompressResult, Error> {
        let crc32 = Crc32::compute(input);
        let crc64 = Crc64::compute(input);
        let adler32 = Adler32::compute(input);

        Ok(CompressResult {
            original_size: input.len() as u64,
            compressed_size: input.len() as u64,
            crc32,
            crc64,
            adler32,
        })
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Compressor::new(Algorithm::Store, CompressionLevel::default())
    }
}

/// Resultado de una operación de compresión
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressResult {
    /// Tamaño original de los datos
    pub original_size: u64,
    /// Tamaño comprimido de los datos
    pub compressed_size: u64,
    /// Checksum CRC-32
    pub crc32: u32,
    /// Checksum CRC-64
    pub crc64: u64,
    /// Checksum ADLER-32
    pub adler32: u32,
}

impl CompressResult {
    /// Calcula la ratio de compresión (comprimido / original)
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }

    /// Calcula el porcentaje de reducción
    pub fn savings_percent(&self) -> f64 {
        (1.0 - self.ratio()) * 100.0
    }

    /// Verifica si hubo reducción de tamaño
    pub fn did_compress(&self) -> bool {
        self.compressed_size < self.original_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_compressor_store_empty() {
        let compressor = Compressor::store();
        let input: &[u8] = &[];
        let mut output = Vec::new();

        let result = compressor
            .compress(&mut Cursor::new(input), &mut output)
            .unwrap();

        assert_eq!(result.original_size, 0);
        assert_eq!(result.compressed_size, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_compressor_store_data() {
        let compressor = Compressor::store();
        let input = b"Hello, CsZip compression!";
        let mut output = Vec::new();

        let result = compressor
            .compress(&mut Cursor::new(input.as_slice()), &mut output)
            .unwrap();

        assert_eq!(result.original_size, input.len() as u64);
        assert_eq!(result.compressed_size, input.len() as u64);
        assert_eq!(output, input);
        assert_eq!(result.ratio(), 1.0);
        assert_eq!(result.savings_percent(), 0.0);
        assert!(!result.did_compress());
    }

    #[test]
    fn test_compressor_store_block() {
        let compressor = Compressor::store();
        let input = b"Block compression test data";

        let result = compressor.compress_block(input).unwrap();

        assert_eq!(result.original_size, input.len() as u64);
        assert_eq!(result.compressed_size, input.len() as u64);
    }

    #[test]
    fn test_compressor_store_checksum_consistency() {
        let compressor = Compressor::store();
        let input = b"123456789";
        let mut output = Vec::new();

        let result = compressor
            .compress(&mut Cursor::new(input.as_slice()), &mut output)
            .unwrap();

        // Valor conocido de CRC-32 para "123456789"
        assert_eq!(result.crc32, 0xCBF43926);
    }

    #[test]
    fn test_compressor_unsupported_algorithm() {
        let compressor = Compressor::new(Algorithm::Lz77Huffman, CompressionLevel::DEFAULT);
        let input = b"test";
        let mut output = Vec::new();

        let result = compressor.compress(&mut Cursor::new(input.as_slice()), &mut output);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnsupportedAlgorithm);
    }

    #[test]
    fn test_compress_result_ratio() {
        let result = CompressResult {
            original_size: 100,
            compressed_size: 50,
            crc32: 0,
            crc64: 0,
            adler32: 0,
        };

        assert_eq!(result.ratio(), 0.5);
        assert_eq!(result.savings_percent(), 50.0);
        assert!(result.did_compress());
    }

    #[test]
    fn test_compress_result_zero_size() {
        let result = CompressResult {
            original_size: 0,
            compressed_size: 0,
            crc32: 0,
            crc64: 0,
            adler32: 0,
        };

        assert_eq!(result.ratio(), 1.0);
    }

    #[test]
    fn test_compressor_with_crc64() {
        let compressor = Compressor::store().with_crc64(true);
        assert!(compressor.use_crc64);
    }

    #[test]
    fn test_compressor_large_data() {
        let compressor = Compressor::store();
        // Generar 100KB de datos
        let input: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let mut output = Vec::new();

        let result = compressor
            .compress(&mut Cursor::new(&input), &mut output)
            .unwrap();

        assert_eq!(result.original_size, 100_000);
        assert_eq!(result.compressed_size, 100_000);
        assert_eq!(output.len(), 100_000);
        assert_eq!(output, input);
    }
}
