//! Benchmark de throughput de descompresión
//!
//! Mide la velocidad de descompresión en bytes/segundo.
//!
//! ```bash
//! cargo bench --bench decompress_throughput
//! ```

use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use cszip::io::{CzReader, CzWriter};

/// Generar datos de prueba y comprimirlos
fn generate_compressed_data(size: usize, pattern: &str) -> (Vec<u8>, Vec<u8>) {
    let original: Vec<u8> = match pattern {
        "random" => (0..size).map(|i| (i * 17 + 31) as u8).collect(),
        "zeros" => vec![0u8; size],
        "sequential" => (0..size).map(|i| (i % 256) as u8).collect(),
        "text" => {
            let text = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
            text.iter().cycle().take(size).copied().collect()
        }
        "repetitive" => {
            let pattern = b"ABABABABAB";
            pattern.iter().cycle().take(size).copied().collect()
        }
        _ => vec![0u8; size],
    };

    let mut compressed = Vec::new();
    {
        let cursor = Cursor::new(&mut compressed);
        let mut writer = CzWriter::new(cursor).unwrap();
        writer.write_block(&original).unwrap();
        writer.finish().unwrap();
    }

    (original, compressed)
}

fn bench_decompress_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress_store");

    let sizes = [1024, 4096, 16384, 65536, 262144];

    for size in sizes {
        let (original, compressed) = generate_compressed_data(size, "text");

        group.throughput(Throughput::Bytes(original.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("text", size),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(compressed.as_slice()));
                    let mut reader = CzReader::new(cursor).unwrap();
                    let mut output: Vec<u8> = Vec::new();

                    while let Some(block) = reader.read_block().unwrap() {
                        output.extend(&block.data);
                    }

                    output
                });
            },
        );
    }

    group.finish();
}

fn bench_decompress_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress_patterns");

    let size = 65536;
    let patterns = ["random", "zeros", "sequential", "text", "repetitive"];

    for pattern in patterns {
        let (original, compressed) = generate_compressed_data(size, pattern);

        group.throughput(Throughput::Bytes(original.len() as u64));

        group.bench_with_input(
            BenchmarkId::new(pattern, size),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(compressed.as_slice()));
                    let mut reader = CzReader::new(cursor).unwrap();
                    let mut output: Vec<u8> = Vec::new();

                    while let Some(block) = reader.read_block().unwrap() {
                        output.extend(&block.data);
                    }

                    output
                });
            },
        );
    }

    group.finish();
}

fn bench_decompress_with_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress_verification");

    let size = 65536;
    let (_, compressed) = generate_compressed_data(size, "text");

    group.throughput(Throughput::Bytes(size as u64));

    group.bench_with_input(
        BenchmarkId::new("with_verify", size),
        &compressed,
        |b, compressed| {
            b.iter(|| {
                let cursor = Cursor::new(black_box(compressed.as_slice()));
                let mut reader = CzReader::new(cursor)
                    .unwrap()
                    .with_checksum_verification(true);
                let mut output: Vec<u8> = Vec::new();

                while let Some(block) = reader.read_block().unwrap() {
                    output.extend(&block.data);
                }

                output
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("without_verify", size),
        &compressed,
        |b, compressed| {
            b.iter(|| {
                let cursor = Cursor::new(black_box(compressed.as_slice()));
                let mut reader = CzReader::new(cursor)
                    .unwrap()
                    .with_checksum_verification(false);
                let mut output: Vec<u8> = Vec::new();

                while let Some(block) = reader.read_block().unwrap() {
                    output.extend(&block.data);
                }

                output
            });
        },
    );

    group.finish();
}

fn bench_decompress_multiple_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress_multi_block");

    let total_size = 262144;
    let block_sizes = [1024, 4096, 16384, 32768];

    for block_size in block_sizes {
        // Crear datos con múltiples bloques
        let data: Vec<u8> = (0..total_size).map(|i| (i % 256) as u8).collect();

        let mut compressed = Vec::new();
        {
            let cursor = Cursor::new(&mut compressed);
            let mut writer = CzWriter::new(cursor).unwrap();

            for chunk in data.chunks(block_size) {
                writer.write_block(chunk).unwrap();
            }

            writer.finish().unwrap();
        }

        let num_blocks = (total_size + block_size - 1) / block_size;

        group.throughput(Throughput::Bytes(total_size as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("{}_blocks", num_blocks), block_size),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(compressed.as_slice()));
                    let mut reader = CzReader::new(cursor).unwrap();
                    let mut output: Vec<u8> = Vec::new();

                    while let Some(block) = reader.read_block().unwrap() {
                        output.extend(&block.data);
                    }

                    output
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_decompress_store,
    bench_decompress_patterns,
    bench_decompress_with_verification,
    bench_decompress_multiple_blocks,
);

criterion_main!(benches);
