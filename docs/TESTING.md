# Testing y Desarrollo

Guía para probar CsZip durante el desarrollo, sin necesidad de instalar el binario.

---

## Ejecutar durante desarrollo

No necesitas compilar un release ni instalar nada. Usa `cargo run --` seguido del comando:

```bash
# Equivalente a: cszip compress archivo.txt
cargo run -- compress archivo.txt

# Equivalente a: cszip decompress archivo.cz
cargo run -- decompress archivo.cz

# Equivalente a: cszip --version
cargo run -- --version

# Equivalente a: cszip --help
cargo run -- --help

# Equivalente a: cszip info -d archivo.cz
cargo run -- info -d archivo.cz
```

Todo lo que va después de `--` se pasa como argumentos al binario.

### Alias rápido (opcional)

Si quieres evitar escribir `cargo run --` cada vez:

```bash
# Bash/Zsh — añadir a ~/.bashrc o ~/.zshrc
alias cszip='cargo run --'

# PowerShell — añadir a $PROFILE
function cszip { cargo run -- @args }
```

---

## Tests automatizados

### Ejecutar todos los tests

```bash
cargo test --all
```

Esto ejecuta los 334 tests del proyecto: unitarios, integración y doctests.

### Ejecutar tests específicos

```bash
# Solo un módulo
cargo test codec_tests
cargo test format_tests
cargo test io_tests
cargo test error_tests
cargo test cli_integration

# Solo un test por nombre
cargo test test_compress_decompress_roundtrip

# Tests de un archivo específico
cargo test --test codec_tests
cargo test --test format_tests
```

### Ver output de los tests

```bash
# Mostrar println! y output de tests que pasan
cargo test -- --nocapture

# Solo un test con output visible
cargo test test_compress_decompress_roundtrip -- --nocapture
```

### Tests ignorados

```bash
# Ejecutar tests marcados con #[ignore]
cargo test -- --ignored

# Ejecutar todos (normales + ignorados)
cargo test -- --include-ignored
```

---

## Verificación completa (lo que hace CI)

Ejecuta esto antes de hacer commit o push:

```bash
# 1. Formato
cargo fmt --check

# 2. Linter
cargo clippy --all-targets -- -D warnings

# 3. Tests
cargo test --all

# 4. Build release (verifica que compila en modo optimizado)
cargo build --release
```

O todo junto:

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --all && cargo build --release
```

---

## Probar manualmente el CLI

### Flujo completo: comprimir → verificar → info → descomprimir

```bash
# 1. Crear archivo de prueba
echo "Contenido de prueba repetido muchas veces" > test.txt

# 2. Comprimir
cargo run -- compress test.txt

# 3. Verificar integridad
cargo run -- verify test.txt.cz

# 4. Ver información
cargo run -- info test.txt.cz
cargo run -- info -d test.txt.cz    # detalle por bloque

# 5. Listar bloques
cargo run -- list test.txt.cz

# 6. Descomprimir
cargo run -- decompress test.txt.cz -o test_restored.txt

# 7. Comparar (deben ser iguales)
diff test.txt test_restored.txt         # Linux/macOS
fc test.txt test_restored.txt           # Windows CMD
Compare-Object (Get-Content test.txt) (Get-Content test_restored.txt)  # PowerShell

# 8. Limpiar
rm test.txt test.txt.cz test_restored.txt
```

### Probar flags

```bash
# Sobrescribir archivo existente
cargo run -- compress test.txt -o out.cz
cargo run -- compress test.txt -o out.cz -f   # sin -f daría error

# Modo silencioso
cargo run -- -q compress test.txt

# Verbose
cargo run -- -v compress test.txt

# Nivel de compresión
cargo run -- compress -l 0 test.txt    # mínimo
cargo run -- compress -l 9 test.txt    # máximo

# CRC-64
cargo run -- compress --crc64 test.txt

# Descomprimir sin verificar
cargo run -- decompress --no-verify test.txt.cz
```

### Probar errores esperados

```bash
# Archivo no existe → error 0x11 (exit code 17)
cargo run -- compress noexiste.txt

# Archivo destino ya existe → error 0x12 (exit code 18)
cargo run -- compress test.txt -o out.cz
cargo run -- compress test.txt -o out.cz      # sin -f → error

# Formato inválido → error 0x01 (exit code 1)
cargo run -- info test.txt                     # no es .cz

# Algoritmo no implementado → error 0x03 (exit code 3)
cargo run -- compress test.txt -a lz77
```

---

## Benchmarks

```bash
# Ejecutar todos los benchmarks
cargo bench

# Un benchmark específico
cargo bench compress_throughput
cargo bench decompress_throughput
cargo bench compression_ratio
```

Los resultados se guardan en `target/criterion/` con gráficos HTML.

---

## Documentación API

```bash
# Generar y abrir en el navegador
cargo doc --no-deps --open
```

---

## Instalar localmente (opcional)

Si quieres probar el binario como si estuviera instalado:

```bash
# Opción 1: cargo install (copia a ~/.cargo/bin que ya está en PATH)
cargo install --path .
cszip --version    # ahora funciona directamente

# Opción 2: release build + copiar
cargo build --release
# Linux/macOS:
sudo cp target/release/cszip /usr/local/bin/
# Windows: copiar target\release\cszip.exe a una carpeta del PATH
```

Para desinstalar:

```bash
cargo uninstall cszip
```

---

## Estructura de tests

| Archivo | Qué prueba | Tests |
|---------|-----------|-------|
| `tests/codec_tests.rs` | Compresión, descompresión, roundtrip | ~100 |
| `tests/format_tests.rs` | Headers, bloques, checksums, constants | ~80 |
| `tests/io_tests.rs` | Reader, Writer, streaming | ~60 |
| `tests/error_tests.rs` | Tipos de error, códigos, conversiones | ~50 |
| `tests/cli_integration.rs` | Comandos CLI end-to-end | ~40 |

---

## Nota sobre exit codes en PowerShell

Al usar `cargo run`, PowerShell puede mostrar `exit code 1` incluso cuando el comando funciona correctamente. Esto ocurre porque `cargo` envía mensajes de compilación a stderr, y PowerShell lo interpreta como error.

Para verificar el exit code real del binario:

```powershell
# Compilar primero, luego ejecutar directamente
cargo build --release
.\target\release\cszip.exe --version
echo $LASTEXITCODE    # debería ser 0
```

O simplemente ignora el mensaje de cargo y verifica que la salida sea correcta.
