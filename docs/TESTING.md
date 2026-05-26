# Testing y Desarrollo en CsZip

Este documento detalla la estructura de pruebas automatizadas y cómo validar la calidad del código del proyecto `cszip`.

---

## 🚀 Pruebas Automatizadas con Scripts (Recomendado)

Para mayor comodidad, el proyecto incluye scripts que se encargan de verificar el formateo, ejecutar linter (`clippy`) y correr todas las pruebas unitarias y de integración automáticamente.

**En Linux / macOS (POSIX):**
```bash
./scripts/dev.sh
```

**En Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

---

## 🧪 Estructura de la Suite de Pruebas

El proyecto cuenta con una amplia suite de pruebas que valida el comportamiento físico del formato binario, el streaming de E/S y la lógica de compresión de los codecs.

| Archivo de Test | Componente Evaluado | Número de Tests |
|-----------------|---------------------|-----------------|
| `tests/codec_tests.rs` | Motores de compresión y descompresión, roundtrips con `STORE` y `LZ77+Huffman`. | ~100 |
| `tests/format_tests.rs` | Serialización de cabeceras, formato físico, constantes y algoritmos de checksum. | ~80 |
| `tests/io_tests.rs` | Flujos de entrada/salida (`CzReader`/`CzWriter`), bufferización y streaming con progreso. | ~60 |
| `tests/error_tests.rs` | Códigos de error numéricos, manejo de tipos de error y conversiones. | ~50 |
| `tests/cli_integration.rs` | Flujos end-to-end de comandos del CLI y redirección ZIP/RAR. | ~40 |

---

## 🛠️ Ejecución de Pruebas Manuales con Cargo

Si deseas ejecutar pruebas específicas de forma directa con Cargo:

### Correr todos los tests
```bash
cargo test
```

### Correr un grupo específico de pruebas
```bash
# Ejecutar solo tests de codec
cargo test --test codec_tests

# Ejecutar un test individual por nombre
cargo test test_roundtrip_lz77_huffman
```

### Ver la salida de consola de los tests
Nativamente, Cargo captura la salida estándar. Para visualizar logs o impresiones `println!` en pruebas que pasan:
```bash
cargo test test_roundtrip_lz77_huffman -- --nocapture
```

---

## 📊 Benchmarks y Rendimiento

El rendimiento de los algoritmos de compresión y descompresión se evalúa mediante `criterion`.

### Ejecutar todos los benchmarks
```bash
cargo bench
```

### Ejecutar un benchmark específico
```bash
# Medir el ratio de compresión
cargo bench --bench compression_ratio

# Medir la velocidad de descompresión
cargo bench --bench decompress_throughput
```
*Los reportes con gráficos interactivos HTML se generan en `target/criterion/report/index.html`.*

---

## 🧹 Verificación de Calidad Manual

Antes de subir cambios, asegúrate de cumplir con los lints y directrices del compilador de Rust:

```bash
# 1. Formatear código
cargo fmt -- --check

# 2. Ejecutar linter con advertencias tratadas como errores
cargo clippy --all-targets -- -D warnings
```
