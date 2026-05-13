# 📋 Cheat Sheet - Formato CsZip

**Referencia rápida del formato binario (imprimir o tener a mano)**

---

## 🔍 Header Global (16 bytes)

```
Byte Offset  Tamaño  Campo               Tipo        Ejemplo
───────────  ──────  ─────────────────  ──────────  ──────────────
0-1          2       Magic              u16 BE      0x435A (CZ)
2            1       Version Major      u8          0x01
3            1       Version Minor      u8          0x00
4            1       Flags              u8          0x00
5            1       Compression Algo   u8          0x04 (DEFLATE)
6-7          2       Block Size Log2    u16 BE      0x0012 (2^18=256K)
8-9          2       Max Expansion %    u16 BE      0x03E8 (1000=10x)
10-11        2       Reserved           u16 BE      0x0000
12-15        4       Header Checksum    u32 BE      (TBD)
```

**Validación checklist:**

- [ ] Magic == 0x435A o 0x5A43
- [ ] Version Major <= 1
- [ ] Block Size Log2 ∈ [9, 16]
- [ ] Max Expansion ∈ [100, 5000]
- [ ] Flags bits 4-7 == 0

---

## 🧩 Block Header (12 bytes)

```
Byte Offset  Tamaño  Campo               Tipo        Rango
───────────  ──────  ──────────────────  ──────────  ────────────
0            1       Block Type         u8          0=DATA, 1=META, 2=INCOMP
1            1       Compression Level  u8          0-9
2-3          2       Original Size      u16 BE      1 - 65535
4-7          4       Compressed Size    u32 BE      0 - 2^31
8-11         4       ADLER-32           u32 BE      Hash original
```

**Validación checklist:**

- [ ] Block Type != 2
- [ ] Original Size > 0
- [ ] Original Size <= block_size (de header global)
- [ ] Compressed Size <= Original Size × Max Expansion / 100

---

## 📦 Estructura de Bloque Completo

```
Header Global (16 bytes)
│
├─ Block 0
│  ├─ Block Header (12 bytes)
│  ├─ Compressed Data (variable)
│  └─ CRC (4 u 8 bytes según flag)
│
├─ Block 1
│  ├─ Block Header (12 bytes)
│  ├─ Compressed Data (variable)
│  └─ CRC (4 u 8 bytes según flag)
│
└─ File Footer (12 bytes)
```

---

## 📄 File Footer (12 bytes)

```
Byte Offset  Tamaño  Campo               Tipo        Valor
───────────  ──────  ──────────────────  ──────────  ──────────
0            1       Marker             u8          0xFE
1-3          3       Num Blocks         u24 BE      0 - 16777215
4-7          4       Total Raw Size     u32 BE      suma de original
8-11         4       Footer Checksum    u32 BE      CRC-32 footer
```

---

## 🔐 Flags del Header Global (Byte 4)

```
Bit  Bit Offset  Nombre                  Descripción
───  ──────────  ──────────────────────  ────────────────────────
0    0x01        HAS_EXTRA_METADATA      Hay metadata tras footer
1    0x02        BLOCK_CRC_SIZE          1=CRC64, 0=CRC32
2    0x04        RESERVED_1              Reservado
3    0x08        RESERVED_2              Reservado
4-7  0xF0        (UNUSED)                Deben ser 0
```

**Ejemplos:**

- `0x00` = Sin metadata, CRC32
- `0x01` = Con metadata, CRC32
- `0x02` = Sin metadata, CRC64
- `0x03` = Con metadata, CRC64

---

## 🔤 Algoritmos de Compresión (Byte 5)

```
Valor  Nombre                  Descr
─────  ────────────────────────  ─────────────────────────────────
0      STORE                  Sin compresión (solo copiar)
1      LZ77_HUFFMAN          LZ77 + Huffman (recomendado)
2      LZ4_STYLE             Compresión rápida
3      LZMA_STYLE            Compresión muy fuerte
4      DEFLATE_STYLE         Compatible RFC 1951
5-14   RESERVED              Futuro
15     EXPERIMENTAL          No usar (error si se intenta)
```

---

## 📏 Tamaños Comunes

```
Tamaño Log2  Bytes              Descripción
──────────  ─────────────────  ─────────────────────
9           512 B              Muy pequeño
10          1 KiB              Pequeño
14          16 KiB             Bajo
16          64 KiB             Estándar bajo
18          256 KiB            RECOMENDADO ⭐
20          1 MiB              Grande
22          4 MiB              Muy grande
24          16 MiB             Extremo
30          1 GiB              Máximo
```

---

## 🔢 Valores de Expansión Comunes

```
Valor (u16 BE)  % Expansión  Significado
──────────────  ───────────  ───────────────────────────────
100             1x           100% (sin expansión)
200             2x           Máximo dobla tamaño
500             5x           Máximo 5x
1000            10x          RECOMENDADO ⭐ (máximo 10x)
2000            20x          Muy permisivo
5000            50x          Máximo permitido
```

---

## 🧮 Cálculo de CRC-32

```
Pseudocódigo:

crc = 0xFFFFFFFF
FOR EACH byte IN data:
    index = (crc XOR byte) AND 0xFF
    crc = (crc >> 8) XOR CRC32_TABLE[index]
RETURN crc XOR 0xFFFFFFFF
```

**Valores conocidos para verificación:**

- CRC32("") = 0x00000000
- CRC32("123456789") = 0xCBF43926
- CRC32("Hello, World!") = 0x3D083C89

---

## ⚠️ Códigos de Error

```
0x01  INVALID_MAGIC_NUMBER
0x02  UNSUPPORTED_VERSION
0x03  UNSUPPORTED_ALGORITHM
0x04  INVALID_BLOCK_SIZE
0x05  INVALID_EXPANSION_LIMIT
0x06  INVALID_BLOCK_TYPE
0x07  BLOCK_CRC_MISMATCH             ← ERROR CRÍTICO
0x08  COMPRESSION_BOMB_SUSPECTED
0x09  INCOMPLETE_BLOCK_FOUND
0x0A  CORRUPTED_BLOCK_HEADER
0x0B  CORRUPTED_FILE_FOOTER
0x0C  INVALID_ADLER32_CHECKSUM
0x0D  MEMORY_LIMIT_EXCEEDED
0x0E  UNEXPECTED_EOF                 ← EOF prematuro
0x0F  INVALID_COMPRESSION_LEVEL
0x10  RESERVED_FOR_FUTURE_USE
```

---

## 📊 Validación Rápida en Orden

```
1. Leer 16 bytes → Header Global
   ├─ Magic válido? (0x435A o 0x5A43)
   ├─ Version <= 1?
   ├─ Block Size Log2 ∈ [9,16]?
   ├─ Max Expansion ∈ [100,5000]?
   └─ Flags bits 4-7 = 0?

2. FOR EACH bloque:
   ├─ Leer 12 bytes → Block Header
   │  ├─ Block Type != 2?
   │  ├─ Original Size > 0?
   │  ├─ Original Size <= block_size?
   │  └─ Compressed Size está ok?
   │
   ├─ Leer Compressed Data bytes
   │
   ├─ Leer CRC (4 u 8 bytes)
   │
   └─ Calcular CRC actual
      └─ ¿Coinciden? Si NO → ERROR 0x07

3. Leer 12 bytes → File Footer
   ├─ Marker = 0xFE?
   ├─ Num Blocks coincide?
   ├─ Total Raw Size coincide?
   └─ Footer CRC válido? (WARNING si no)
```

---

## 🔄 Endianness

**SIEMPRE BIG-ENDIAN (Network Byte Order)**

```
Ejemplo: Valor 0x12345678

Little-Endian (x86):  78 56 34 12  (común en PC)
Big-Endian (CsZip):   12 34 56 78  (estándar en red)

En Rust:
u16::from_be_bytes([0x12, 0x34]) → 0x1234
u32::from_be_bytes([0x12, 0x34, 0x56, 0x78]) → 0x12345678

Siempre usa: from_be_bytes() y to_be_bytes()
```

---

## 📝 Ejemplo Hex Mínimo

```
Archivo más simple posible:
- Header: 16 bytes
- 1 Bloque STORE: 12 + 100 + 4 = 116 bytes
- Footer: 12 bytes
Total: 144 bytes

Hex (sin datos):
43 5A 01 00 00 04 00 12 03 E8 00 00 00 00
00 00 00 64 00 00 64 ... (100 bytes de datos)
... CRC32 (4 bytes)
FE 00 00 01 00 00 00 64 00 00 00 00
```

---

## 🎯 Quick Validation Checklist

```
□ Magic = 0x435A
□ Version = 1.0
□ Block Size Log2 = 18 (256 KB)
□ Max Expansion = 1000 (10x)
□ Algoritmo soportado
□ Flags válidos
□ Block Type = 0 o 1
□ Original Size > 0
□ Compressed Size reasonable
□ CRC32 o CRC64 válido
□ Footer Marker = 0xFE
□ Num Blocks coincide
□ Total Raw Size coincide
```

---

## 💾 Tamaños Típicos por Compresión

```
Datos          Compresión  Tamaño Final  CRC
───────────────  ──────────  ─────────────  ─────
100 bytes texto  STORE       100 + 28       4
100 bytes texto  DEFLATE     ~30-40 + 28    4
1 MB repos       STORE       1 MB + 28      4
1 MB repos       DEFLATE     ~300 KB + 28   4
1 MB aleatorio   STORE       1 MB + 28      4
1 MB aleatorio   DEFLATE     ~1 MB + 28     4
```

---

<div align="center">

**Imprimir esta página para referencia rápida**

Actualizada: 7 Febrero 2026

</div>
