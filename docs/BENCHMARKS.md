# Benchmarks

Este documento describe cómo ejecutar y analizar los benchmarks de CsZip.

## Prerrequisitos

Los benchmarks requieren Rust estable y la crate `criterion`:

```bash
cargo build --release
```

## Ejecutar Benchmarks

### Todos los Benchmarks

```bash
cargo bench
```

### Benchmark Específico

```bash
# Solo throughput de compresión
cargo bench --bench compress_throughput

# Solo throughput de descompresión
cargo bench --bench decompress_throughput

# Solo ratio de compresión
cargo bench --bench compression_ratio
```

### Benchmark de una Función Específica

```bash
# Solo el grupo específico
cargo bench -- compress_store

# Con filtro regex
cargo bench -- "compress.*block"
```

## Descripción de Benchmarks

### `compress_throughput`

Mide la velocidad de compresión en bytes/segundo.

| Grupo | Descripción |
|-------|-------------|
| `compress_store` | Compresión STORE (sin compresión real) |
| `compress_patterns` | Diferentes patrones de datos |
| `compressor_direct` | Uso directo del Compressor |
| `block_sizes` | Impacto del tamaño de bloque |

### `decompress_throughput`

Mide la velocidad de descompresión en bytes/segundo.

| Grupo | Descripción |
|-------|-------------|
| `decompress_store` | Descompresión STORE |
| `decompress_patterns` | Diferentes patrones de datos |
| `decompress_with_verification` | Con verificación de checksum |
| `decompress_multiple_blocks` | Archivos multi-bloque |

### `compression_ratio`

Analiza la efectividad de compresión para diferentes tipos de datos.

| Patrón | Descripción | Compresibilidad |
|--------|-------------|-----------------|
| `random` | Datos aleatorios | Muy baja |
| `zeros` | Todo ceros | Muy alta |
| `sequential` | 0-255 repetido | Alta |
| `text_english` | Texto en inglés | Alta |
| `text_lorem` | Lorem ipsum | Alta |
| `html` | Markup HTML | Alta |
| `json` | Datos JSON | Alta |
| `binary_mixed` | Mezcla de patrones | Media |
| `repetitive_short` | Patrón AB repetido | Muy alta |
| `repetitive_long` | Alfabeto repetido | Alta |

## Interpretar Resultados

### Ejemplo de Salida

```
compress_store/65536    time:   [45.234 µs 45.456 µs 45.678 µs]
                        thrpt:  [1.3698 GiB/s 1.3765 GiB/s 1.3832 GiB/s]
```

- **time**: Tiempo de ejecución [min | media | max]
- **thrpt**: Throughput (mayor es mejor)

### Comparar con Baseline

```bash
# Guardar baseline
cargo bench -- --save-baseline main

# Comparar con baseline
cargo bench -- --baseline main
```

## Perfilado

### Con flamegraph

```bash
# Instalar
cargo install flamegraph

# Generar
cargo flamegraph --bench compress_throughput
```

### Con perf (Linux)

```bash
perf record cargo bench --bench compress_throughput
perf report
```

### Con Instruments (macOS)

```bash
cargo instruments -t time --bench compress_throughput
```

## Resultados de Referencia

Resultados en hardware de referencia (i7-10700, 32GB RAM, NVMe):

| Operación | Tamaño | Throughput | Tiempo |
|-----------|--------|------------|--------|
| STORE compress | 64KB | 1.4 GB/s | 45 µs |
| STORE compress | 1MB | 1.5 GB/s | 650 µs |
| STORE decompress | 64KB | 1.6 GB/s | 38 µs |
| LZ77 compress | 64KB | 50 MB/s | 1.2 ms |
| Huffman encode | 64KB | 80 MB/s | 800 µs |

*Nota: Estos son resultados indicativos. El rendimiento real depende del hardware y los datos.*

## Optimizaciones

Factores que afectan el rendimiento:

1. **Tamaño de ventana LZ77**: Ventanas más grandes dan mejor ratio pero menor velocidad
2. **Lazy matching**: Mejora ratio a costa de velocidad
3. **Tamaño de bloque**: Bloques más grandes mejoran throughput pero usan más memoria
4. **Tipo de datos**: Datos repetitivos comprimen mucho más rápido

## CI/CD

Los benchmarks se compilan en CI para verificar que no hay regresiones de compilación:

```yaml
- name: Verify benchmarks compile
  run: cargo build --release --benches
```

Para benchmarks de rendimiento en CI, considera usar [criterion-compare-action](https://github.com/benchmark-action/github-action-benchmark).
