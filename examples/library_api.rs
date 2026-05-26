//! Ejemplo de uso de la API de biblioteca
//!
//! Muestra las diferentes formas de usar CsZip como biblioteca.
//!
//! ```bash
//! cargo run --example library_api
//! ```

use std::io::Cursor;

use cszip::codec::{
    Algorithm, CompressionLevel, HuffmanDecoder, HuffmanEncoder, Lz77Compressor, Lz77Config,
    Lz77Decompressor,
};
use cszip::format::checksum::{Adler32, Crc32, Crc64};
use cszip::format::{BlockHeader, FileFooter, Header};
use cszip::io::{CzReader, CzWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ejemplos de API de CsZip ===\n");

    example_basic_compression()?;
    example_checksums();
    example_lz77();
    example_huffman()?;
    example_format_structures()?;
    example_full_roundtrip()?;

    println!("\n=== Todos los ejemplos completados ===");
    Ok(())
}

/// Ejemplo 1: Compresión básica con CzWriter/CzReader
fn example_basic_compression() -> Result<(), cszip::Error> {
    println!("--- Ejemplo 1: Compresión Básica ---");

    let original_data = b"Hello, CsZip! Este es un ejemplo de compresion basica.";

    // Comprimir a buffer en memoria
    let mut compressed = Vec::new();
    {
        let cursor = Cursor::new(&mut compressed);
        let mut writer = CzWriter::new(cursor)?;

        // Escribir datos como un bloque
        writer.write_block(original_data)?;

        // Finalizar (escribe footer)
        let stats = writer.finish()?;
        println!("  Estadísticas de compresión:");
        println!("    Bloques: {}", stats.block_count);
        println!("    Original: {} bytes", stats.original_size);
        println!("    Comprimido: {} bytes", stats.compressed_size);
    }

    // Descomprimir
    let cursor = Cursor::new(&compressed);
    let mut reader = CzReader::new(cursor)?;

    // Leer bloque
    if let Some(block) = reader.read_block()? {
        println!("  Datos recuperados: {} bytes", block.data.len());
        assert_eq!(block.data.as_slice(), original_data);
        println!("  ✓ Datos idénticos al original");
    }

    println!();
    Ok(())
}

/// Ejemplo 2: Uso de checksums
fn example_checksums() {
    println!("--- Ejemplo 2: Checksums ---");

    let data = b"Datos para calcular checksum";

    // CRC-32
    let crc32 = Crc32::compute(data);
    println!("  CRC-32:   0x{:08X}", crc32);

    // CRC-32 incremental
    let mut crc = Crc32::new();
    crc.update(b"Datos ");
    crc.update(b"para calcular ");
    crc.update(b"checksum");
    let crc32_inc = crc.finalize();
    println!("  CRC-32 (inc): 0x{:08X}", crc32_inc);
    assert_eq!(crc32, crc32_inc);

    // CRC-64
    let crc64 = Crc64::compute(data);
    println!("  CRC-64:   0x{:016X}", crc64);

    // ADLER-32
    let adler = Adler32::compute(data);
    println!("  ADLER-32: 0x{:08X}", adler);

    // Verificación
    let is_valid = Crc32::verify(data, crc32);
    println!(
        "  Verificación CRC-32: {}",
        if is_valid { "✓ OK" } else { "✗ Fallo" }
    );

    println!();
}

/// Ejemplo 3: Uso directo de LZ77
fn example_lz77() {
    println!("--- Ejemplo 3: Algoritmo LZ77 ---");

    // Datos con repeticiones (ideales para LZ77)
    let data = b"abracadabra abracadabra magic magic magic";

    // Configurar compresor
    let config = Lz77Config::for_level(6);
    let compressor = Lz77Compressor::with_config(config);

    // Comprimir a tokens
    let tokens = compressor.compress(data);
    println!("  Datos originales: {} bytes", data.len());
    println!("  Tokens generados: {}", tokens.len());

    // Contar literales vs matches
    let literals = tokens.iter().filter(|t| t.is_literal()).count();
    let matches = tokens.iter().filter(|t| t.is_match()).count();
    println!("  Literales: {}, Matches: {}", literals, matches);

    // Comprimir a bytes
    let compressed = compressor.compress_to_bytes(data);
    println!("  Tamaño comprimido: {} bytes", compressed.len());

    // Descomprimir
    let decompressed = Lz77Decompressor::decompress(&compressed).unwrap();
    assert_eq!(decompressed.as_slice(), data);
    println!("  ✓ Roundtrip LZ77 exitoso");

    println!();
}

/// Ejemplo 4: Codificación Huffman
fn example_huffman() -> Result<(), cszip::Error> {
    println!("--- Ejemplo 4: Codificación Huffman ---");

    // Datos con frecuencias desiguales (buenos para Huffman)
    let data = b"aaaaaabbbbcccdde";

    let mut encoder = HuffmanEncoder::new();
    let encoded = encoder.encode(data)?;

    println!("  Original: {} bytes", data.len());
    println!("  Codificado: {} bytes", encoded.len());

    // Decodificar
    let decoded = HuffmanDecoder::decode(&encoded)?;
    assert_eq!(decoded.as_slice(), data);
    println!("  ✓ Roundtrip Huffman exitoso");

    println!();
    Ok(())
}

/// Ejemplo 5: Estructuras del formato
fn example_format_structures() -> Result<(), cszip::Error> {
    println!("--- Ejemplo 5: Estructuras del Formato ---");

    // Crear header
    let header = Header::new(
        0,    // STORE algorithm
        15,   // 32KB blocks (2^15)
        1000, // 10x max expansion
    )?;

    println!("  Header:");
    println!("    Magic: 0x{:04X}", header.magic);
    println!(
        "    Versión: {}.{}",
        header.version_major, header.version_minor
    );
    println!("    Algoritmo: {}", header.compression_algo);
    println!("    Tamaño de bloque: {} bytes", header.block_size());

    // Serializar y deserializar
    let bytes = header.to_bytes();
    let restored = Header::from_bytes(&bytes)?;
    assert_eq!(header, restored);
    println!("    ✓ Serialización header OK");

    // Crear block header
    let block = BlockHeader::new(
        1024,       // original size
        1024,       // compressed size (STORE)
        0x12345678, // adler32
        0,          // compression level
    )?;

    println!("  Block Header:");
    println!("    Tipo: {}", block.block_type);
    println!("    Original: {} bytes", block.original_size);
    println!("    Comprimido: {} bytes", block.compressed_size);

    // Crear footer
    let footer = FileFooter::new(5, 10240)?;
    println!("  Footer:");
    println!("    Bloques: {}", footer.num_blocks);
    println!("    Tamaño total: {} bytes", footer.total_raw_size);

    println!();
    Ok(())
}

/// Ejemplo 6: Roundtrip completo
fn example_full_roundtrip() -> Result<(), cszip::Error> {
    println!("--- Ejemplo 6: Roundtrip Completo ---");

    // Crear datos de prueba variados
    let mut data = Vec::new();

    // Texto
    data.extend_from_slice(b"Este es texto normal. ");

    // Datos repetitivos
    data.extend(std::iter::repeat(b'X').take(100));

    // Secuencia numérica
    data.extend((0u8..=255).cycle().take(256));

    // Más texto
    data.extend_from_slice(b" Fin de los datos de prueba.");

    println!("  Datos originales: {} bytes", data.len());

    // Comprimir con diferentes configuraciones
    for algo_name in ["STORE"] {
        let mut compressed = Vec::new();
        {
            let cursor = Cursor::new(&mut compressed);
            let mut writer =
                CzWriter::new_with_options(cursor, Algorithm::Store, CompressionLevel::new(5)?)?;

            // Escribir en múltiples bloques pequeños
            for chunk in data.chunks(100) {
                writer.write_block(chunk)?;
            }

            let stats = writer.finish()?;
            println!(
                "  {} - Comprimido: {} bytes, {} bloques",
                algo_name, stats.compressed_size, stats.block_count
            );
        }

        // Descomprimir
        let mut decompressed: Vec<u8> = Vec::new();
        {
            let cursor = Cursor::new(&compressed);
            let mut reader = CzReader::new(cursor)?;

            while let Some(block) = reader.read_block()? {
                decompressed.extend(&block.data);
            }
        }

        assert_eq!(decompressed, data);
        println!("  ✓ Roundtrip {} exitoso", algo_name);
    }

    println!();
    Ok(())
}
