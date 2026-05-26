//! Benchmark de ratio de compresión
//!
//! Mide la efectividad de compresión para diferentes tipos de datos.
//!
//! ```bash
//! cargo bench --bench compression_ratio
//! ```

use std::io::Cursor;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use cszip::codec::{HuffmanEncoder, Lz77Compressor, Lz77Config};
use cszip::io::CzWriter;

/// Resultado de análisis de compresión
#[allow(dead_code)]
struct CompressionResult {
    original_size: usize,
    compressed_size: usize,
    algorithm: String,
}

impl CompressionResult {
    fn ratio(&self) -> f64 {
        self.compressed_size as f64 / self.original_size as f64 * 100.0
    }

    fn savings(&self) -> f64 {
        100.0 - self.ratio()
    }
}

/// Generar diferentes patrones de datos
fn generate_pattern(pattern: &str, size: usize) -> Vec<u8> {
    match pattern {
        "random" => {
            // Datos pseudo-aleatorios (deterministicos para reproducibilidad)
            (0..size)
                .map(|i| {
                    let x = i.wrapping_mul(1103515245).wrapping_add(12345);
                    (x >> 16) as u8
                })
                .collect()
        }
        "zeros" => vec![0u8; size],
        "ones" => vec![0xFF; size],
        "sequential" => (0..size).map(|i| (i % 256) as u8).collect(),
        "text_english" => {
            let text = b"The quick brown fox jumps over the lazy dog. ";
            text.iter().cycle().take(size).copied().collect()
        }
        "text_lorem" => {
            let text = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ";
            text.iter().cycle().take(size).copied().collect()
        }
        "html" => {
            let html =
                b"<html><body><div class=\"container\"><p>Content here</p></div></body></html>";
            html.iter().cycle().take(size).copied().collect()
        }
        "json" => {
            let json = b"{\"name\":\"value\",\"array\":[1,2,3],\"nested\":{\"key\":\"val\"}}";
            json.iter().cycle().take(size).copied().collect()
        }
        "binary_mixed" => {
            // Mezcla de patrones
            let mut data = Vec::with_capacity(size);
            for i in 0..size {
                data.push(match (i / 100) % 4 {
                    0 => 0,
                    1 => (i % 256) as u8,
                    2 => 0xFF,
                    _ => ((i * 7) % 256) as u8,
                });
            }
            data
        }
        "repetitive_short" => {
            // Patrón corto repetido
            let pattern = b"AB";
            pattern.iter().cycle().take(size).copied().collect()
        }
        "repetitive_long" => {
            // Patrón largo repetido
            let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            pattern.iter().cycle().take(size).copied().collect()
        }
        _ => vec![0u8; size],
    }
}

fn bench_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratio");

    // No medir tiempo, solo ratio
    group.sample_size(10);

    let patterns = [
        "random",
        "zeros",
        "sequential",
        "text_english",
        "text_lorem",
        "html",
        "json",
        "binary_mixed",
        "repetitive_short",
        "repetitive_long",
    ];

    let sizes = [1024, 16384, 65536];

    println!("\n=== Análisis de Ratio de Compresión ===\n");
    println!(
        "{:<20} {:>10} {:>12} {:>10} {:>10}",
        "Patrón", "Tamaño", "Comprimido", "Ratio", "Ahorro"
    );
    println!("{}", "-".repeat(65));

    for pattern in patterns {
        for size in sizes {
            let data = generate_pattern(pattern, size);

            // Comprimir con CzWriter (STORE)
            let mut compressed = Vec::new();
            {
                let cursor = Cursor::new(&mut compressed);
                let mut writer = CzWriter::new(cursor).unwrap();
                writer.write_block(&data).unwrap();
                writer.finish().unwrap();
            }

            let result = CompressionResult {
                original_size: data.len(),
                compressed_size: compressed.len(),
                algorithm: "STORE".to_string(),
            };

            println!(
                "{:<20} {:>10} {:>12} {:>9.1}% {:>9.1}%",
                format!("{}/{}", pattern, size),
                result.original_size,
                result.compressed_size,
                result.ratio(),
                result.savings()
            );

            // Benchmark (para que criterion registre algo)
            group.bench_with_input(BenchmarkId::new(pattern, size), &data, |b, data| {
                b.iter(|| {
                    let mut compressed = Vec::new();
                    let cursor = Cursor::new(&mut compressed);
                    let mut writer = CzWriter::new(cursor).unwrap();
                    writer.write_block(data).unwrap();
                    writer.finish().unwrap();
                    compressed.len()
                });
            });
        }
    }

    println!();
    group.finish();
}

fn bench_lz77_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("lz77_ratio");
    group.sample_size(10);

    let patterns = ["text_english", "repetitive_short", "random"];
    let size = 16384;

    println!("\n=== Análisis LZ77 ===\n");
    println!(
        "{:<20} {:>10} {:>12} {:>10}",
        "Patrón", "Original", "LZ77", "Ratio"
    );
    println!("{}", "-".repeat(55));

    for pattern in patterns {
        let data = generate_pattern(pattern, size);

        let compressor = Lz77Compressor::with_config(Lz77Config::for_level(6));
        let compressed = compressor.compress_to_bytes(&data);

        println!(
            "{:<20} {:>10} {:>12} {:>9.1}%",
            pattern,
            data.len(),
            compressed.len(),
            compressed.len() as f64 / data.len() as f64 * 100.0
        );

        group.bench_with_input(BenchmarkId::new(pattern, size), &data, |b, data| {
            let compressor = Lz77Compressor::with_config(Lz77Config::for_level(6));
            b.iter(|| compressor.compress_to_bytes(data));
        });
    }

    println!();
    group.finish();
}

fn bench_huffman_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("huffman_ratio");
    group.sample_size(10);

    let patterns = ["text_english", "zeros", "random"];
    let size = 16384;

    println!("\n=== Análisis Huffman ===\n");
    println!(
        "{:<20} {:>10} {:>12} {:>10}",
        "Patrón", "Original", "Huffman", "Ratio"
    );
    println!("{}", "-".repeat(55));

    for pattern in patterns {
        let data = generate_pattern(pattern, size);

        let mut encoder = HuffmanEncoder::new();
        let compressed = encoder.encode(&data).unwrap();

        println!(
            "{:<20} {:>10} {:>12} {:>9.1}%",
            pattern,
            data.len(),
            compressed.len(),
            compressed.len() as f64 / data.len() as f64 * 100.0
        );

        group.bench_with_input(BenchmarkId::new(pattern, size), &data, |b, data| {
            b.iter(|| {
                let mut encoder = HuffmanEncoder::new();
                encoder.encode(data).unwrap()
            });
        });
    }

    println!();
    group.finish();
}

criterion_group!(
    benches,
    bench_compression_ratio,
    bench_lz77_ratio,
    bench_huffman_ratio,
);

criterion_main!(benches);
