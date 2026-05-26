# Referencia de API de CsZip

Esta documentación proporciona la referencia de la API pública expuesta por la librería `cszip`.

---

## 📦 Módulo de Códecs (`cszip::codec`)

Implementa y expone la lógica interna de compresión y descompresión de datos.

### `Algorithm`
Enum que representa los algoritmos de compresión soportados en el formato binario.

```rust
pub enum Algorithm {
    Store = 0,          // Copia directa sin compresión
    Lz77Huffman = 1,    // Compresión LZ77 combinada con codificación Huffman (por defecto)
    Lz4 = 2,            // Reservado para LZ4
    Lzma = 3,           // Reservado para LZMA
    Deflate = 4,        // Reservado para DEFLATE
}
```

### `Compressor`
Estructura encargada de realizar la compresión de bloques en memoria.

```rust
use cszip::codec::{Compressor, Algorithm};

// Crear compresor con algoritmo LZ77+Huffman y nivel 6
let compressor = Compressor::new(Algorithm::Lz77Huffman, 6);

// Comprimir un bloque de datos original
let compressed_data = compressor.compress_block(&original_data)?;
```

### `Decompressor`
Estructura encargada de realizar la descompresión de datos.

```rust
use cszip::codec::{Decompressor, Algorithm};

// Crear descompresor para el algoritmo de bloques respectivo
let decompressor = Decompressor::new(Algorithm::Lz77Huffman);

// Descomprimir bloque
let result = decompressor.decompress_block(&compressed_data)?;
let raw_data = result.data;
```

---

## 🖨️ Módulo de E/S (`cszip::io`)

Manejo optimizado de lectura y escritura de archivos en formato `.cz` por bloques.

### `CzWriter`
Estructura para crear y escribir archivos `.cz` estructurados.

```rust
use cszip::io::CzWriter;
use std::fs::File;

let file = File::create("salida.cz")?;
let mut writer = CzWriter::new(file)?;

// Escribir un bloque de datos
writer.write_block(&data)?;

// Finalizar la escritura (añade el footer global y firma el archivo)
writer.finish()?;
```

### `CzReader`
Estructura para leer y verificar archivos `.cz`.

```rust
use cszip::io::CzReader;
use std::fs::File;

// Abrir archivo directamente usando BufReader interno
let mut reader = CzReader::open("entrada.cz")?;

// Iterar y leer bloques secuencialmente hasta el final del archivo
while let Some(block) = reader.read_block()? {
    println!("Bloque index: {}, tamaño: {}", block.index, block.data.len());
}
```

---

## ⚙️ Módulo de Utilidades (`cszip::utils`)

Funciones comunes para formatear texto y metadatos.

```rust
use cszip::utils;

// Formatear bytes en representación legible (Ej: "1.25 MB")
let text = utils::format_size(1310720);

// Calcular ratio de compresión porcentual
let ratio = utils::compression_ratio(original_size, compressed_size);

// Calcular ahorro de espacio porcentual
let savings = utils::space_savings(original_size, compressed_size);
```
