# Manual de Uso

## Referencia rápida

```bash
cszip compress <archivo>       # comprimir
cszip decompress <archivo.cz>  # descomprimir
cszip verify <archivo.cz>      # verificar integridad
cszip info <archivo.cz>        # ver metadata
cszip list <archivo.cz>        # listar bloques
```

Alias cortos: `c`, `d`, `v`, `i`, `l`.

---

## Comprimir

```bash
cszip compress archivo.txt
cszip c archivo.txt                    # alias
```

**Salida:** `archivo.txt.cz` (el original no se modifica).

### Opciones

| Opción | Descripción | Default |
|--------|-------------|---------|
| `-o <ruta>` | Archivo de salida | `<input>.cz` |
| `-a <algo>` | Algoritmo: `store`, `lz77`, `lz4`, `lzma`, `deflate` | `store` |
| `-l <0-9>` | Nivel de compresión (0=rápido, 9=máximo) | `6` |
| `-f` | Sobrescribir si el archivo ya existe | — |
| `--crc64` | Usar CRC-64 en vez de CRC-32 | CRC-32 |
| `-v` | Verbose (muestra detalles) | — |
| `-q` | Silencioso (solo errores) | — |

### Ejemplos

```bash
# Compresión básica
cszip compress datos.bin

# Nivel máximo, nombre personalizado
cszip compress -l 9 -o backup.cz datos.bin

# Sobrescribir, CRC-64, verbose
cszip compress -f --crc64 -v datos.bin

# Comprimir varios archivos
for f in *.log; do cszip compress "$f"; done
```

### Salida de ejemplo

```
Comprimiendo: datos.bin -> datos.bin.cz
Algoritmo: STORE, Nivel: 6

Resultado:
  Tamaño original:  1048576 bytes
  Tamaño comprimido: 1048576 bytes
  Ratio: 100.00%
  Bloques: 32
  CRC-32: 0x1A2B3C4D
  Tiempo: 0.02s
```

---

## Descomprimir

```bash
cszip decompress archivo.txt.cz
cszip d archivo.txt.cz                # alias
```

**Salida:** `archivo.txt` (restaura el nombre original).

### Opciones

| Opción | Descripción | Default |
|--------|-------------|---------|
| `-o <ruta>` | Archivo de salida | quita `.cz` del nombre |
| `-f` | Sobrescribir si ya existe | — |
| `--no-verify` | No verificar checksums (más rápido) | verifica |

### Ejemplos

```bash
# Descompresión básica
cszip decompress datos.bin.cz

# Salida personalizada
cszip decompress -o restaurado.bin datos.cz

# Sobrescribir existente
cszip decompress -f datos.bin.cz

# Sin verificación (máxima velocidad)
cszip decompress --no-verify datos.bin.cz
```

---

## Verificar integridad

Comprueba que el archivo no está corrupto sin descomprimirlo:

```bash
cszip verify archivo.cz
cszip v archivo.cz
```

Verifica:
- Magic number y versión del header
- ADLER-32 de cada bloque
- CRC-32 global contra el footer
- Número de bloques correcto

```
Verificando: datos.bin.cz

✓ Archivo válido
  Bloques verificados: 32
  CRC-32: 0x1A2B3C4D
  Tiempo: 0.01s
```

---

## Información del archivo

```bash
cszip info archivo.cz        # resumen
cszip info -d archivo.cz     # detalle por bloque
```

```
Información de archivo: datos.bin.cz
--------------------------------------------------
Version:     1.0
Algoritmo:   STORE (0)
Tamaño bloque: 32768 bytes
CRC:         CRC-32

Estadisticas del archivo:
  Bloques:    32
  Tamano original: 1048576 bytes
  Checksum global: 0x1A2B3C4D
```

Con `-d` (detallado):

```
Bloques:
  [0] Original: 32768 bytes, Comprimido: 32768 bytes, CRC: 0xABCD1234
  [1] Original: 32768 bytes, Comprimido: 32768 bytes, CRC: 0xEF567890
  ...
```

---

## Listar bloques

```bash
cszip list archivo.cz
cszip list -v archivo.cz    # con tabla detallada
```

Con `-v`:

```
Archivo: datos.bin.cz
Bloques: 32
Tamano original: 1048576 bytes

Detalle de bloques:
Bloque     Original   Comprimido    Ratio
---------------------------------------------
    0        32768        32768   100.0%
    1        32768        32768   100.0%
    ...
```

---

## Flags globales

Estas opciones funcionan con cualquier comando:

| Flag | Descripción |
|------|-------------|
| `-v` | Verbose — muestra más información |
| `-vv` | Muy verbose — detalle de cada bloque |
| `-vvv` | Debug — toda la información disponible |
| `-q` | Silencioso — solo errores |
| `--help` | Ayuda del comando |
| `--version` | Versión de cszip |

---

## Uso como biblioteca (Rust)

Añade a tu `Cargo.toml`:

```toml
[dependencies]
cszip = "0.1"
```

### Comprimir datos

```rust
use cszip::io::CzWriter;
use std::io::Cursor;

let data = b"Datos a comprimir";
let mut buffer = Vec::new();
{
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CzWriter::new(cursor).unwrap();
    writer.write_block(data).unwrap();
    writer.finish().unwrap();
}
// buffer contiene el archivo .cz
```

### Descomprimir datos

```rust
use cszip::io::CzReader;
use std::io::Cursor;

let cursor = Cursor::new(&buffer);
let mut reader = CzReader::new(cursor).unwrap();
while let Some(block) = reader.read_block().unwrap() {
    println!("Bloque {}: {} bytes", block.index, block.data.len());
}
```

### Comprimir/descomprimir archivos

```rust
use cszip::{compress_file, decompress_file};
use std::path::Path;

compress_file(Path::new("datos.bin"), None).unwrap();
decompress_file(Path::new("datos.bin.cz"), None).unwrap();
```

### Streaming

```rust
use cszip::io::{StreamingCompressor, StreamOptions};
use std::io::Cursor;

let options = StreamOptions::default()
    .with_block_size(8192)
    .with_memory_limit(1024 * 1024);

let output = Cursor::new(Vec::new());
let mut compressor = StreamingCompressor::new(output, options).unwrap();
compressor.compress_stream(&mut input_reader).unwrap();
let stats = compressor.finish().unwrap();
```

---

## Pipes y scripts

```bash
# Comprimir desde stdin (próximamente)
cat datos.bin | cszip compress -o datos.cz -

# Comprimir todos los .log mayores de 1MB
find /var/log -name "*.log" -size +1M -exec cszip compress {} \;

# Verificar todos los .cz de un directorio
for f in *.cz; do cszip verify "$f" || echo "CORRUPTO: $f"; done

# Comprimir y verificar
cszip compress datos.bin && cszip verify datos.bin.cz
```

---

## Códigos de salida

| Código | Significado |
|--------|-------------|
| `0` | Éxito |
| `1` | Error de formato (magic number, versión) |
| `7` | CRC no coincide |
| `8` | Posible zip bomb |
| `17` | Archivo no encontrado |
| `18` | Archivo ya existe |
| `19` | Error de I/O |
