# ⚡ Quick Start - CsZip

**Empieza AQUÍ para configuras el proyecto en 5 minutos**

---

## 🚀 Paso 1: Crear Proyecto (2 minutos)

```bash
# Navega a tu carpeta de trabajo
cd /path/to/your/workspace

# Crea el proyecto
cargo new CsZip
cd CsZip

# Verifica que funciona
cargo test
```

**Esperado:**

```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored
```

---

## 📝 Paso 2: Copiar Documentación (1 minuto)

Copia estos 4 archivos al raíz del proyecto:

- `README.md` ← Documentación principal
- `FORMAT.md` ← Especificación del formato
- `ARCHITECTURE.md` ← Guía de implementación
- `DEVELOPMENT.md` ← Roadmap de desarrollo
- `LICENSE` ← Para MIT (copia y pega)

---

## ⚙️ Paso 3: Configurar Cargo.toml (1 minuto)

**Reemplaza `[Cargo.toml](Cargo.toml) con:**

```toml
[package]
name = "CsZip"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
```

**Ejecuta:**

```bash
cargo check
```

---

## 📁 Paso 4: Crear Estructura (1 minuto)

```bash
mkdir -p src/{codec,format,io,cli}
mkdir -p tests
mkdir -p examples
mkdir -p docs
```

**Structure tree:**

```
CsZip/
├── Cargo.toml
├── README.md
├── FORMAT.md
├── ARCHITECTURE.md
├── DEVELOPMENT.md
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── error.rs
│   ├── codec/
│   │   └── mod.rs
│   ├── format/
│   │   └── mod.rs
│   ├── io/
│   │   └── mod.rs
│   └── cli/
│       └── mod.rs
├── tests/
└── examples/
```

---

## ✍️ Paso 5: Primer Código (Empieza AQUÍ)

### 5.1 Crear `src/error.rs`

```rust
// src/error.rs - Manejo de errores

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CsZip Error: {}", self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
```

### 5.2 Actualizar `src/lib.rs`

```rust
// src/lib.rs

pub mod error;
pub mod codec;
pub mod format;
pub mod io;

pub use error::{Error, Result};

/// Versión de la librería
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Comprimir datos
pub fn compress(data: &[u8], _level: u8) -> Result<Vec<u8>> {
    // TODO: Implementar
    Ok(data.to_vec())
}

/// Descomprimir datos
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implementar
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
```

### 5.3 Crear stubs básicos

**src/codec/mod.rs:**

```rust
pub mod compressor;
pub mod decompressor;
```

**src/format/mod.rs:**

```rust
pub mod header;
pub mod block;
pub mod checksum;
pub mod constants;
```

**src/io/mod.rs:**

```rust
pub mod reader;
pub mod writer;
```

**src/cli/mod.rs:**

```rust
pub mod options;
pub mod commands;
```

---

## 🧪 Paso 6: Verificar Build (1 minuto)

```bash
# Compilar librería
cargo build

# Ejecutar tests
cargo test

# Verificar sin warnings
cargo clippy

# Formatear código
cargo fmt
```

**Todo debe pasar sin errores.**

---

## 🎯 Próximos Pasos: ORDEN DE IMPLEMENTACIÓN

### Semana 1-2: Fundamentos

1. **`src/format/constants.rs`** - Todas las constantes del formato
2. **`src/format/header.rs`** - Header global (16 bytes)
3. **`src/format/block.rs`** - Block header/footer
4. **`src/format/checksum.rs`** - CRC-32 y CRC-64
5. **Tests exhaustivos** para cada módulo

### Semana 3: Compresión Básica

6. **`src/codec/algorithm.rs`** - Algoritmo STORE (sin compresión)
7. **`src/codec/compressor.rs`** - Compresor simple
8. **`src/codec/decompressor.rs`** - Descompresor
9. **Tests de roundtrip** (compress → decompress → original)

### Semana 4-5: CLI y I/O

10. **`src/io/reader.rs`** - Lectura bufferizada
11. **`src/io/writer.rs`** - Escritura bufferizada
12. **`src/cli/options.rs`** - Parseo de argumentos con clap
13. **`src/main.rs`** - Implementar: `-c`, `-d`, `-t`, `-l`

### Semana 6+: Optimización

14. Implementar LZ77
15. Implementar Huffman
16. Optimizar performance
17. Paralelización (opcional)

---

## 📚 Referencias Rápidas

### Aprender el formato

```bash
# Leer especificación completa
less FORMAT.md

# Entender arquitectura
less ARCHITECTURE.md

# Ver roadmap completo
less DEVELOPMENT.md
```

### Comandos útiles

```bash
# Compilar en release
cargo build --release

# Ejecutar ejemplo
cargo run --example basic_compress

# Generar documentación
cargo doc --open

# Ejecutar benchmarks
cargo bench

# Fuzzing (requiere toolchain nightly)
cargo +nightly fuzz run fuzz_decompressor
```

---

## ✅ Checklist Diario

Cada día de desarrollo:

1. ✅ `cargo test --all` pasa
2. ✅ `cargo clippy` sin warnings
3. ✅ `cargo fmt` aplicado
4. ✅ Commits frecuentes (cada tarea pequeña)

---

## 🐛 Troubleshooting

### Problema: "cargo check" falla

```bash
# Limpia cache
cargo clean
cargo check
```

### Problema: Warnings de clippy

```bash
# Aplica formato automático
cargo fix --allow-dirty
cargo fmt
cargo clippy --fix --allow-dirty
```

### Problema: No puedo encontrar archivos

```bash
# Verifica estructura
ls -la src/
ls -la

# Recrear archivos si falta alguno
touch src/error.rs
touch src/lib.rs
```

---

## 📞 Próximos Pasos

1. **Hoy:** Crear proyecto + estructura (este documento)
2. **Mañana:** Implementar `error.rs` + `format/constants.rs`
3. **Día 3:** Implementar `format/header.rs` + tests
4. **Día 4:** Implementar `format/block.rs` + tests
5. **Día 5:** Implementar `format/checksum.rs` + tests

Después de eso, sigue el roadmap en `DEVELOPMENT.md`.

---

## 💡 Tips Profesionales

### 1. Usa cargo watch para desarrollo iterativo

```bash
cargo install cargo-watch
cargo watch -x test -x clippy
```

### 2. Documenta mientras codificas

````rust
/// Realiza X con Y
///
/// # Errores
/// Retorna error si...
///
/// # Ejemplos
/// ```
/// let result = my_function()?;
/// ```
pub fn my_function() -> Result<()> {
    Ok(())
}
````

### 3. Crea commits frecuentes

```bash
# Cada función pequeña
git add src/format/header.rs
git commit -m "feat: implementar Header::read()"
```

### 4. Escribe tests mientras codificas

```rust
#[test]
fn test_my_feature() {
    // Primero escribe el test
    // Luego implementa la función
}
```

---

## 🎉 Milestone Timeline Estimado

| Checkpoint       | Duración      | Entregable                 |
| ---------------- | ------------- | -------------------------- |
| Proyecto base    | 1 día         | Compilación funcional      |
| Formato completo | 1 semana      | Format validado            |
| Algoritmo básico | 1 semana      | Compress/decompress manual |
| CLI funcional    | 1 semana      | `-c`, `-d`, `-t`, `-l`     |
| **MVP**          | **4 semanas** | **v0.1.0 funcional**       |
| Optimización     | 3 semanas     | v0.2.0 rápido              |
| Hardening        | 3 semanas     | v1.0.0 seguro              |

---

<div align="center">

**¡ABRE VSCODE Y EMPIEZA AHORA!** 🚀

Comenzando por `src/error.rs` y `DEVELOPMENT.md` para tu próximo paso.

</div>
