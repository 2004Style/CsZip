//! Módulo de interfaz de línea de comandos
//!
//! Define la estructura de comandos y argumentos para el ejecutable cszip.

pub mod args;
pub mod commands;
pub mod progress;

pub use args::{Cli, Commands};
pub use progress::{ProgressBar, ProgressConfig, ProgressStyle};
