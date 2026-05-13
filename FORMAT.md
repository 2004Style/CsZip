# 🔧 Especificación del Formato CsZip (.cz)

**Versión:** 1.0  
**Estado:** Especificación Formal  
**Última actualización:** 7 Febrero 2026

---

## 📋 Tabla de Contenidos

1. [Resumen Ejecutivo](#resumen-ejecutivo)
2. [Principios de Diseño](#principios-de-diseño)
3. [Estructura Global](#estructura-global)
4. [Header Global](#header-global)
5. [Estructura de Bloques](#estructura-de-bloques)
6. [Códigos de Error](#códigos-de-error)
7. [Ejemplos Prácticos](#ejemplos-prácticos)
8. [Compatibilidad](#compatibilidad)

---

## 📌 Resumen Ejecutivo

**CsZip** es un formato binario propietario para compresión sin pérdida diseñado con:

- ✅ **Validación estricta:** Todos los campos son validados antes de procesarse
- ✅ **Independencia de bloques:** Cada bloque es autónomo y recuperable
- ✅ **Integridad garantizada:** Checksums CRC en cada bloque
- ✅ **Streaming:** Descompresión sin cargar el archivo completo
- ✅ **Versionado:** Soporte para futuras evoluciones del formato

**Extensión de archivo:** `.cz`  
**Endianness:** Big-endian para todas las entidades multi-byte  
**Compresión:** Algoritmo configurable (recomendado: LZ77 + Huffman)

---

## 🎯 Principios de Diseño

### 1. Seguridad por defecto

- Validación de todos los headers antes de procesamiento
- Límites explícitos en memoria y expansión
- Checksums redundantes

### 2. Recuperabilidad

- Bloques independientes
- Cada bloque contiene todo lo necesario para descompresión
- Fallos en un bloque no afectan otros bloques

### 3. Eficiencia

- Mínimo overhead (headers pequeños)
- Streaming sin buffering completo
- Procesamiento paralelo posible

### 4. Extensibilidad

- Campo de versión para cambios futuros
- Flags para características adicionales
- Espacio reservado para evolución

---

## 🏗️ Estructura Global del Archivo

```
┌────────────────────────────────────────────────────────────┐
│  FILE HEADER (16+ bytes) - Validación y configuración      │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  BLOQUE 0 (variable bytes)                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Block Header (12+ bytes)                             │  │
│  │ Datos Comprimidos (variable)                         │  │
│  │ Block Checksum (4 o 8 bytes CRC)                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  BLOQUE 1 (variable bytes)                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Block Header (12+ bytes)                             │  │
│  │ Datos Comprimidos (variable)                         │  │
│  │ Block Checksum (4 o 8 bytes CRC)                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ... más bloques ...                                        │
│                                                              │
│  FILE FOOTER (8+ bytes) - Cierre e integridad              │
└────────────────────────────────────────────────────────────┘
```

---

## 📖 Header Global

Define la configuración del archivo completo.

### Estructura

```
Offset  Tamaño  Campo              Tipo        Rango/Valores          Descripción
──────  ──────  ─────────────────  ──────────  ─────────────────────  ──────────────────────────
0       2       Magic Number       uint16_be   0x435A (0x5A43 alt)   Firma "CZ" o alternativa
2       1       Version Major      uint8       0-255                  Versión mayor (actual: 1)
3       1       Version Minor      uint8       0-255                  Versión menor (actual: 0)
4       1       Flags              uint8       Bitmask                Ver tabla de flags
5       1       Compression Algo   uint8       0-15                   Algoritmo usado
6       2       Block Size Log2    uint16_be   9-16                   log₂ del tamaño de bloque (máx 64KB por u16)
8       2       Max Expansion %    uint16_be   100-5000               Máximo % expansión permitido
10      2       Reserved           uint16_be   0x0000                 Reservado para evolución
12      4       File Checksum      uint32_be   0x00000000 (TBD)      Checksum del header/footer
```

**Total:** 16 bytes fijos (puede extenderse con flags)

### Magic Number

```
Standard:    0x435A  → "CZ" en ASCII (C=0x43, Z=0x5A)
Alternative: 0x5A43  → "ZC" (para detectar endianness)

Importancia: Detección rápida de formato, prevención de falsos positivos
```

### Tabla de Flags (Byte 4)

```
Bit  Flag Name                Description
───  ──────────────────────  ─────────────────────────────────────
0    HAS_EXTRA_METADATA      Hay datos adicionales tras footer
1    BLOCK_CRC_SIZE          1=CRC64, 0=CRC32
2    RESERVED_FUTURE_1       Reservado
3    RESERVED_FUTURE_2       Reservado
4-7  (unused)                Deben ser 0 (para compatibilidad)
```

**Ejemplo:**

- `0x01` → Tiene metadata extra
- `0x02` → Usa CRC64
- `0x03` → Metadata + CRC64

### Algoritmos de Compresión (Byte 5)

```
Valor  Algoritmo           Descripción
─────  ──────────────────  ─────────────────────────────────────────
0      Almacenamiento      Sin compresión (solo headers + datos)
1      LZ77 + Huffman      Referencia básica
2      LZ4-style           Compresión rápida
3      LZMA-style          Compresión fuerte (lenta)
4      Deflate-style       Compatible DEFLATE (RFC 1951)
5-14   Reservados          Para evolución futura
15     Experimental        Uso experimental (no soportar)
```

### Tamaño de Bloque

```
log₂(Tamaño) = valor en bytes 6-7

Campo:    uint16_be (big-endian)
Válido:   9-16 (limitado por u16 en block header original_size)
Ejemplo:
  value=15 → 2^15 = 32768 bytes = 32 KiB (recomendado)
  value=16 → 2^16 = 65536 bytes = 64 KiB (máximo)
```

### Max Expansion

```
Porcentaje máximo al que un bloque puede expandirse

Valor:    uint16_be (big-endian)
Rango:    100-5000
Ejemplo:
  200 = máximo dobla de tamaño
  1000 = máximo 10x en tamaño

Validación:
  IF (bloque_comprimido_size > bloque_original_size * max_expansion / 100):
    ERROR: "Bloque hace explotar límite de expansión"
```

---

## 🧩 Estructura de Bloques

Cada bloque incluye su propio header, datos y checksum para independencia total.

### Block Header

```
Offset  Tamaño  Campo                  Tipo        Rango/Valores    Descripción
──────  ──────  ────────────────────────────────  ────────────────  ──────────────
0       1       Block Type             uint8       0-3              Tipo de bloque
1       1       Compression Level      uint8       0-9              Nivel de compresión usado
2       2       Original Data Size     uint16_be   1-65535          Bytes ANTES de comprimir
4       4       Compressed Data Size   uint32_be   0-2^31-1         Bytes DESPUÉS de comprimir
8       4       ADLER-32 Original      uint32_be   checksum         Hash de datos originales
```

**Total:** 12 bytes por bloque

### Block Types

```
Tipo  Nombre                 Descripción
────  ──────────────────────  ────────────────────────────────────────
0     DATA                   Bloque de datos comprimidos
1     METADATA_BLOCK         Bloque de metadata (ignorar si no es 0)
2     INCOMPLETE_BLOCK       Indica bloque incompleto (error)
3     RESERVED              Reservado para uso futuro
```

### Block Checksum

Ubicado al final del bloque, después de todos los datos comprimidos.

```
Tipo           Tamaño  Fórmula           Protege
────────────────────  ────────────────  ────────────────────────────────
CRC32          4      CRC-32 (ISO 3309) Datos comprimidos + block header
CRC64          8      CRC-64 (ECMA)     Datos comprimidos + block header

Orden: Big-endian
Cálculo: CRC(__data_comprimidos || __block_header) = checksum_value
```

---

## 📄 File Footer

Ubicado al final del archivo, después del último bloque.

```
Offset  Tamaño  Campo              Tipo        Descripción
──────  ──────  ─────────────────  ──────────  ────────────────────────
0       1       Footer Marker      uint8       0xFE (marca de inicio)
1       3       Number of Blocks   uint24_be   Cantidad total de bloques
4       4       Total RawSize      uint32_be   Suma de tamaños sin comprimir
8       4       Footer Checksum    uint32_be   CRC32 del footer completo
```

**Total:** 12 bytes fijos

**Propósito:**

- Validar arquitectura e integridad
- Permitir saltear bloques en lectura inversa (opcional)
- Verificación rápida de integridad global

---

## 🔍 Validación Estricta

### En Lectura de Header Global

```
1. Magic Number
   IF magic != 0x435A AND magic != 0x5A43:
     RETURN ERROR: "Magic number inválido"

2. Versión
   IF version_major > SOPORTADA:
     RETURN ERROR: "Versión no soportada"

3. Algoritmo
   IF algo == 15:
     RETURN ERROR: "Algoritmo experimental no soportado"
   IF algo > 14:
     RETURN ERROR: "Algoritmo desconocido"

4. Block Size
   IF block_size_log2 < 9 OR block_size_log2 > 16:
     RETURN ERROR: "Tamaño de bloque inválido"

5. Max Expansion
   IF max_expansion < 100 OR max_expansion > 5000:
     RETURN ERROR: "Máxima expansión fuera de rango"

6. Flags
   IF (flags & 0xF0) != 0:  // Bits 4-7 deben ser 0
     RETURN ERROR: "Flags inválidos para compatibilidad"
```

### En Lectura de Block Header

```
1. Block Type
   IF block_type > 3:
     RETURN ERROR: "Tipo de bloque inválido"
   IF block_type == 2:
     RETURN ERROR: "Bloque incompleto detectado"

2. Original Data Size
   block_size = 2^(file_header.block_size_log2)
   IF original_size == 0 OR original_size > block_size:
     RETURN ERROR: "Tamaño de datos inválido"

3. Compressed Size
   IF compressed_size == 0 OR compressed_size > block_size * max_expansion / 100:
     RETURN ERROR: "Tamaño comprimido violata límite"
```

### En Checksum

```
1. Block Checksum
   IF is_crc64_enabled:
     calculated = CRC64(block_data)
     IF calculated != stored_crc64:
       RETURN ERROR: "CRC64 mismatch en bloque"
   ELSE:
     calculated = CRC32(block_data)
     IF calculated != stored_crc32:
       RETURN ERROR: "CRC32 mismatch en bloque"

2. File Footer
   footer_checksum = CRC32(footer_content[0:8])
   IF footer_checksum != stored_footer_crc:
     RETURN WARNING: "Footer potencialmente corrupto"
```

---

## ⚠️ Códigos de Error

Errores críticos que detienen la descompresión:

```
Código  Nombre                              Descripción
──────  ────────────────────────────────────  ──────────────────────────────────
0x01    INVALID_MAGIC_NUMBER                Magic number no coincide
0x02    UNSUPPORTED_VERSION                 Versión del formato no soportada
0x03    UNSUPPORTED_ALGORITHM               Algoritmo de compresión no implementado
0x04    INVALID_BLOCK_SIZE                  Tamaño de bloque fuera de rango válido
0x05    INVALID_EXPANSION_LIMIT             Límite de expansión inválido
0x06    INVALID_BLOCK_TYPE                  Tipo de bloque desconocido
0x07    BLOCK_CRC_MISMATCH                  Checksum de bloque no coincide
0x08    COMPRESSION_BOMB_SUSPECTED          Ratio de expansión sospechoso
0x09    INCOMPLETE_BLOCK_FOUND              Bloque marcado como incompleto
0x0A    CORRUPTED_BLOCK_HEADER              Header de bloque corrompido
0x0B    CORRUPTED_FILE_FOOTER               Footer del archivo corrompido
0x0C    INVALID_ADLER32_CHECKSUM            Checksum de datos originales no coincide
0x0D    MEMORY_LIMIT_EXCEEDED               Sobrepasaría límite de memoria
0x0E    UNEXPECTED_EOF                      Fin de archivo inesperado
0x0F    INVALID_COMPRESSION_LEVEL           Nivel de compresión inválido
0x10    RESERVED_FOR_FUTURE_USE             Reservado (indica extensión no soportada)
```

---

## 💡 Ejemplos Prácticos

### Ejemplo 1: Archivo Simple Comprimido

```
Hex dump de un archivo de ejemplo (16 bytes + 50 bytes bloque + 12 bytes footer):

00000000:  435A 0100 0104 0012 0378 00FF FFFF FFFF
00000010:  FF00 4500 4500 1E00 0032 AABB CCDD 4142
00000020:  4344 4546 4748 4950 5152 5354 5556 5758
00000030:  5960 6162 6364 6566 6768 696A 6B6C 6D6E
00000040:  6F70 71FF 0003 0000 00FF CFBF CF33

Interpretación:
─────────────────────────────────────────────────────────────
Offset 00-01:  435A         → Magic Number (CZ)
Offset 02:     01           → Version Major (1)
Offset 03:     00           → Version Minor (0)
Offset 04:     01           → Flags (0x01 = HAS_EXTRA_METADATA)
Offset 05:     04           → Compression Algo (4 = Deflate-style)
Offset 06-07:  0012         → Block Size = 2^18 = 256 KiB
Offset 08-09:  0378         → Max Expansion = 888%
Offset 0A-0B:  00FF         → Reserved
Offset 0C-0F:  FFFF FFFF   → Header Checksum

Offset 10:     FF           → Block Type (data)
Offset 11:     00           → Compression Level (0)
Offset 12-13:  0045         → Original Size = 69 bytes
Offset 14-17:  00000032     → Compressed Size = 50 bytes
Offset 18-1B:  AABBCCDD     → ADLER-32
Offset 1C-4B:  [50 bytes]   → Datos comprimidos
Offset 4C-4F:  [CRC32]      → Block Checksum

Offset 50:     FF           → Footer Marker
Offset 51-53:  000001       → Número de bloques = 1
Offset 54-57:  00000045     → Total raw size = 69 bytes
Offset 58-5B:  CF BF CF 33  → Footer Checksum
```

### Ejemplo 2: Múltiples Bloques

```
Archivo de 1 MB, bloques de 256 KB:

Header (16 bytes)
├─ Bloque 0: 256 KB original → ~180 KB comprimido + 12 B header + 4 B CRC = ~180 KB
├─ Bloque 1: 256 KB original → ~175 KB comprimido + 12 B header + 4 B CRC = ~175 KB
├─ Bloque 2: 256 KB original → ~185 KB comprimido + 12 B header + 4 B CRC = ~185 KB
└─ Bloque 3: 256 KB original → ~190 KB comprimido + 12 B header + 4 B CRC = ~190 KB
└─ Footer (12 bytes)

Total: archivo 1MB → ~730 KB comprimido + 16 B header + 12 B footer = ~730 KB (73% compresión)
```

---

## 🔄 Flujo de Descompresión Segura

```
ENTRADA: archivo.cz
SALIDA:  archivo (datos originales)

1. LECTURA DE HEADER
   ├─ Leer 16 bytes
   ├─ Validar magic number
   ├─ Validar versión
   ├─ Validar flags
   ├─ Obtener algoritmo, tamaño bloque, max expansion
   └─ Calcular bloque_size = 2^(block_size_log2)

2. BUCLE PARA CADA BLOQUE
   ├─ Leer 12 bytes de block header
   ├─ Validar block type
   ├─ Validar original_size <= bloque_size
   ├─ Validar compressed_size <= (original_size * max_expansion / 100)
   │
   ├─ Leer compressed_size bytes de datos
   │
   ├─ Calcular checksum:
   │  ├─ Si CRC64: Calcular CRC64(block_header || data)
   │  └─ Si CRC32: Calcular CRC32(block_header || data)
   │
   ├─ Leer stored_checksum (4 u 8 bytes según flag)
   │
   ├─ Validar checksum
   │  └─ IF calculated != stored:
   │     → ERROR: "Bloque corrompido"
   │
   ├─ DESCOMPRIMIR (algoritmo específico)
   │  └─ output = decompress(data, original_size)
   │
   ├─ Validar tamaño: output.len() == original_size
   │
   ├─ Escribir output al archivo destino
   │
   └─ ¿Más bloques? → IR A PASO 2

3. LECTURA DE FOOTER
   ├─ Leer 12 bytes
   ├─ Validar footer marker (0xFF)
   ├─ Leer número de bloques
   ├─ Comparar con bloques leídos
   ├─ Leer total raw size
   ├─ Comparar con bytes escritos
   ├─ Validar footer checksum
   └─ IF TODO VÁLIDO: SUCCESS
      ELSE: WARNING (pero datos pueden ser válidos)

SALIDA:  archivo descomprimido
```

---

## 🛡️ Protecciones Contra Ataques

### Zip Bomb Detection

```
Un "zip bomb" es un archivo que al descomprimir consume
toda la memoria disponible.

PROTECCIÓN 1: Límite de Expansion
  MAX_EXPANSION = 10 (1000% = 10x) default

  IF compressed_size > original_size * 10:
    REJECT: "Archivo rechazado (potencial bomb)"

PROTECCIÓN 2: Límite de Memoria
  MAX_MEMORY = 100 MB (configurable)

  total_uncompressed = 0
  FOR EACH bloque:
    total_uncompressed += original_size
    IF total_uncompressed > MAX_MEMORY:
      REJECT: "Límite de memoria excedido"

PROTECCIÓN 3: Bloque Incompleto
  IF block.type == INCOMPLETE_BLOCK:
    REJECT: "Bloque incompleto detectado"
```

### Desbordamiento de Buffer

```
PROTECCIÓN: Validación estricta de tamaños

PARA CADA campo de tamaño:
  1. Verificar no es 0 (archivo vacío es válido)
  2. Verificar no excede límite físico (2^31-1)
  3. Verificar no excede bloque_size
  4. Verificar cumple reglas de compresión

NUNCA:
  - Leer sin validar tamaño primero
  - Asignar memoria sin límite
  - Asumir datos contiguos
```

---

## 📊 Compatibilidad

### Versiones

```
Versión Actual: 1.0
Lectura:  Soporta 0.x (con warnings) y 1.x
Escritura: Solo 1.0

Future Versioning:
  v1.0 → v1.1: Cambios menores (compatible)
  v1.x → v2.0: Cambios mayores (nueva implementación)
```

### Endianness

**Todo el formato usa Big-Endian (Network Byte Order):**

```
Little-Endian (x86):    0x12345678
Big-Endian (Descarga):  0x78563412

CsZip: SIEMPRE Big-Endian en bytes 0-1

Ejemplo:
  block_size_log2 = 18
  En memoria: 0x0012
  En archivo: (byte0=0x00, byte1=0x12)
```

### Plataformas Soportadas

- ✅ Linux (x86-64, ARM, PowerPC)
- ✅ Windows (x86-64)
- ✅ macOS (x86-64, Apple Silicon)
- ✅ Cualquier platform con Rust 1.70+

---

## 📚 Referencias

- [CRC-32 (ISO 3309)](https://en.wikipedia.org/wiki/Cyclic_redundancy_check)
- [CRC-64 (ECMA)](https://en.wikipedia.org/wiki/CRC-64#ECMA)
- [ADLER-32 (RFC 1950)](https://tools.ietf.org/html/rfc1950)
- [Big-Endian vs Little-Endian](https://en.wikipedia.org/wiki/Endianness)
- [Zip Bomb (Security)](https://en.wikipedia.org/wiki/Zip_bomb)

---

<div align="center">

**Formato CsZip 1.0 — Especificación Formal Completa**

Próximas extensiones consideradas: SIMD, Diccionarios, Multithreading

</div>
