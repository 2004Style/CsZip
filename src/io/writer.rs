//! Escritor de archivos CsZip
//!
//! Permite crear archivos .cz con compresión por bloques.

use std::fs::File;
use std::io::{BufWriter, Read, Seek, Write};
use std::path::Path;

use crate::codec::{Algorithm, CompressionLevel, Compressor};
use crate::error::{Error, ErrorKind};
use crate::format::checksum::Crc32;
use crate::format::constants;
use crate::format::{BlockHeader, FileFooter, Header};

/// Escritor de archivos CsZip
pub struct CzWriter<W: Write + Seek> {
    writer: W,
    header: Header,
    compressor: Compressor,
    block_count: u32,
    total_original: u64,
    total_compressed: u64,
    global_crc32: Crc32,
}

impl CzWriter<BufWriter<File>> {
    /// Crea un nuevo archivo CsZip
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::create_with_options(path, Algorithm::Store, CompressionLevel::default())
    }

    /// Crea un nuevo archivo CsZip con opciones
    pub fn create_with_options<P: AsRef<Path>>(
        path: P,
        algorithm: Algorithm,
        level: CompressionLevel,
    ) -> Result<Self, Error> {
        let file = File::create(path.as_ref())
            .map_err(|e| Error::new(ErrorKind::IoError, format!("Error creando archivo: {}", e)))?;

        let writer = BufWriter::new(file);
        Self::new_with_options(writer, algorithm, level)
    }
}

impl<W: Write + Seek> CzWriter<W> {
    /// Crea un nuevo escritor CsZip sobre un writer existente
    pub fn new(writer: W) -> Result<Self, Error> {
        Self::new_with_options(writer, Algorithm::Store, CompressionLevel::default())
    }

    /// Crea un nuevo escritor CsZip con opciones
    pub fn new_with_options(
        mut writer: W,
        algorithm: Algorithm,
        level: CompressionLevel,
    ) -> Result<Self, Error> {
        // Crear header con configuracion por defecto
        let header = Header::new(
            algorithm.id(),
            constants::DEFAULT_BLOCK_SIZE_LOG2,
            constants::DEFAULT_EXPANSION,
        )?;

        // Escribir header
        header.write(&mut writer)?;

        let compressor = Compressor::new(algorithm, level);

        Ok(CzWriter {
            writer,
            header,
            compressor,
            block_count: 0,
            total_original: 0,
            total_compressed: 0,
            global_crc32: Crc32::new(),
        })
    }

    /// Retorna el tamaño de bloque configurado
    pub fn block_size(&self) -> usize {
        self.header.block_size()
    }

    /// Retorna el algoritmo de compresión
    pub fn algorithm(&self) -> Algorithm {
        self.compressor.algorithm()
    }

    /// Escribe datos desde un reader, dividiéndolos en bloques
    pub fn write_stream<R: Read>(&mut self, reader: &mut R) -> Result<(), Error> {
        let block_size = self.block_size();
        let mut buffer = vec![0u8; block_size];

        loop {
            // Leer un bloque completo
            let mut bytes_read = 0;
            while bytes_read < block_size {
                match reader.read(&mut buffer[bytes_read..]) {
                    Ok(0) => break, // EOF
                    Ok(n) => bytes_read += n,
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::IoError,
                            format!("Error leyendo datos: {}", e),
                        ))
                    }
                }
            }

            if bytes_read == 0 {
                break; // No más datos
            }

            // Comprimir y escribir el bloque
            self.write_block(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    /// Escribe un bloque de datos
    pub fn write_block(&mut self, data: &[u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Ok(());
        }

        // Actualizar CRC global
        self.global_crc32.update(data);

        // Comprimir el bloque
        let mut compressed_buf = Vec::new();
        let compress_result = if self.algorithm() == Algorithm::Store {
            self.compressor.compress_block(data)?
        } else {
            let mut cursor_in = std::io::Cursor::new(data);
            self.compressor
                .compress(&mut cursor_in, &mut compressed_buf)?
        };

        // Para STORE, los datos comprimidos son los mismos que los originales
        let compressed_data = if self.algorithm() == Algorithm::Store {
            data
        } else {
            &compressed_buf
        };

        // Crear block header
        let block_header = BlockHeader::new(
            data.len() as u16,
            compressed_data.len() as u32,
            compress_result.adler32,
            self.compressor.level().value(),
        )?;

        // Escribir block header
        let header_bytes = block_header.to_bytes();
        self.writer.write_all(&header_bytes).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error escribiendo block header: {}", e),
            )
        })?;

        // Escribir datos comprimidos
        self.writer.write_all(compressed_data).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error escribiendo datos: {}", e),
            )
        })?;

        // Actualizar estadísticas
        self.block_count += 1;
        self.total_original += data.len() as u64;
        self.total_compressed += compressed_data.len() as u64;

        Ok(())
    }

    /// Finaliza el archivo escribiendo el footer
    pub fn finish(mut self) -> Result<WriteStats, Error> {
        let global_crc = self.global_crc32.finalize();

        // Crear y escribir footer con checksum
        let footer =
            FileFooter::with_checksum(self.block_count, self.total_original as u32, global_crc)?;

        let footer_bytes = footer.to_bytes();
        self.writer.write_all(&footer_bytes).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error escribiendo footer: {}", e),
            )
        })?;

        // Flush final
        self.writer
            .flush()
            .map_err(|e| Error::new(ErrorKind::IoError, format!("Error en flush: {}", e)))?;

        Ok(WriteStats {
            block_count: self.block_count,
            original_size: self.total_original,
            compressed_size: self.total_compressed,
            global_crc32: global_crc,
        })
    }

    /// Cancela la escritura sin finalizar el archivo
    /// Útil para manejo de errores
    pub fn abort(self) {
        // Simplemente drop el writer
        drop(self.writer);
    }
}

/// Estadísticas de escritura
#[derive(Debug, Clone, Copy)]
pub struct WriteStats {
    /// Número de bloques escritos
    pub block_count: u32,
    /// Tamaño original total
    pub original_size: u64,
    /// Tamaño comprimido total
    pub compressed_size: u64,
    /// CRC-32 global
    pub global_crc32: u32,
}

impl WriteStats {
    /// Calcula la ratio de compresión
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.original_size as f64
    }

    /// Calcula el porcentaje de ahorro
    pub fn savings_percent(&self) -> f64 {
        (1.0 - self.ratio()) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_writer_empty() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = CzWriter::new(cursor).unwrap();

        let stats = writer.finish().unwrap();
        assert_eq!(stats.block_count, 0);
        assert_eq!(stats.original_size, 0);
        assert_eq!(stats.compressed_size, 0);
    }

    #[test]
    fn test_writer_single_block() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = CzWriter::new(cursor).unwrap();

        let data = b"Hello, CsZip!";
        writer.write_block(data).unwrap();

        let stats = writer.finish().unwrap();
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.original_size, data.len() as u64);
    }

    #[test]
    fn test_writer_multiple_blocks() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = CzWriter::new(cursor).unwrap();

        for i in 0..5 {
            let data = format!("Block {}", i);
            writer.write_block(data.as_bytes()).unwrap();
        }

        let stats = writer.finish().unwrap();
        assert_eq!(stats.block_count, 5);
    }

    #[test]
    fn test_writer_stream() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = CzWriter::new(cursor).unwrap();

        let data = vec![0u8; 1024 * 10]; // 10KB
        writer.write_stream(&mut Cursor::new(&data)).unwrap();

        let stats = writer.finish().unwrap();
        assert!(stats.block_count >= 1);
        assert_eq!(stats.original_size, 10240);
    }

    #[test]
    fn test_writer_block_size() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = CzWriter::new(cursor).unwrap();

        // Tamaño por defecto: 2^16 = 65536
        assert_eq!(writer.block_size(), 1 << constants::DEFAULT_BLOCK_SIZE_EXP);
    }

    #[test]
    fn test_writer_stats_ratio() {
        let stats = WriteStats {
            block_count: 1,
            original_size: 100,
            compressed_size: 50,
            global_crc32: 0,
        };

        assert_eq!(stats.ratio(), 0.5);
        assert_eq!(stats.savings_percent(), 50.0);
    }
}
