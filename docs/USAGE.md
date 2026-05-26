# Manual de Uso de CsZip

`cszip` es una herramienta CLI versátil para compresión y descompresión. Soporta de forma transparente el formato nativo `.cz` (con compresión LZ77+Huffman) y formatos estándar de la industria como `.zip` y `.rar`.

---

## Referencia Rápida de Comandos

```bash
cszip compress <archivo>             # Comprimir usando el algoritmo por defecto (.cz)
cszip compress <archivo> -o doc.zip   # Comprimir a archivo zip estándar
cszip decompress <archivo.cz>        # Descomprimir un archivo .cz
cszip decompress <archivo.zip>       # Descomprimir un archivo .zip nativamente
cszip decompress <archivo.rar>       # Extraer un archivo .rar (vía unrar)
cszip verify <archivo.cz>            # Verificar checksums e integridad de un archivo .cz
cszip info <archivo.cz>              # Mostrar metadata y estructura de bloques de un .cz
cszip list <archivo.cz>              # Listar bloques y ratios de compresión
```

*Nota: Todos los comandos tienen alias cortos correspondientes: `c`, `d`, `v`, `i`, `l`.*

---

## 💾 Comprimir

El comportamiento por defecto del comando `compress` (o `c`) depende de la extensión de salida proporcionada en el argumento `-o`:

1. **Sin especificar `-o` o especificando salida `.cz`**: Comprime los datos al formato nativo `.cz` usando el algoritmo nativo `LZ77+Huffman` (o `STORE` si se indica explícitamente).
2. **Especificando salida `.zip`**: Crea un archivo comprimido en formato `.zip` estándar utilizando compresión DEFLATE nativa.

### Opciones de Compresión

| Opción | Descripción | Valor por Defecto |
|--------|-------------|-------------------|
| `-o <ruta>` | Ruta del archivo de salida | `<input>.cz` |
| `-a <algoritmo>`| Algoritmo para formato `.cz` (`store`, `lz77-huffman`) | `lz77-huffman` |
| `-l <0-9>` | Nivel de compresión (0 = sin compresión, 9 = compresión máxima) | `6` |
| `-f` | Fuerza la escritura sobrescribiendo el archivo destino si existe | — |
| `--crc64` | Utiliza verificación CRC-64 en lugar de CRC-32 en los bloques `.cz` | CRC-32 |
| `-v` / `-vv` | Nivel de verbosidad (detalla el proceso bloque a bloque) | — |
| `-q` | Modo silencioso (solo reporta errores) | — |

### Ejemplos

```bash
# Compresión nativa estándar a .cz (usa LZ77+Huffman)
cszip compress documento.txt

# Comprimir a formato .cz con nivel máximo (9) y nombre personalizado
cszip compress -l 9 -o backup.cz documento.txt

# Comprimir nativamente a un archivo ZIP común
cszip compress -o archivo.zip carpeta_de_datos/

# Forzar compresión sobrescribiendo el destino y usando hash CRC-64
cszip compress -f --crc64 datos.bin
```

---

## 🔓 Descomprimir

El comando `decompress` (o `d`) identifica la firma del archivo de entrada y redirige automáticamente el procesamiento:

- **Archivos `.cz`**: Se descomprimen nativamente en base a sus bloques.
- **Archivos `.zip`**: Se extraen nativamente usando la librería integrada.
- **Archivos `.rar`**: Se extraen delegando la descompresión de forma segura en la utilidad `unrar` instalada en el sistema.

### Opciones de Descompresión

| Opción | Descripción | Valor por Defecto |
|--------|-------------|-------------------|
| `-o <ruta>` | Ruta/directorio del archivo o carpeta de salida | Remueve la extensión original |
| `-f` | Fuerza la sobrescritura de archivos existentes al extraer | — |
| `--no-verify` | Omite la validación de checksums (solo para formato `.cz`) | Verifica integridad |

### Ejemplos

```bash
# Descomprimir formato nativo .cz
cszip decompress datos.bin.cz

# Extraer un archivo .zip nativo a un directorio específico
cszip decompress backup.zip -o ./directorio_salida/

# Extraer un archivo .rar (ejecuta unrar en segundo plano)
cszip decompress paquete.rar -o ./directorio_salida/

# Descompresión rápida omitiendo la verificación de checksums Adler32/CRC
cszip decompress --no-verify datos.bin.cz
```

---

## 🔍 Inspección e Integridad (Solo para formato `.cz`)

Estas utilidades permiten auditar archivos `.cz` sin necesidad de descomprimirlos.

### Verificar Integridad
```bash
cszip verify datos.bin.cz
```
*Comprueba el magic number, Adler-32 de cada bloque, número total de bloques e integridad global contra el footer final del archivo.*

### Información del Archivo
```bash
cszip info datos.bin.cz
```
*Muestra información de cabecera como versión, algoritmo de compresión de bloques, tamaño de bloque log2, checksum global y número de bloques.*

### Listar Bloques (Detallado)
```bash
cszip list -v datos.bin.cz
```
*Imprime una tabla de bloques con sus respectivos tamaños original, comprimido y ratio de compresión.*
