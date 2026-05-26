//! Streaming para procesamiento de archivos grandes
//!
//! Proporciona interfaces para comprimir y descomprimir datos sin
//! cargar el archivo completo en memoria.

use std::io::{Read, Seek, SeekFrom, Write};

use crate::codec::{Algorithm, CompressionLevel, Compressor, Decompressor};
use crate::error::{Error, ErrorKind, Result};
use crate::format::checksum::Crc32;
use crate::format::constants;
use crate::format::{BlockHeader, FileFooter, Header};

/// Callback de progreso
pub type ProgressCallback = Box<dyn Fn(StreamProgress) + Send>;

/// Información de progreso
#[derive(Debug, Clone, Copy)]
pub struct StreamProgress {
    /// Bytes procesados
    pub bytes_processed: u64,
    /// Bytes totales (si se conoce)
    pub bytes_total: Option<u64>,
    /// Bloques procesados
    pub blocks_processed: u32,
    /// Bloques totales (si se conoce)
    pub blocks_total: Option<u32>,
}

impl StreamProgress {
    /// Calcular porcentaje completado
    pub fn percentage(&self) -> Option<f64> {
        self.bytes_total.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.bytes_processed as f64 / total as f64) * 100.0
            }
        })
    }
}

/// Opciones de streaming
#[derive(Debug, Clone)]
pub struct StreamOptions {
    /// Tamaño de bloque
    pub block_size: usize,
    /// Algoritmo de compresión
    pub algorithm: Algorithm,
    /// Nivel de compresión
    pub level: CompressionLevel,
    /// Usar CRC-64 en lugar de CRC-32
    pub use_crc64: bool,
    /// Límite de memoria (bytes)
    pub memory_limit: Option<usize>,
    /// Verificar checksums al descomprimir
    pub verify_checksums: bool,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            block_size: constants::DEFAULT_BLOCK_SIZE,
            algorithm: Algorithm::Store,
            level: CompressionLevel::default(),
            use_crc64: false,
            memory_limit: None,
            verify_checksums: true,
        }
    }
}

impl StreamOptions {
    /// Crear opciones con tamaño de bloque específico
    pub fn with_block_size(mut self, size: usize) -> Self {
        self.block_size = size;
        self
    }

    /// Establecer algoritmo
    pub fn with_algorithm(mut self, algo: Algorithm) -> Self {
        self.algorithm = algo;
        self
    }

    /// Establecer nivel de compresión
    pub fn with_level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }

    /// Establecer límite de memoria
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = Some(limit);
        self
    }
}

/// Compresor de streaming
pub struct StreamingCompressor<W: Write + Seek> {
    writer: W,
    options: StreamOptions,
    #[allow(dead_code)]
    header: Header,
    compressor: Compressor,
    block_count: u32,
    total_original: u64,
    total_compressed: u64,
    global_crc: Crc32,
    progress_callback: Option<ProgressCallback>,
}

impl<W: Write + Seek> StreamingCompressor<W> {
    /// Crear nuevo compresor de streaming
    pub fn new(mut writer: W, options: StreamOptions) -> Result<Self> {
        let block_size_log2 = (options.block_size as f64).log2() as u16;

        let header = Header::new(
            options.algorithm.id(),
            block_size_log2,
            constants::DEFAULT_EXPANSION,
        )?;

        header.write(&mut writer)?;

        let compressor =
            Compressor::new(options.algorithm, options.level).with_crc64(options.use_crc64);

        Ok(Self {
            writer,
            options,
            header,
            compressor,
            block_count: 0,
            total_original: 0,
            total_compressed: 0,
            global_crc: Crc32::new(),
            progress_callback: None,
        })
    }

    /// Establecer callback de progreso
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Comprimir datos desde un reader
    pub fn compress_stream<R: Read>(&mut self, reader: &mut R) -> Result<()> {
        let mut buffer = vec![0u8; self.options.block_size];

        loop {
            let bytes_read = read_full_block(reader, &mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            self.write_block(&buffer[..bytes_read])?;

            if let Some(ref callback) = self.progress_callback {
                callback(StreamProgress {
                    bytes_processed: self.total_original,
                    bytes_total: None,
                    blocks_processed: self.block_count,
                    blocks_total: None,
                });
            }
        }

        Ok(())
    }

    /// Escribir un bloque de datos
    pub fn write_block(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        // Verificar límite de memoria si está establecido
        if let Some(limit) = self.options.memory_limit {
            if data.len() > limit {
                return Err(Error::new(
                    ErrorKind::MemoryLimitExceeded,
                    format!("Bloque de {} bytes excede límite de {}", data.len(), limit),
                ));
            }
        }

        // Actualizar CRC global
        self.global_crc.update(data);

        // Comprimir bloque a un buffer
        let mut compressed_data = Vec::new();
        let mut cursor_in = std::io::Cursor::new(data);
        let result = self
            .compressor
            .compress(&mut cursor_in, &mut compressed_data)?;

        // Crear y escribir header de bloque
        let block_header = BlockHeader::new(
            data.len() as u16,
            result.compressed_size as u32,
            result.adler32,
            self.options.level.value(),
        )?;

        block_header.write(&mut self.writer)?;

        // Escribir datos comprimidos
        self.writer.write_all(&compressed_data)?;

        // Escribir CRC del bloque
        self.writer.write_all(&result.crc32.to_be_bytes())?;

        // Actualizar estadísticas
        self.block_count += 1;
        self.total_original += data.len() as u64;
        self.total_compressed += result.compressed_size;

        Ok(())
    }

    /// Finalizar compresión y escribir footer
    pub fn finish(mut self) -> Result<StreamStats> {
        // Escribir footer
        let footer = FileFooter::new(self.block_count, self.total_original as u32)?;
        footer.write(&mut self.writer)?;

        self.writer.flush()?;

        Ok(StreamStats {
            original_size: self.total_original,
            compressed_size: self.total_compressed,
            block_count: self.block_count,
            crc32: self.global_crc.finalize(),
        })
    }
}

/// Descompresor de streaming
pub struct StreamingDecompressor<R: Read + Seek> {
    reader: R,
    options: StreamOptions,
    header: Header,
    decompressor: Decompressor,
    current_block: u32,
    footer: Option<FileFooter>,
    progress_callback: Option<ProgressCallback>,
}

impl<R: Read + Seek> StreamingDecompressor<R> {
    /// Crear nuevo descompresor de streaming
    pub fn new(mut reader: R, options: StreamOptions) -> Result<Self> {
        let header = Header::read(&mut reader)?;

        let algorithm = Algorithm::from_id(header.compression_algo)?;
        let decompressor =
            Decompressor::new(algorithm).with_checksum_verification(options.verify_checksums);

        Ok(Self {
            reader,
            options,
            header,
            decompressor,
            current_block: 0,
            footer: None,
            progress_callback: None,
        })
    }

    /// Establecer callback de progreso
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Descomprimir a un writer
    pub fn decompress_stream<W: Write>(&mut self, writer: &mut W) -> Result<StreamStats> {
        let mut total_original = 0u64;
        let mut total_compressed = 0u64;
        let mut global_crc = Crc32::new();

        while let Some((data, block_info)) = self.read_block()? {
            writer.write_all(&data)?;

            global_crc.update(&data);
            total_original += data.len() as u64;
            total_compressed += block_info.compressed_size as u64;

            if let Some(ref callback) = self.progress_callback {
                callback(StreamProgress {
                    bytes_processed: total_original,
                    bytes_total: self.footer.as_ref().map(|f| f.total_raw_size as u64),
                    blocks_processed: self.current_block,
                    blocks_total: self.footer.as_ref().map(|f| f.num_blocks),
                });
            }
        }

        writer.flush()?;

        Ok(StreamStats {
            original_size: total_original,
            compressed_size: total_compressed,
            block_count: self.current_block,
            crc32: global_crc.finalize(),
        })
    }

    /// Leer un bloque
    fn read_block(&mut self) -> Result<Option<(Vec<u8>, BlockHeader)>> {
        // Primero leer un byte para verificar si es footer
        let mut first_byte = [0u8; 1];
        match self.reader.read_exact(&mut first_byte) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(Error::from(e)),
        }

        // Si es el marcador de footer (0xFE), leer el footer
        if first_byte[0] == 0xFE {
            // Retroceder para leer el footer completo
            self.reader.seek(SeekFrom::Current(-1))?;
            self.footer = Some(FileFooter::read(&mut self.reader)?);
            return Ok(None);
        }

        // No es footer, leer el resto del BlockHeader
        let mut rest = [0u8; 11]; // BlockHeader::SIZE - 1 = 12 - 1 = 11
        self.reader.read_exact(&mut rest)?;

        // Combinar los bytes
        let mut header_bytes = [0u8; 12];
        header_bytes[0] = first_byte[0];
        header_bytes[1..].copy_from_slice(&rest);

        let block_header = BlockHeader::from_bytes(&header_bytes)?;

        // Verificar tipo de bloque
        if block_header.block_type == 2 {
            return Err(Error::new(
                ErrorKind::IncompleteBlockFound,
                "Bloque incompleto encontrado",
            ));
        }

        // Leer datos comprimidos
        let mut compressed = vec![0u8; block_header.compressed_size as usize];
        self.reader.read_exact(&mut compressed)?;

        // Leer CRC
        let mut crc_bytes = [0u8; 4];
        self.reader.read_exact(&mut crc_bytes)?;
        let expected_crc = u32::from_be_bytes(crc_bytes);

        // Descomprimir usando decompress() con buffers
        let mut cursor = std::io::Cursor::new(&compressed);
        let mut decompressed = Vec::with_capacity(block_header.original_size as usize);
        let result = self.decompressor.decompress(
            &mut cursor,
            &mut decompressed,
            Some(block_header.original_size as u64),
        )?;

        // Verificar CRC si está habilitado
        if self.options.verify_checksums && result.crc32 != expected_crc {
            return Err(Error::new(
                ErrorKind::BlockCrcMismatch,
                format!(
                    "CRC no coincide: esperado 0x{:08X}, obtenido 0x{:08X}",
                    expected_crc, result.crc32
                ),
            ));
        }

        self.current_block += 1;

        Ok(Some((decompressed, block_header)))
    }

    /// Leer footer sin procesar bloques
    pub fn read_footer(&mut self) -> Result<&FileFooter> {
        if self.footer.is_none() {
            // Ir al final - tamaño del footer
            self.reader
                .seek(SeekFrom::End(-(FileFooter::SIZE as i64)))?;
            self.footer = Some(FileFooter::read(&mut self.reader)?);
            // Volver al inicio de datos
            self.reader.seek(SeekFrom::Start(Header::SIZE as u64))?;
        }
        Ok(self.footer.as_ref().unwrap())
    }

    /// Obtener header
    pub fn header(&self) -> &Header {
        &self.header
    }
}

/// Estadísticas de streaming
#[derive(Debug, Clone, Copy)]
pub struct StreamStats {
    /// Tamaño original
    pub original_size: u64,
    /// Tamaño comprimido
    pub compressed_size: u64,
    /// Número de bloques
    pub block_count: u32,
    /// CRC-32 global
    pub crc32: u32,
}

impl StreamStats {
    /// Ratio de compresión
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 100.0;
        }
        (self.compressed_size as f64 / self.original_size as f64) * 100.0
    }

    /// Ahorro de espacio
    pub fn savings(&self) -> f64 {
        100.0 - self.ratio()
    }
}

/// Leer un bloque completo del reader
fn read_full_block<R: Read>(reader: &mut R, buffer: &mut [u8]) -> Result<usize> {
    let mut total_read = 0;

    while total_read < buffer.len() {
        match reader.read(&mut buffer[total_read..]) {
            Ok(0) => break, // EOF
            Ok(n) => total_read += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(total_read)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_stream_options_default() {
        let opts = StreamOptions::default();
        assert_eq!(opts.algorithm, Algorithm::Store);
        assert!(opts.verify_checksums);
    }

    #[test]
    fn test_stream_options_builder() {
        let opts = StreamOptions::default()
            .with_block_size(8192)
            .with_algorithm(Algorithm::Store)
            .with_memory_limit(1024 * 1024);

        assert_eq!(opts.block_size, 8192);
        assert_eq!(opts.memory_limit, Some(1024 * 1024));
    }

    #[test]
    fn test_stream_progress() {
        let progress = StreamProgress {
            bytes_processed: 50,
            bytes_total: Some(100),
            blocks_processed: 1,
            blocks_total: Some(2),
        };

        assert_eq!(progress.percentage(), Some(50.0));
    }

    #[test]
    fn test_stream_stats_ratio() {
        let stats = StreamStats {
            original_size: 100,
            compressed_size: 50,
            block_count: 1,
            crc32: 0,
        };

        assert_eq!(stats.ratio(), 50.0);
        assert_eq!(stats.savings(), 50.0);
    }

    #[test]
    fn test_streaming_roundtrip() {
        let input = b"Hello, streaming world! This is test data.".to_vec();
        let options = StreamOptions::default();

        // Comprimir
        let mut compressed = Vec::new();
        {
            let cursor = Cursor::new(&mut compressed);
            let mut compressor = StreamingCompressor::new(cursor, options.clone()).unwrap();
            let mut input_cursor = Cursor::new(&input);
            compressor.compress_stream(&mut input_cursor).unwrap();
            compressor.finish().unwrap();
        }

        // Descomprimir
        let mut decompressed = Vec::new();
        {
            let cursor = Cursor::new(&compressed);
            let mut decompressor = StreamingDecompressor::new(cursor, options).unwrap();
            decompressor.decompress_stream(&mut decompressed).unwrap();
        }

        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_read_full_block() {
        let data = b"test data here";
        let mut cursor = Cursor::new(data.as_slice());
        let mut buffer = vec![0u8; 5];

        let n = read_full_block(&mut cursor, &mut buffer).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buffer, b"test ");
    }
}
