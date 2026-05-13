//! Módulo de formato CsZip
//!
//! Define las estructuras y constantes del formato binario .cz
//!
//! # Estructura del archivo
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │   File Header (16 bytes)        │
//! ├─────────────────────────────────┤
//! │   Block 0                       │
//! │   ├── Block Header (12 bytes)   │
//! │   ├── Compressed Data           │
//! │   └── Block CRC (4/8 bytes)     │
//! ├─────────────────────────────────┤
//! │   Block 1 ...                   │
//! ├─────────────────────────────────┤
//! │   File Footer (12 bytes)        │
//! └─────────────────────────────────┘
//! ```

pub mod block;
pub mod checksum;
pub mod constants;
pub mod header;

// Re-exportar tipos públicos principales
pub use block::{BlockHeader, FileFooter};
pub use checksum::{Adler32, Crc32, Crc64};
pub use constants::*;
pub use header::Header;
