//! Módulo de manejo de errores para CsZip
//!
//! Define todos los tipos de error posibles durante la compresión y descompresión.

use std::fmt;
use std::io;

/// Tipos de errores específicos del formato CsZip
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Magic number inválido (no es 0x435A ni 0x5A43)
    InvalidMagicNumber,
    /// Versión del formato no soportada
    UnsupportedVersion,
    /// Algoritmo de compresión no implementado
    UnsupportedAlgorithm,
    /// Tamaño de bloque fuera de rango válido [9, 30]
    InvalidBlockSize,
    /// Límite de expansión inválido [100, 5000]
    InvalidExpansionLimit,
    /// Tipo de bloque desconocido o inválido
    InvalidBlockType,
    /// Checksum CRC no coincide
    BlockCrcMismatch,
    /// Ratio de expansión sospechoso (potencial zip bomb)
    CompressionBombSuspected,
    /// Bloque marcado como incompleto
    IncompleteBlockFound,
    /// Header de bloque corrompido
    CorruptedBlockHeader,
    /// Footer del archivo corrompido
    CorruptedFileFooter,
    /// Checksum ADLER-32 no coincide
    InvalidAdler32Checksum,
    /// Se excedería el límite de memoria
    MemoryLimitExceeded,
    /// Fin de archivo inesperado
    UnexpectedEof,
    /// Nivel de compresión inválido
    InvalidCompressionLevel,
    /// Error de I/O
    Io,
    /// Archivo no encontrado
    FileNotFound,
    /// Archivo ya existe
    FileExists,
    /// Error de I/O genérico
    IoError,
    /// Datos inválidos
    InvalidData,
    /// Checksum no coincide
    ChecksumMismatch,
}

/// Error principal de CsZip con información detallada
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    context: Option<String>,
}

impl Error {
    /// Crear un nuevo error con tipo y mensaje
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
        }
    }

    /// Agregar contexto adicional al error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Obtener el tipo de error
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Obtener el contexto del error si existe
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    /// Obtener el codigo numerico del error
    pub fn code(&self) -> u8 {
        match self.kind {
            ErrorKind::InvalidMagicNumber => 0x01,
            ErrorKind::UnsupportedVersion => 0x02,
            ErrorKind::UnsupportedAlgorithm => 0x03,
            ErrorKind::InvalidBlockSize => 0x04,
            ErrorKind::InvalidExpansionLimit => 0x05,
            ErrorKind::InvalidBlockType => 0x06,
            ErrorKind::BlockCrcMismatch => 0x07,
            ErrorKind::CompressionBombSuspected => 0x08,
            ErrorKind::IncompleteBlockFound => 0x09,
            ErrorKind::CorruptedBlockHeader => 0x0A,
            ErrorKind::CorruptedFileFooter => 0x0B,
            ErrorKind::InvalidAdler32Checksum => 0x0C,
            ErrorKind::MemoryLimitExceeded => 0x0D,
            ErrorKind::UnexpectedEof => 0x0E,
            ErrorKind::InvalidCompressionLevel => 0x0F,
            ErrorKind::Io => 0x10,
            ErrorKind::FileNotFound => 0x11,
            ErrorKind::FileExists => 0x12,
            ErrorKind::IoError => 0x13,
            ErrorKind::InvalidData => 0x14,
            ErrorKind::ChecksumMismatch => 0x15,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[CsZip Error 0x{:02X}] {}", self.code(), self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        let kind = match err.kind() {
            io::ErrorKind::UnexpectedEof => ErrorKind::UnexpectedEof,
            _ => ErrorKind::Io,
        };
        Error::new(kind, err.to_string())
    }
}

/// Tipo Result personalizado para CsZip
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::new(ErrorKind::InvalidMagicNumber, "Magic number inválido");
        assert!(err.to_string().contains("0x01"));
        assert!(err.to_string().contains("Magic number inválido"));
    }

    #[test]
    fn test_error_with_context() {
        let err =
            Error::new(ErrorKind::BlockCrcMismatch, "CRC no coincide").with_context("bloque 5");
        assert!(err.to_string().contains("bloque 5"));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::new(ErrorKind::InvalidMagicNumber, "").code(), 0x01);
        assert_eq!(Error::new(ErrorKind::UnsupportedVersion, "").code(), 0x02);
        assert_eq!(Error::new(ErrorKind::BlockCrcMismatch, "").code(), 0x07);
        assert_eq!(Error::new(ErrorKind::UnexpectedEof, "").code(), 0x0E);
    }
}
