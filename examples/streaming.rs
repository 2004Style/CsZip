//! Ejemplo de compresión/descompresión en streaming
//!
//! Muestra cómo procesar archivos grandes sin cargar todo en memoria.
//!
//! ```bash
//! cargo run --example streaming -- compress input.txt output.cz
//! cargo run --example streaming -- decompress output.cz restored.txt
//! ```

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};

use cszip::io::{StreamOptions, StreamingCompressor, StreamingDecompressor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Uso: {} <compress|decompress> <entrada> <salida>", args[0]);
        eprintln!();
        eprintln!("Ejemplos:");
        eprintln!("  {} compress datos.txt datos.cz", args[0]);
        eprintln!("  {} decompress datos.cz datos.txt", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match command.as_str() {
        "compress" => compress_streaming(input_path, output_path)?,
        "decompress" => decompress_streaming(input_path, output_path)?,
        _ => {
            eprintln!("Comando desconocido: {}", command);
            eprintln!("Use 'compress' o 'decompress'");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn compress_streaming(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Comprimiendo en modo streaming: {} -> {}", input, output);

    // Configurar opciones
    let options = StreamOptions::default().with_block_size(32 * 1024); // 32KB por bloque

    // Abrir archivos
    let input_file = File::open(input)?;
    let input_size = input_file.metadata()?.len();
    let mut reader = BufReader::new(input_file);

    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);

    // Crear compresor de streaming con callback de progreso
    let mut compressor =
        StreamingCompressor::new(writer, options)?.with_progress(Box::new(move |progress| {
            if let Some(pct) = progress.percentage() {
                eprint!(
                    "\rProgreso: {:.1}% - {} bloques",
                    pct, progress.blocks_processed
                );
            } else {
                eprint!(
                    "\rProcesados: {} bytes - {} bloques",
                    progress.bytes_processed, progress.blocks_processed
                );
            }
        }));

    // El total no se conoce a priori en streaming puro, pero lo tenemos
    // así que podemos simular conocerlo

    // Comprimir
    compressor.compress_stream(&mut reader)?;
    let stats = compressor.finish()?;

    eprintln!(); // Nueva línea después del progreso
    println!();
    println!("Compresión completada:");
    println!("  Original:   {} bytes", input_size);
    println!("  Comprimido: {} bytes", stats.compressed_size);
    println!("  Bloques:    {}", stats.block_count);
    println!("  Ratio:      {:.1}%", stats.ratio());
    println!("  Ahorro:     {:.1}%", stats.savings());

    Ok(())
}

fn decompress_streaming(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Descomprimiendo en modo streaming: {} -> {}", input, output);

    let options = StreamOptions::default();

    // Abrir archivos
    let input_file = File::open(input)?;
    let reader = BufReader::new(input_file);

    let output_file = File::create(output)?;
    let mut writer = BufWriter::new(output_file);

    // Crear descompresor de streaming
    let mut decompressor =
        StreamingDecompressor::new(reader, options)?.with_progress(Box::new(|progress| {
            eprint!("\rProcesados: {} bloques", progress.blocks_processed);
        }));

    // Descomprimir
    let stats = decompressor.decompress_stream(&mut writer)?;

    eprintln!(); // Nueva línea
    println!();
    println!("Descompresión completada:");
    println!("  Tamaño restaurado: {} bytes", stats.original_size);
    println!("  Bloques:           {}", stats.block_count);
    println!("  CRC-32:            0x{:08X}", stats.crc32);

    Ok(())
}

/// Ejemplo de compresión en memoria (sin archivos)
#[allow(dead_code)]
fn memory_streaming_example() -> Result<(), cszip::Error> {
    let data = b"Este es un ejemplo de datos que se comprimiran en memoria \
                 usando streaming. Repite: datos datos datos datos.";

    let options = StreamOptions::default();

    // Comprimir a buffer en memoria
    let mut compressed = Vec::new();
    {
        let cursor = Cursor::new(&mut compressed);
        let mut compressor = StreamingCompressor::new(cursor, options.clone())?;
        let mut input = Cursor::new(data.as_slice());
        compressor.compress_stream(&mut input)?;
        compressor.finish()?;
    }

    println!("Original: {} bytes", data.len());
    println!("Comprimido: {} bytes", compressed.len());

    // Descomprimir
    let mut decompressed = Vec::new();
    {
        let cursor = Cursor::new(&compressed);
        let mut decompressor = StreamingDecompressor::new(cursor, options)?;
        decompressor.decompress_stream(&mut decompressed)?;
    }

    assert_eq!(decompressed, data);
    println!("Roundtrip exitoso!");

    Ok(())
}
