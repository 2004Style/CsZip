//! Módulo de entrada/salida
//!
//! Proporciona lectores y escritores de alto nivel para archivos CsZip.

pub mod reader;
pub mod streaming;
pub mod writer;

pub use reader::CzReader;
pub use streaming::{
    StreamOptions, StreamProgress, StreamStats, StreamingCompressor, StreamingDecompressor,
};
pub use writer::CzWriter;
