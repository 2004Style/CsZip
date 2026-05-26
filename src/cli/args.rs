//! Definición de argumentos CLI usando clap
//!
//! Estructura de comandos:
//! - cszip compress <input> [-o output] [-a algorithm] [-l level]
//! - cszip decompress <input> [-o output]
//! - cszip info <input>
//! - cszip verify <input>

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// CsZip - Sistema de Compresión y Descompresión de Archivos
///
/// Herramienta de compresión sin pérdidas con formato personalizado .cz
#[derive(Parser, Debug)]
#[command(name = "cszip")]
#[command(author = "CsZip Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Sistema de compresión sin pérdidas con formato .cz")]
#[command(long_about = None)]
pub struct Cli {
    /// Subcomando a ejecutar
    #[command(subcommand)]
    pub command: Commands,

    /// Nivel de verbosidad (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Modo silencioso (sin salida excepto errores)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// Subcomandos disponibles
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Comprimir un archivo
    #[command(alias = "c")]
    Compress {
        /// Archivo de entrada a comprimir
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Archivo de salida (.cz)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Algoritmo de compresión
        #[arg(short, long, value_enum, default_value = "store")]
        algorithm: AlgorithmArg,

        /// Nivel de compresión (0-9)
        #[arg(short, long, default_value = "6", value_parser = clap::value_parser!(u8).range(0..=9))]
        level: u8,

        /// Forzar sobreescritura si el archivo existe
        #[arg(short, long)]
        force: bool,

        /// Usar CRC-64 en lugar de CRC-32
        #[arg(long)]
        crc64: bool,
    },

    /// Descomprimir un archivo
    #[command(alias = "d")]
    Decompress {
        /// Archivo .cz a descomprimir
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Archivo de salida
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Forzar sobreescritura si el archivo existe
        #[arg(short, long)]
        force: bool,

        /// No verificar checksums
        #[arg(long)]
        no_verify: bool,
    },

    /// Mostrar información de un archivo .cz
    #[command(alias = "i")]
    Info {
        /// Archivo .cz a inspeccionar
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Mostrar información detallada de bloques
        #[arg(short, long)]
        detailed: bool,
    },

    /// Verificar integridad de un archivo .cz
    #[command(alias = "v")]
    Verify {
        /// Archivo .cz a verificar
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// Listar contenido de un archivo .cz
    #[command(alias = "l")]
    List {
        /// Archivo .cz a listar
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },
}

/// Algoritmos de compresión disponibles
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmArg {
    /// Sin compresión (copia directa)
    Store,
    /// LZ77 con Huffman (no implementado aún)
    Lz77,
    /// LZ4 rápido (no implementado aún)
    Lz4,
    /// LZMA alta compresión (no implementado aún)
    Lzma,
    /// DEFLATE (no implementado aún)
    Deflate,
}

impl AlgorithmArg {
    /// Convierte a ID numérico
    pub fn to_id(self) -> u8 {
        match self {
            AlgorithmArg::Store => 0,
            AlgorithmArg::Lz77 => 1,
            AlgorithmArg::Lz4 => 2,
            AlgorithmArg::Lzma => 3,
            AlgorithmArg::Deflate => 4,
        }
    }

    /// Nombre legible
    pub fn name(self) -> &'static str {
        match self {
            AlgorithmArg::Store => "STORE",
            AlgorithmArg::Lz77 => "LZ77+Huffman",
            AlgorithmArg::Lz4 => "LZ4",
            AlgorithmArg::Lzma => "LZMA",
            AlgorithmArg::Deflate => "DEFLATE",
        }
    }

    /// Indica si está implementado
    pub fn is_implemented(self) -> bool {
        matches!(self, AlgorithmArg::Store | AlgorithmArg::Lz77)
    }
}

impl Cli {
    /// Obtiene el nivel de verbosidad efectivo
    pub fn verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else {
            match self.verbose {
                0 => Verbosity::Normal,
                1 => Verbosity::Verbose,
                2 => Verbosity::VeryVerbose,
                _ => Verbosity::Debug,
            }
        }
    }
}

/// Niveles de verbosidad
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    /// Sin salida excepto errores
    Quiet,
    /// Salida normal
    Normal,
    /// Salida detallada (-v)
    Verbose,
    /// Salida muy detallada (-vv)
    VeryVerbose,
    /// Salida de depuración (-vvv)
    Debug,
}

impl Verbosity {
    /// Indica si se debe mostrar información
    pub fn show_info(&self) -> bool {
        *self >= Verbosity::Normal
    }

    /// Indica si se debe mostrar detalles
    pub fn show_details(&self) -> bool {
        *self >= Verbosity::Verbose
    }

    /// Indica si se debe mostrar todo
    pub fn show_all(&self) -> bool {
        *self >= Verbosity::VeryVerbose
    }

    /// Indica si se debe mostrar depuración
    pub fn show_debug(&self) -> bool {
        *self >= Verbosity::Debug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parse_compress() {
        let cli = Cli::parse_from(["cszip", "compress", "test.txt"]);
        match cli.command {
            Commands::Compress { input, .. } => {
                assert_eq!(input, PathBuf::from("test.txt"));
            }
            _ => panic!("Expected Compress command"),
        }
    }

    #[test]
    fn test_cli_parse_compress_with_options() {
        let cli = Cli::parse_from([
            "cszip",
            "compress",
            "test.txt",
            "-o",
            "output.cz",
            "-a",
            "store",
            "-l",
            "9",
            "-f",
        ]);
        match cli.command {
            Commands::Compress {
                input,
                output,
                algorithm,
                level,
                force,
                ..
            } => {
                assert_eq!(input, PathBuf::from("test.txt"));
                assert_eq!(output, Some(PathBuf::from("output.cz")));
                assert_eq!(algorithm, AlgorithmArg::Store);
                assert_eq!(level, 9);
                assert!(force);
            }
            _ => panic!("Expected Compress command"),
        }
    }

    #[test]
    fn test_cli_parse_decompress() {
        let cli = Cli::parse_from(["cszip", "decompress", "test.cz", "-o", "output.txt"]);
        match cli.command {
            Commands::Decompress { input, output, .. } => {
                assert_eq!(input, PathBuf::from("test.cz"));
                assert_eq!(output, Some(PathBuf::from("output.txt")));
            }
            _ => panic!("Expected Decompress command"),
        }
    }

    #[test]
    fn test_cli_parse_info() {
        let cli = Cli::parse_from(["cszip", "info", "test.cz", "-d"]);
        match cli.command {
            Commands::Info { input, detailed } => {
                assert_eq!(input, PathBuf::from("test.cz"));
                assert!(detailed);
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn test_cli_parse_verify() {
        let cli = Cli::parse_from(["cszip", "verify", "test.cz"]);
        match cli.command {
            Commands::Verify { input } => {
                assert_eq!(input, PathBuf::from("test.cz"));
            }
            _ => panic!("Expected Verify command"),
        }
    }

    #[test]
    fn test_cli_aliases() {
        // Compress alias
        let cli = Cli::parse_from(["cszip", "c", "test.txt"]);
        assert!(matches!(cli.command, Commands::Compress { .. }));

        // Decompress alias
        let cli = Cli::parse_from(["cszip", "d", "test.cz"]);
        assert!(matches!(cli.command, Commands::Decompress { .. }));

        // Info alias
        let cli = Cli::parse_from(["cszip", "i", "test.cz"]);
        assert!(matches!(cli.command, Commands::Info { .. }));

        // Verify alias
        let cli = Cli::parse_from(["cszip", "v", "test.cz"]);
        assert!(matches!(cli.command, Commands::Verify { .. }));
    }

    #[test]
    fn test_verbosity() {
        let cli = Cli::parse_from(["cszip", "info", "test.cz"]);
        assert_eq!(cli.verbosity(), Verbosity::Normal);

        let cli = Cli::parse_from(["cszip", "-v", "info", "test.cz"]);
        assert_eq!(cli.verbosity(), Verbosity::Verbose);

        let cli = Cli::parse_from(["cszip", "-vv", "info", "test.cz"]);
        assert_eq!(cli.verbosity(), Verbosity::VeryVerbose);

        let cli = Cli::parse_from(["cszip", "-q", "info", "test.cz"]);
        assert_eq!(cli.verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn test_algorithm_conversion() {
        assert_eq!(AlgorithmArg::Store.to_id(), 0);
        assert_eq!(AlgorithmArg::Lz77.to_id(), 1);
        assert_eq!(AlgorithmArg::Lz4.to_id(), 2);
        assert_eq!(AlgorithmArg::Lzma.to_id(), 3);
        assert_eq!(AlgorithmArg::Deflate.to_id(), 4);
    }

    #[test]
    fn test_algorithm_implemented() {
        assert!(AlgorithmArg::Store.is_implemented());
        assert!(AlgorithmArg::Lz77.is_implemented());
        assert!(!AlgorithmArg::Lz4.is_implemented());
        assert!(!AlgorithmArg::Lzma.is_implemented());
        assert!(!AlgorithmArg::Deflate.is_implemented());
    }

    #[test]
    fn test_cli_help() {
        // Verificar que el help se genera correctamente
        Cli::command().debug_assert();
    }

    #[test]
    fn test_compression_level_range() {
        // Nivel válido
        let result = Cli::try_parse_from(["cszip", "compress", "test.txt", "-l", "5"]);
        assert!(result.is_ok());

        // Nivel máximo
        let result = Cli::try_parse_from(["cszip", "compress", "test.txt", "-l", "9"]);
        assert!(result.is_ok());

        // Nivel inválido
        let result = Cli::try_parse_from(["cszip", "compress", "test.txt", "-l", "10"]);
        assert!(result.is_err());
    }
}
