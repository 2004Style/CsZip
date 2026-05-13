//! Implementación de comandos CLI
//!
//! Contiene la lógica de ejecución para cada subcomando.

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::codec::{Algorithm, CompressionLevel};
use crate::error::{Error, ErrorKind};
use crate::io::{CzReader, CzWriter};

use super::args::{AlgorithmArg, Verbosity};

/// Ejecuta el comando de compresión
pub fn compress(
    input: &Path,
    output: Option<&Path>,
    algorithm: AlgorithmArg,
    level: u8,
    force: bool,
    _crc64: bool,
    verbosity: Verbosity,
) -> Result<(), Error> {
    // Verificar que el algoritmo está implementado
    if !algorithm.is_implemented() {
        return Err(Error::new(
            ErrorKind::UnsupportedAlgorithm,
            format!(
                "El algoritmo {} no está implementado aún",
                algorithm.name()
            ),
        ));
    }

    // Verificar que el archivo de entrada existe
    if !input.exists() {
        return Err(Error::new(
            ErrorKind::FileNotFound,
            format!("Archivo no encontrado: {}", input.display()),
        ));
    }

    // Determinar archivo de salida
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            // Añadir .cz a la ruta completa (preservando extensión original)
            let mut p = input.to_path_buf().into_os_string();
            p.push(".cz");
            PathBuf::from(p)
        }
    };

    // Verificar si el archivo de salida existe
    if output_path.exists() && !force {
        return Err(Error::new(
            ErrorKind::FileExists,
            format!(
                "El archivo ya existe: {}. Use -f para sobreescribir.",
                output_path.display()
            ),
        ));
    }

    if verbosity.show_info() {
        println!("Comprimiendo: {} -> {}", input.display(), output_path.display());
        println!("Algoritmo: {}, Nivel: {}", algorithm.name(), level);
    }

    let start = Instant::now();

    // Abrir archivos
    let input_file = File::open(input).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error abriendo archivo de entrada: {}", e),
        )
    })?;

    let input_size = input_file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::new(input_file);

    let output_file = File::create(&output_path).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error creando archivo de salida: {}", e),
        )
    })?;

    let writer = BufWriter::new(output_file);

    // Crear escritor CsZip
    let alg = Algorithm::from_id(algorithm.to_id())?;
    let lvl = CompressionLevel::new(level)?;
    let mut cz_writer = CzWriter::new_with_options(writer, alg, lvl)?;

    // Comprimir
    cz_writer.write_stream(&mut reader)?;
    let stats = cz_writer.finish()?;

    let elapsed = start.elapsed();

    if verbosity.show_info() {
        println!("\nResultado:");
        println!("  Tamaño original:  {} bytes", input_size);
        println!("  Tamaño comprimido: {} bytes", stats.compressed_size);
        println!("  Ratio: {:.2}%", stats.ratio() * 100.0);
        println!("  Bloques: {}", stats.block_count);
        println!("  CRC-32: 0x{:08X}", stats.global_crc32);
        println!("  Tiempo: {:.2?}", elapsed);
    }

    Ok(())
}

/// Ejecuta el comando de descompresión
pub fn decompress(
    input: &Path,
    output: Option<&Path>,
    force: bool,
    no_verify: bool,
    verbosity: Verbosity,
) -> Result<(), Error> {
    // Verificar que el archivo de entrada existe
    if !input.exists() {
        return Err(Error::new(
            ErrorKind::FileNotFound,
            format!("Archivo no encontrado: {}", input.display()),
        ));
    }

    // Verificar extensión
    if input.extension().and_then(|e| e.to_str()) != Some("cz") {
        if verbosity.show_info() {
            eprintln!("Advertencia: El archivo no tiene extensión .cz");
        }
    }

    // Determinar archivo de salida
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let mut p = input.to_path_buf();
            // Remover .cz y restaurar extensión original o usar .out
            p.set_extension("");
            if p.extension().is_none() {
                p.set_extension("out");
            }
            p
        }
    };

    // Verificar si el archivo de salida existe
    if output_path.exists() && !force {
        return Err(Error::new(
            ErrorKind::FileExists,
            format!(
                "El archivo ya existe: {}. Use -f para sobreescribir.",
                output_path.display()
            ),
        ));
    }

    if verbosity.show_info() {
        println!(
            "Descomprimiendo: {} -> {}",
            input.display(),
            output_path.display()
        );
    }

    let start = Instant::now();

    // Abrir archivo de entrada
    let mut reader = CzReader::open(input)?;

    if no_verify {
        reader = reader.with_checksum_verification(false);
    }

    if verbosity.show_details() {
        let alg = reader.algorithm()?;
        println!("Algoritmo: {}", alg.name());
        println!("Verificación: {}", if no_verify { "desactivada" } else { "activada" });
    }

    // Crear archivo de salida
    let output_file = File::create(&output_path).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error creando archivo de salida: {}", e),
        )
    })?;

    let mut writer = BufWriter::new(output_file);

    // Descomprimir
    let stats = reader.decompress_all(&mut writer)?;

    writer.flush().map_err(|e| {
        Error::new(ErrorKind::IoError, format!("Error en flush: {}", e))
    })?;

    let elapsed = start.elapsed();

    if verbosity.show_info() {
        println!("\nResultado:");
        println!("  Tamaño descomprimido: {} bytes", stats.original_size);
        println!("  Bloques procesados: {}", stats.block_count);
        println!("  CRC-32: 0x{:08X}", stats.global_crc32);
        println!("  Tiempo: {:.2?}", elapsed);
    }

    Ok(())
}

/// Ejecuta el comando de información
pub fn info(input: &Path, detailed: bool, _verbosity: Verbosity) -> Result<(), Error> {
    // Verificar que el archivo existe
    if !input.exists() {
        return Err(Error::new(
            ErrorKind::FileNotFound,
            format!("Archivo no encontrado: {}", input.display()),
        ));
    }

    let mut reader = CzReader::open(input)?;
    let header = reader.header();

    println!("Información de archivo: {}", input.display());
    println!("{}", "-".repeat(50));
    println!("Version:     {}.{}", header.version_major, header.version_minor);

    let algorithm = Algorithm::from_id(header.compression_algo)?;
    println!("Algoritmo:   {} ({})", algorithm.name(), header.compression_algo);
    println!("Tamaño bloque: {} bytes", header.block_size());
    println!("CRC:         {}", if header.uses_crc64() { "CRC-64" } else { "CRC-32" });

    // Leer footer para más información
    if let Ok(footer) = reader.read_footer() {
        println!("\nEstadisticas del archivo:");
        println!("  Bloques:    {}", footer.num_blocks);
        println!("  Tamano original: {} bytes", footer.total_raw_size);
        println!("  Checksum global: 0x{:08X}", footer.checksum);
    }

    if detailed {
        println!("\nBloques:");
        reader.rewind()?;
        let mut block_num = 0;
        while let Some(block) = reader.read_block()? {
            println!(
                "  [{}] Original: {} bytes, Comprimido: {} bytes, CRC: 0x{:08X}",
                block_num,
                block.original_size,
                block.compressed_size,
                block.crc32
            );
            block_num += 1;
        }
    }

    Ok(())
}

/// Ejecuta el comando de verificación
pub fn verify(input: &Path, verbosity: Verbosity) -> Result<(), Error> {
    // Verificar que el archivo existe
    if !input.exists() {
        return Err(Error::new(
            ErrorKind::FileNotFound,
            format!("Archivo no encontrado: {}", input.display()),
        ));
    }

    if verbosity.show_info() {
        println!("Verificando: {}", input.display());
    }

    let start = Instant::now();

    let mut reader = CzReader::open(input)?;

    // Leer footer primero
    let footer = reader.read_footer()?;
    let expected_blocks = footer.num_blocks;
    let expected_crc = footer.checksum;

    if verbosity.show_details() {
        println!("Esperando {} bloques", expected_blocks);
        println!("CRC-32 esperado: 0x{:08X}", expected_crc);
    }

    // Verificar cada bloque
    reader.rewind()?;

    let mut verified_blocks = 0u32;
    let mut global_crc = crate::format::checksum::Crc32::new();

    while let Some(block) = reader.read_block()? {
        global_crc.update(&block.data);
        verified_blocks += 1;

        if verbosity.show_all() {
            println!(
                "  Bloque {} OK: {} bytes, CRC: 0x{:08X}",
                block.index, block.original_size, block.crc32
            );
        }
    }

    let calculated_crc = global_crc.finalize();
    let elapsed = start.elapsed();

    // Verificar resultados
    if verified_blocks != expected_blocks {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Número de bloques incorrecto: esperado {}, encontrado {}",
                expected_blocks, verified_blocks
            ),
        ));
    }

    if calculated_crc != expected_crc {
        return Err(Error::new(
            ErrorKind::ChecksumMismatch,
            format!(
                "CRC-32 global no coincide: esperado 0x{:08X}, calculado 0x{:08X}",
                expected_crc, calculated_crc
            ),
        ));
    }

    if verbosity.show_info() {
        println!("\n✓ Archivo válido");
        println!("  Bloques verificados: {}", verified_blocks);
        println!("  CRC-32: 0x{:08X}", calculated_crc);
        println!("  Tiempo: {:.2?}", elapsed);
    }

    Ok(())
}

/// Ejecuta el comando de listar
pub fn list(input: &Path, verbosity: Verbosity) -> Result<(), Error> {
    // Verificar que el archivo existe
    if !input.exists() {
        return Err(Error::new(
            ErrorKind::FileNotFound,
            format!("Archivo no encontrado: {}", input.display()),
        ));
    }

    let mut reader = CzReader::open(input)?;

    // Leer footer
    if let Ok(footer) = reader.read_footer() {
        println!("Archivo: {}", input.display());
        println!("Bloques: {}", footer.num_blocks);
        println!("Tamano original: {} bytes", footer.total_raw_size);

        if verbosity.show_details() {
            println!("\nDetalle de bloques:");
            println!("{:>5} {:>12} {:>12} {:>8}", "Bloque", "Original", "Comprimido", "Ratio");
            println!("{}", "-".repeat(45));

            reader.rewind()?;
            while let Some(block) = reader.read_block()? {
                let ratio = if block.original_size > 0 {
                    block.compressed_size as f64 / block.original_size as f64 * 100.0
                } else {
                    100.0
                };
                println!(
                    "{:>5} {:>12} {:>12} {:>7.1}%",
                    block.index, block.original_size, block.compressed_size, ratio
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compress_nonexistent_file() {
        let result = compress(
            Path::new("nonexistent_file.txt"),
            None,
            AlgorithmArg::Store,
            6,
            false,
            false,
            Verbosity::Quiet,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::FileNotFound);
    }

    #[test]
    fn test_compress_unsupported_algorithm() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"test data").unwrap();

        let result = compress(
            temp.path(),
            None,
            AlgorithmArg::Lzma, // No implementado
            6,
            false,
            false,
            Verbosity::Quiet,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnsupportedAlgorithm);
    }

    #[test]
    fn test_decompress_nonexistent_file() {
        let result = decompress(
            Path::new("nonexistent_file.cz"),
            None,
            false,
            false,
            Verbosity::Quiet,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::FileNotFound);
    }

    #[test]
    fn test_info_nonexistent_file() {
        let result = info(Path::new("nonexistent_file.cz"), false, Verbosity::Quiet);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::FileNotFound);
    }

    #[test]
    fn test_verify_nonexistent_file() {
        let result = verify(Path::new("nonexistent_file.cz"), Verbosity::Quiet);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::FileNotFound);
    }
}
