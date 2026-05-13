//! Ejemplo básico de compresión
//!
//! Muestra cómo comprimir un archivo usando la API de CsZip.
//!
//! ```bash
//! cargo run --example basic_compress -- input.txt output.cz
//! ```

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use cszip::io::CzWriter;
use cszip::codec::{Algorithm, CompressionLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_entrada> [archivo_salida]", args[0]);
        eprintln!();
        eprintln!("Ejemplo:");
        eprintln!("  {} datos.txt datos.cz", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        format!("{}.cz", args[1])
    };

    // Verificar que el archivo existe
    if !input_path.exists() {
        eprintln!("Error: El archivo '{}' no existe", input_path.display());
        std::process::exit(1);
    }

    println!("Comprimiendo: {} -> {}", input_path.display(), output_path);

    // Abrir archivo de entrada
    let input_file = File::open(input_path)?;
    let input_size = input_file.metadata()?.len();
    let mut reader = BufReader::new(input_file);

    // Crear archivo de salida con CzWriter
    let mut writer = CzWriter::create_with_options(
        &output_path,
        Algorithm::Store,
        CompressionLevel::default(),
    )?;

    // Comprimir usando stream
    writer.write_stream(&mut reader)?;

    // Finalizar y obtener estadísticas
    let stats = writer.finish()?;

    // Mostrar resultados
    println!();
    println!("Compresión completada:");
    println!("  Tamaño original:   {} bytes", input_size);
    println!("  Tamaño comprimido: {} bytes", stats.compressed_size);
    println!("  Bloques:           {}", stats.block_count);
    println!("  Ratio:             {:.1}%", stats.ratio() * 100.0);

    Ok(())
}
