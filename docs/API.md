# API Reference

Esta documentación proporciona una referencia completa de la API pública de CsZip.

## Módulos

### `cszip::codec`

Módulo de compresión y descompresión.

#### `Compressor`

```rust
use cszip::codec::{Compressor, CompressionMethod};

// Crear compresor STORE
let mut compressor = Compressor::new(CompressionMethod::Store);

// Comprimir datos
let compressed = compressor.compress(&data)?;
```

#### `Decompressor`

```rust
use cszip::codec::Decompressor;

let decompressor = Decompressor::new();
let decompressed = decompressor.decompress(&compressed, original_size)?;
```

#### `Lz77Compressor`

Compresor basado en el algoritmo LZ77.

```rust
use cszip::codec::{Lz77Compressor, Lz77Config};

// Configuración por defecto
let compressor = Lz77Compressor::new();
let tokens = compressor.compress(&data);
let bytes = compressor.compress_to_bytes(&data);

// Con configuración personalizada
let config = Lz77Config {
    window_size: 32768,
    min_match_length: 3,
    max_match_length: 258,
    lazy_matching: true,
};
let compressor = Lz77Compressor::with_config(config);
```

#### `HuffmanEncoder` / `HuffmanDecoder`

```rust
use cszip::codec::{HuffmanEncoder, HuffmanDecoder};

// Codificar
let mut encoder = HuffmanEncoder::new();
let encoded = encoder.encode(&data)?;

// Decodificar
let mut decoder = HuffmanDecoder::new();
let decoded = decoder.decode(&encoded)?;
```

#### Filtros de Preprocesamiento

```rust
use cszip::codec::{DeltaFilter, MtfTransform, RleEncoder};

// Filtro Delta
let filtered = DeltaFilter::encode(&data);
let original = DeltaFilter::decode(&filtered);

// Move-to-Front
let mut mtf = MtfTransform::new();
let encoded = mtf.encode(&data);
let decoded = mtf.decode(&encoded);

// Run-Length Encoding
let mut rle = RleEncoder::new();
let compressed = rle.encode(&data);
let decompressed = rle.decode(&compressed)?;
```

---

### `cszip::io`

Módulo de I/O para lectura y escritura de archivos `.cz`.

#### `CzWriter`

```rust
use cszip::io::CzWriter;
use std::fs::File;

let file = File::create("output.cz")?;
let mut writer = CzWriter::new(file)?;

// Escribir bloques
writer.write_block(&data)?;
writer.write_block(&more_data)?;

// Finalizar archivo
writer.finish()?;
```

#### `CzReader`

```rust
use cszip::io::CzReader;
use std::fs::File;

let file = File::open("input.cz")?;
let mut reader = CzReader::new(file)?;

// Leer header
let version = reader.version();

// Iterar bloques
while let Some(block) = reader.next_block()? {
    process(&block);
}

// Verificar integridad
reader.verify()?;
```

#### Streaming

```rust
use cszip::io::{StreamingCompressor, StreamingDecompressor, StreamOptions};

// Compresión streaming
let options = StreamOptions::default()
    .with_block_size(65536)
    .with_checksum(true);

let compressor = StreamingCompressor::with_options(writer, options)?;
compressor.write_chunk(&chunk)?;
compressor.finish()?;

// Descompresión streaming
let decompressor = StreamingDecompressor::new(reader)?;
while let Some(chunk) = decompressor.read_chunk()? {
    process(&chunk);
}
```

---

### `cszip::format`

Estructuras del formato de archivo.

#### `Header`

```rust
use cszip::format::Header;

let header = Header::new(1, 0);  // versión 1.0
let bytes = header.to_bytes();
```

Constantes:
- `MAGIC`: `[0x43, 0x53, 0x5A, 0x50]` ("CSZP")
- `SIZE`: 16 bytes

#### `BlockHeader`

```rust
use cszip::format::{BlockHeader, CompressionMethod, ChecksumType};

let block_header = BlockHeader::new(
    original_size,
    compressed_size,
    CompressionMethod::Store,
    ChecksumType::Crc32,
);
```

SIZE: 12 bytes

#### `Footer`

```rust
use cszip::format::Footer;

let footer = Footer::new(block_count, total_size_original);
```

SIZE: 12 bytes

---

### `cszip::error`

Tipos de error.

```rust
use cszip::error::{CzError, Result};

fn my_function() -> Result<()> {
    // Usar ? para propagar errores
    Ok(())
}
```

Variantes de `CzError`:
- `Io(std::io::Error)` - Errores de I/O
- `InvalidMagic` - Magic number inválido
- `InvalidVersion` - Versión no soportada
- `InvalidChecksum` - Checksum no coincide
- `InvalidCompressionMethod` - Método de compresión desconocido
- `DecompressionError` - Error durante descompresión
- `FormatError(String)` - Error de formato genérico

---

### `cszip::utils`

Funciones de utilidad.

```rust
use cszip::utils;

// Formatear tamaños
let size_str = utils::format_size(1048576);  // "1.00 MB"

// Calcular ratio
let ratio = utils::compression_ratio(1000, 500);  // 50.0

// Calcular ahorro
let savings = utils::space_savings(1000, 500);  // 50.0

// Verificar extensión
let is_cz = utils::is_cz_file("archivo.cz");  // true

// Formatear duración
let duration_str = utils::format_duration(std::time::Duration::from_secs(65));  // "1m 5s"
```

---

### `cszip::cli`

Módulo CLI (cuando se compila como binario).

#### `ProgressBar`

```rust
use cszip::cli::{ProgressBar, ProgressConfig, ProgressStyle};

let config = ProgressConfig::new(total_size)
    .with_style(ProgressStyle::Bar)
    .with_width(50);

let pb = ProgressBar::new(config);
pb.update(processed_bytes);
pb.finish("¡Completado!");
```

---

## Feature Flags

| Feature | Descripción | Default |
|---------|-------------|---------|
| `default` | Características estándar | ✓ |
| `progress` | Barra de progreso con indicatif | ✗ |
| `lz4` | Soporte para compresión LZ4 | ✗ |
| `lzma` | Soporte para compresión LZMA | ✗ |

```toml
[dependencies]
cszip = { version = "0.1", features = ["progress"] }
```

---

## Códigos de Error CLI

| Código | Significado |
|--------|-------------|
| 0 | Éxito |
| 1 | Error general |
| 2 | Argumentos inválidos |
| 3 | Archivo no encontrado |
| 4 | Error de I/O |
| 5 | Archivo corrupto |

---

## Ejemplos Completos

Ver el directorio `examples/` para ejemplos completos:

- `basic_compress.rs` - Compresión básica
- `basic_decompress.rs` - Descompresión básica
- `streaming.rs` - Operaciones streaming
- `library_api.rs` - Uso como biblioteca
