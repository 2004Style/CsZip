//! Módulo de entrada/salida
//!
//! Proporciona lectores y escritores de alto nivel para archivos CsZip.

pub mod reader;
pub mod streaming;
pub mod writer;

pub use reader::CzReader;
pub use streaming::{StreamingCompressor, StreamingDecompressor, StreamOptions, StreamProgress, StreamStats};
pub use writer::CzWriter;
