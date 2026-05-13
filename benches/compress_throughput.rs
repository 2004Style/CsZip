//! Benchmark de throughput de compresión
//!
//! Mide la velocidad de compresión en bytes/segundo.
//!
//! ```bash
//! cargo bench --bench compress_throughput
//! ```

use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

use cszip::codec::{Algorithm, CompressionLevel, Compressor};
use cszip::io::CzWriter;

/// Generar datos de prueba
fn generate_test_data(size: usize, pattern: &str) -> Vec<u8> {
    match pattern {
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
    }
}

fn bench_compress_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress_store");
    
    let sizes = [1024, 4096, 16384, 65536, 262144]; // 1K, 4K, 16K, 64K, 256K
    
    for size in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        
        let data = generate_test_data(size, "text");
        
        group.bench_with_input(BenchmarkId::new("text", size), &data, |b, data| {
            b.iter(|| {
                let mut output = Vec::with_capacity(size + 1024);
                let cursor = Cursor::new(&mut output);
                let mut writer = CzWriter::new(cursor).unwrap();
                writer.write_block(black_box(data)).unwrap();
                writer.finish().unwrap();
                output
            });
        });
    }
    
    group.finish();
}

fn bench_compress_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress_patterns");
    
    let size = 65536; // 64K
    let patterns = ["random", "zeros", "sequential", "text", "repetitive"];
    
    for pattern in patterns {
        group.throughput(Throughput::Bytes(size as u64));
        
        let data = generate_test_data(size, pattern);
        
        group.bench_with_input(BenchmarkId::new(pattern, size), &data, |b, data| {
            b.iter(|| {
                let mut output = Vec::with_capacity(size + 1024);
                let cursor = Cursor::new(&mut output);
                let mut writer = CzWriter::new(cursor).unwrap();
                writer.write_block(black_box(data)).unwrap();
                writer.finish().unwrap();
                output
            });
        });
    }
    
    group.finish();
}

fn bench_compressor_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("compressor_direct");
    
    let size = 32768;
    let data = generate_test_data(size, "text");
    
    group.throughput(Throughput::Bytes(size as u64));
    
    group.bench_function("store_block", |b| {
        let compressor = Compressor::new(Algorithm::Store, CompressionLevel::default());
        b.iter(|| {
            compressor.compress_block(black_box(&data)).unwrap()
        });
    });
    
    group.finish();
}

fn bench_block_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_sizes");
    
    let total_size = 262144; // 256K total
    let block_sizes = [512, 1024, 4096, 16384, 32768, 65536];
    let data = generate_test_data(total_size, "text");
    
    for block_size in block_sizes {
        group.throughput(Throughput::Bytes(total_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("block_size", block_size),
            &block_size,
            |b, &block_size| {
                b.iter(|| {
                    let mut output = Vec::with_capacity(total_size + 4096);
                    let cursor = Cursor::new(&mut output);
                    let mut writer = CzWriter::new(cursor).unwrap();
                    
                    for chunk in data.chunks(block_size) {
                        writer.write_block(black_box(chunk)).unwrap();
                    }
                    
                    writer.finish().unwrap();
                    output
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_compress_store,
    bench_compress_patterns,
    bench_compressor_direct,
    bench_block_sizes,
);

criterion_main!(benches);
