//! Ejemplo básico de descompresión
//!
//! Muestra cómo descomprimir un archivo .cz usando la API de CsZip.
//!
//! ```bash
//! cargo run --example basic_decompress -- archivo.cz [salida]
//! ```

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use cszip::io::CzReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Uso: {} <archivo.cz> [archivo_salida]", args[0]);
        eprintln!();
        eprintln!("Ejemplo:");
        eprintln!("  {} datos.cz datos.txt", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    // Determinar archivo de salida
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        // Quitar extensión .cz
        let input_str = input_path.to_string_lossy();
        if let Some(stripped) = input_str.strip_suffix(".cz") {
            stripped.to_string()
        } else {
            format!("{}.out", input_str)
        }
    };

    // Verificar que el archivo existe
    if !input_path.exists() {
        eprintln!("Error: El archivo '{}' no existe", input_path.display());
        std::process::exit(1);
    }

    println!(
        "Descomprimiendo: {} -> {}",
        input_path.display(),
        output_path
    );

    // Abrir archivo comprimido
    let mut reader = CzReader::open(input_path)?;

    // Mostrar información del archivo
    let header = reader.header();
    let algorithm = cszip::codec::Algorithm::from_id(header.compression_algo)?;
    println!();
    println!("Información del archivo:");
    println!(
        "  Versión:    {}.{}",
        header.version_major, header.version_minor
    );
    println!("  Algoritmo:  {}", algorithm.name());
    println!("  Tamaño de bloque: {} bytes", header.block_size());

    // Crear archivo de salida
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Descomprimir bloque por bloque
    let mut total_bytes = 0u64;
    let mut block_count = 0u32;

    while let Some(block) = reader.read_block()? {
        writer.write_all(&block.data)?;
        total_bytes += block.data.len() as u64;
        block_count += 1;
    }

    writer.flush()?;

    // Mostrar resultados
    println!();
    println!("Descompresión completada:");
    println!("  Bytes descomprimidos: {}", total_bytes);
    println!("  Bloques procesados:   {}", block_count);

    Ok(())
}
