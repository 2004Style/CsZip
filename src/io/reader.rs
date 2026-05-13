//! Lector de archivos CsZip
//!
//! Permite leer y descomprimir archivos .cz.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::codec::{Algorithm, Decompressor};
use crate::error::{Error, ErrorKind};
use crate::format::{BlockHeader, FileFooter, Header};
use crate::format::checksum::Crc32;
use crate::format::constants;

/// Lector de archivos CsZip
pub struct CzReader<R: Read + Seek> {
    reader: R,
    header: Header,
    footer: Option<FileFooter>,
    decompressor: Decompressor,
    current_block: u32,
    verify_checksums: bool,
}

impl CzReader<BufReader<File>> {
    /// Abre un archivo CsZip existente
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path.as_ref()).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error abriendo archivo: {}", e),
            )
        })?;

        let reader = BufReader::new(file);
        Self::new(reader)
    }
}

impl<R: Read + Seek> CzReader<R> {
    /// Crea un nuevo lector CsZip sobre un reader existente
    pub fn new(mut reader: R) -> Result<Self, Error> {
        // Leer y validar header
        let header = Header::read(&mut reader)?;

        // Obtener algoritmo
        let algorithm = Algorithm::from_id(header.compression_algo)?;
        let decompressor = Decompressor::new(algorithm);

        Ok(CzReader {
            reader,
            header,
            footer: None,
            decompressor,
            current_block: 0,
            verify_checksums: true,
        })
    }

    /// Configura si se deben verificar los checksums
    pub fn with_checksum_verification(mut self, verify: bool) -> Self {
        self.verify_checksums = verify;
        self.decompressor = self.decompressor.with_checksum_verification(verify);
        self
    }

    /// Retorna el header del archivo
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Retorna el footer si ya fue leído
    pub fn footer(&self) -> Option<&FileFooter> {
        self.footer.as_ref()
    }

    /// Retorna el algoritmo de compresion usado
    pub fn algorithm(&self) -> Result<Algorithm, Error> {
        Algorithm::from_id(self.header.compression_algo)
    }

    /// Retorna el tamaño de bloque
    pub fn block_size(&self) -> usize {
        self.header.block_size()
    }

    /// Lee el footer del archivo (salta al final)
    pub fn read_footer(&mut self) -> Result<&FileFooter, Error> {
        if self.footer.is_some() {
            return Ok(self.footer.as_ref().unwrap());
        }

        // Ir al final del archivo menos el tamaño del footer
        self.reader
            .seek(SeekFrom::End(-(constants::FOOTER_SIZE as i64)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("Error buscando footer: {}", e),
                )
            })?;

        // Leer footer
        let mut footer_bytes = [0u8; constants::FOOTER_SIZE];
        self.reader.read_exact(&mut footer_bytes).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error leyendo footer: {}", e),
            )
        })?;

        let footer = FileFooter::from_bytes(&footer_bytes)?;

        // Nota: la validacion basica se hace en from_bytes

        self.footer = Some(footer);

        // Volver al inicio de los datos
        self.reader
            .seek(SeekFrom::Start(constants::HEADER_SIZE as u64))
            .map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error en seek: {}", e))
            })?;

        Ok(self.footer.as_ref().unwrap())
    }

    /// Lee el siguiente bloque
    pub fn read_block(&mut self) -> Result<Option<BlockData>, Error> {
        // Leer block header
        let mut header_bytes = [0u8; constants::BLOCK_HEADER_SIZE];
        match self.reader.read_exact(&mut header_bytes) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Verificar si encontramos el footer
                return Ok(None);
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::IoError,
                    format!("Error leyendo block header: {}", e),
                ))
            }
        }

        // Verificar si esto es el footer (marcador 0xFE)
        if header_bytes[0] == constants::FILE_FOOTER_MARKER {
            return Ok(None);
        }

        let block_header = BlockHeader::from_bytes(&header_bytes)?;

        // Validar block header
        block_header.validate(&self.header)?;

        // Leer datos comprimidos
        let compressed_size = block_header.compressed_size as usize;
        let mut compressed_data = vec![0u8; compressed_size];
        self.reader.read_exact(&mut compressed_data).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error leyendo datos comprimidos: {}", e),
            )
        })?;

        // Descomprimir
        let original_size = block_header.original_size as usize;
        let mut decompressed_data = Vec::with_capacity(original_size);

        let result = self.decompressor.decompress(
            &mut std::io::Cursor::new(&compressed_data),
            &mut decompressed_data,
            Some(original_size as u64),
        )?;

        // Verificar checksum del bloque (ADLER-32)
        if self.verify_checksums {
            let expected_adler = block_header.adler32;
            if result.adler32 != expected_adler {
                return Err(Error::new(
                    ErrorKind::ChecksumMismatch,
                    format!(
                        "ADLER-32 del bloque {} no coincide: esperado 0x{:08X}, calculado 0x{:08X}",
                        self.current_block, expected_adler, result.adler32
                    ),
                ));
            }
        }

        self.current_block += 1;

        Ok(Some(BlockData {
            index: self.current_block - 1,
            original_size: original_size as u32,
            compressed_size: compressed_size as u32,
            data: decompressed_data,
            crc32: result.crc32,
        }))
    }

    /// Descomprime todo el archivo a un writer
    pub fn decompress_all<W: Write>(&mut self, writer: &mut W) -> Result<ReadStats, Error> {
        let mut total_blocks = 0u32;
        let mut total_original = 0u64;
        let mut total_compressed = 0u64;
        let mut global_crc = Crc32::new();

        // Asegurar que estamos al inicio de los datos
        self.reader
            .seek(SeekFrom::Start(constants::HEADER_SIZE as u64))
            .map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error en seek: {}", e))
            })?;

        self.current_block = 0;

        loop {
            match self.read_block()? {
                Some(block) => {
                    // Actualizar CRC global
                    global_crc.update(&block.data);

                    // Escribir datos descomprimidos
                    writer.write_all(&block.data).map_err(|e| {
                        Error::new(
                            ErrorKind::IoError,
                            format!("Error escribiendo datos: {}", e),
                        )
                    })?;

                    total_blocks += 1;
                    total_original += block.original_size as u64;
                    total_compressed += block.compressed_size as u64;
                }
                None => break,
            }
        }

        let global_crc32 = global_crc.finalize();

        // Verificar contra el footer si esta disponible
        if self.verify_checksums {
            if let Some(footer) = &self.footer {
                if footer.checksum != global_crc32 {
                    return Err(Error::new(
                        ErrorKind::ChecksumMismatch,
                        format!(
                            "CRC-32 global no coincide: esperado 0x{:08X}, calculado 0x{:08X}",
                            footer.checksum,
                            global_crc32
                        ),
                    ));
                }
            }
        }

        Ok(ReadStats {
            block_count: total_blocks,
            original_size: total_original,
            compressed_size: total_compressed,
            global_crc32,
        })
    }

    /// Reinicia la lectura al primer bloque
    pub fn rewind(&mut self) -> Result<(), Error> {
        self.reader
            .seek(SeekFrom::Start(constants::HEADER_SIZE as u64))
            .map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error en seek: {}", e))
            })?;

        self.current_block = 0;
        Ok(())
    }
}

/// Datos de un bloque leído
#[derive(Debug, Clone)]
pub struct BlockData {
    /// Índice del bloque (0-based)
    pub index: u32,
    /// Tamaño original
    pub original_size: u32,
    /// Tamaño comprimido
    pub compressed_size: u32,
    /// Datos descomprimidos
    pub data: Vec<u8>,
    /// CRC-32 del bloque
    pub crc32: u32,
}

/// Estadísticas de lectura
#[derive(Debug, Clone, Copy)]
pub struct ReadStats {
    /// Número de bloques leídos
    pub block_count: u32,
    /// Tamaño original total
    pub original_size: u64,
    /// Tamaño comprimido total
    pub compressed_size: u64,
    /// CRC-32 global calculado
    pub global_crc32: u32,
}

impl ReadStats {
    /// Calcula la ratio de compresión
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::CzWriter;
    use std::io::Cursor;

    #[allow(dead_code)]
    fn create_test_archive(data: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = CzWriter::new(cursor).unwrap();
        if !data.is_empty() {
            writer.write_block(data).unwrap();
        }
        writer.finish().unwrap();
        buffer
    }

    #[test]
    fn test_reader_header() {
        // Crear un archivo mínimo
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = CzWriter::new(cursor).unwrap();
        let _ = writer.finish().unwrap();

        // No podemos obtener el buffer directamente, así que creamos uno nuevo
        let mut buffer: Vec<u8> = Vec::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            let header = Header::new(0, 15, 1000).unwrap();
            header.write(&mut cursor).unwrap();
            let footer = FileFooter::new(0, 0).unwrap();
            cursor.write_all(&footer.to_bytes()).unwrap();
        }

        let cursor = Cursor::new(buffer);
        let reader = CzReader::new(cursor).unwrap();

        assert_eq!(reader.header().version_major, 1);
        assert_eq!(reader.header().version_minor, 0);
        assert_eq!(reader.header().compression_algo, 0);
    }

    #[test]
    fn test_roundtrip_simple() {
        let original_data = b"Hello, CsZip roundtrip test!";

        // Escribir
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original_data).unwrap();
            writer.finish().unwrap();
        }

        // Leer
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();

        let block = reader.read_block().unwrap().unwrap();
        assert_eq!(block.data, original_data);
    }

    #[test]
    fn test_roundtrip_multiple_blocks() {
        let blocks: Vec<&[u8]> = vec![
            b"First block",
            b"Second block with more data",
            b"Third block",
        ];

        // Escribir
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            for block in &blocks {
                writer.write_block(*block).unwrap();
            }
            writer.finish().unwrap();
        }

        // Leer
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();

        for (i, expected) in blocks.iter().enumerate() {
            let block = reader.read_block().unwrap();
            assert!(block.is_some(), "Block {} should exist", i);
            assert_eq!(block.unwrap().data, *expected);
        }
    }

    #[test]
    fn test_decompress_all() {
        let original_data = b"Complete decompression test data!";

        // Escribir
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(original_data).unwrap();
            writer.finish().unwrap();
        }

        // Leer y descomprimir todo
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();

        let mut output = Vec::new();
        let stats = reader.decompress_all(&mut output).unwrap();

        assert_eq!(output, original_data);
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, original_data.len() as u64);
    }

    #[test]
    fn test_reader_verify_checksums() {
        let data = b"Checksum verification test";

        // Escribir
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(data).unwrap();
            writer.finish().unwrap();
        }

        // Corromper datos (modificar un byte en los datos comprimidos)
        // El header tiene 16 bytes, el block header tiene 12 bytes
        // Así que los datos empiezan en offset 28
        if buffer.len() > 30 {
            buffer[30] ^= 0xFF; // Invertir un byte
        }

        // Intentar leer - debería fallar verificación
        let cursor = Cursor::new(&buffer);
        let mut reader = CzReader::new(cursor).unwrap();

        let result = reader.read_block();
        assert!(result.is_err() || result.unwrap().is_none());
    }

    #[test]
    fn test_reader_skip_verification() {
        let data = b"Skip verification test";

        // Escribir
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = CzWriter::new(cursor).unwrap();
            writer.write_block(data).unwrap();
            writer.finish().unwrap();
        }

        // Leer sin verificación
        let cursor = Cursor::new(&buffer);
        let reader = CzReader::new(cursor)
            .unwrap()
            .with_checksum_verification(false);

        assert!(!reader.verify_checksums);
    }
}
