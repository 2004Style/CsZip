# Instalación

## Requisitos

| Componente | Mínimo |
|------------|--------|
| Rust | 1.70+ |
| RAM | 512 MB |
| Disco | 100 MB |

---

## Binarios pre-compilados (más rápido)

Descarga desde [GitHub Releases](https://github.com/tu-usuario/cszip/releases/latest) el binario para tu sistema:

```bash
# Linux x86_64
curl -LO https://github.com/tu-usuario/cszip/releases/latest/download/cszip-linux-x86_64.tar.gz
tar -xzf cszip-linux-x86_64.tar.gz
sudo mv cszip /usr/local/bin/

# Linux x86_64 (estático, sin dependencias)
curl -LO https://github.com/tu-usuario/cszip/releases/latest/download/cszip-linux-x86_64-musl.tar.gz
tar -xzf cszip-linux-x86_64-musl.tar.gz
sudo mv cszip /usr/local/bin/

# macOS
curl -LO https://github.com/tu-usuario/cszip/releases/latest/download/cszip-macos-aarch64.tar.gz
tar -xzf cszip-macos-aarch64.tar.gz
sudo mv cszip /usr/local/bin/
```

Windows: descarga `cszip-windows-x86_64.zip`, extrae y añade la carpeta al `PATH`.

Verifica:

```bash
cszip --version
```

### Verificar integridad

```bash
curl -LO https://github.com/tu-usuario/cszip/releases/latest/download/checksums.sha256
sha256sum -c checksums.sha256
```

---

## Compilar desde fuente

### Linux

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y build-essential curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
sudo cp target/release/cszip /usr/local/bin/
```

**Fedora:**

```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
```

**Arch:**

```bash
sudo pacman -S base-devel rust git
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
```

### macOS

```bash
xcode-select --install  # herramientas de compilación
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
sudo cp target/release/cszip /usr/local/bin/
```

### Windows

1. Instalar [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (seleccionar "Desarrollo de escritorio con C++")
2. Instalar Rust desde [rustup.rs](https://rustup.rs)
3. En PowerShell:

```powershell
git clone https://github.com/tu-usuario/cszip.git
cd cszip
cargo build --release
# Binario en target\release\cszip.exe
```

---

## Compilación optimizada

```bash
# Optimizar para tu CPU específica (máxima velocidad)
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Binario estático (Linux)

Sin dependencias externas, funciona en cualquier distro Linux:

```bash
rustup target add x86_64-unknown-linux-musl
sudo apt install musl-tools  # Ubuntu/Debian
cargo build --release --target x86_64-unknown-linux-musl
# Binario: target/x86_64-unknown-linux-musl/release/cszip
```

### Cross-compilation macOS

```bash
# Apple Silicon desde Intel (o viceversa)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Universal binary (ambas arquitecturas)
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create \
  target/x86_64-apple-darwin/release/cszip \
  target/aarch64-apple-darwin/release/cszip \
  -output cszip-universal
```

---

## Instalar globalmente

```bash
# Opción 1: cargo install (compila e instala en ~/.cargo/bin)
cargo install --path .

# Opción 2: copiar manualmente
sudo cp target/release/cszip /usr/local/bin/  # Linux/macOS
```

## Desinstalar

```bash
cargo uninstall cszip  # si se instaló con cargo install
# o
sudo rm /usr/local/bin/cszip
```

---

## Solución de problemas

**`cargo: command not found`** — Ejecuta `source "$HOME/.cargo/env"` o reinicia la terminal.

**`linker cc not found`** — Instala herramientas de compilación: `sudo apt install build-essential` (Ubuntu) o `xcode-select --install` (macOS).

**Error de compilación musl** — Instala `musl-tools`: `sudo apt install musl-tools`.

**Lento al compilar** — Es normal la primera vez. Las compilaciones siguientes son incrementales y más rápidas.
