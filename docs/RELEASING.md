# Crear Releases

## Cómo funciona

CsZip usa **GitHub Actions** para automatizar la creación de releases. Al pushear un tag de versión, el workflow automáticamente:

1. Compila binarios para 5 plataformas en paralelo
2. Ejecuta tests en cada plataforma
3. Empaqueta cada binario con README y LICENSE
4. Genera checksums SHA-256
5. Crea un Release en GitHub con todos los archivos
6. Publica en crates.io (solo versiones estables)

**No necesitas compilar nada manualmente.** GitHub lo hace todo por ti en sus servidores.

---

## Crear un release

### 1. Actualizar versión en Cargo.toml

```toml
[package]
version = "0.2.0"  # nueva versión
```

### 2. Commit y tag

```bash
git add Cargo.toml
git commit -m "v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 3. Esperar

GitHub Actions tarda ~5-10 minutos. Puedes ver el progreso en la pestaña **Actions** del repositorio.

### 4. Resultado

En la pestaña **Releases** del repositorio aparecerá:

```
CsZip v0.2.0
├── cszip-linux-x86_64.tar.gz
├── cszip-linux-x86_64-musl.tar.gz
├── cszip-macos-x86_64.tar.gz
├── cszip-macos-aarch64.tar.gz
├── cszip-windows-x86_64.zip
└── checksums.sha256
```

Los usuarios pueden descargar el binario directamente sin compilar nada.

---

## Qué es GitHub Actions

GitHub Actions es un servicio de CI/CD integrado en GitHub. Ejecuta tareas automáticas (compilar, testear, publicar) en servidores de GitHub cuando ocurren eventos (push, tag, pull request).

El archivo `.github/workflows/release.yml` define qué hacer cuando se pushea un tag `v*`:

```
Evento: git push --tags (v0.2.0)
    │
    ▼
┌─────────────────────────────────────────────────┐
│  Job: build (5 plataformas en paralelo)         │
│  ├─ ubuntu-latest  → cszip-linux-x86_64.tar.gz  │
│  ├─ ubuntu-latest  → cszip-linux-x86_64-musl     │
│  ├─ macos-latest   → cszip-macos-x86_64          │
│  ├─ macos-latest   → cszip-macos-aarch64          │
│  └─ windows-latest → cszip-windows-x86_64.zip    │
├─────────────────────────────────────────────────┤
│  Job: release (espera a build)                  │
│  └─ Descarga artefactos → genera checksums      │
│     → crea Release en GitHub                    │
├─────────────────────────────────────────────────┤
│  Job: publish-crates (espera a release)         │
│  └─ cargo publish → crates.io                   │
└─────────────────────────────────────────────────┘
```

**No necesitas configurar nada.** El workflow ya está en `.github/workflows/release.yml`.

### Secretos necesarios

Para publicar en crates.io, añade el token como secreto en GitHub:

1. Ve a https://crates.io/settings/tokens y crea un token
2. En tu repositorio: Settings → Secrets → Actions → New repository secret
3. Nombre: `CRATES_TOKEN`, valor: tu token

El `GITHUB_TOKEN` se proporciona automáticamente por GitHub.

---

## Versionado

Usa [Semantic Versioning](https://semver.org/):

| Tag | Tipo | Cuándo |
|-----|------|--------|
| `v0.1.0` → `v0.2.0` | Minor | Nueva funcionalidad |
| `v0.2.0` → `v0.2.1` | Patch | Corrección de bug |
| `v0.9.0` → `v1.0.0` | Major | Cambio incompatible |
| `v1.0.0-rc.1` | Pre-release | Candidato a release (no publica en crates.io) |

---

## Compilar manualmente (opcional)

Si alguna vez necesitas compilar binarios a mano sin GitHub Actions:

```bash
# Linux
cargo build --release
tar -czvf cszip-linux-x86_64.tar.gz -C target/release cszip

# Linux estático
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
tar -czvf cszip-linux-x86_64-musl.tar.gz -C target/x86_64-unknown-linux-musl/release cszip

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
tar -czvf cszip-macos-aarch64.tar.gz -C target/aarch64-apple-darwin/release cszip

# Windows
cargo build --release
Compress-Archive target\release\cszip.exe cszip-windows-x86_64.zip
```

### Checksums

```bash
sha256sum cszip-*.tar.gz cszip-*.zip > checksums.sha256
```

---

## Verificar un release

Los usuarios pueden verificar la integridad de lo descargado:

```bash
curl -LO https://github.com/tu-usuario/cszip/releases/download/v0.2.0/checksums.sha256
sha256sum -c checksums.sha256
```
