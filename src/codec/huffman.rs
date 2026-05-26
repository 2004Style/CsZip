//! Implementación de codificación Huffman
//!
//! La codificación Huffman es un algoritmo de compresión sin pérdida que asigna
//! códigos de longitud variable basados en la frecuencia de los símbolos.
//! Símbolos más frecuentes obtienen códigos más cortos.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::error::{Error, ErrorKind, Result};

/// Máximo número de símbolos (256 bytes + marcador EOF)
const MAX_SYMBOLS: usize = 257;

/// Símbolo EOF para marcar fin de datos
const EOF_SYMBOL: u16 = 256;

/// Nodo del árbol Huffman
#[derive(Debug, Clone)]
struct HuffmanNode {
    /// Frecuencia del nodo (suma de hijos para nodos internos)
    frequency: u64,
    /// Símbolo (solo para hojas)
    symbol: Option<u16>,
    /// Hijo izquierdo (0)
    left: Option<Box<HuffmanNode>>,
    /// Hijo derecho (1)
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    /// Crear nodo hoja
    fn leaf(symbol: u16, frequency: u64) -> Self {
        Self {
            frequency,
            symbol: Some(symbol),
            left: None,
            right: None,
        }
    }

    /// Crear nodo interno
    fn internal(left: HuffmanNode, right: HuffmanNode) -> Self {
        Self {
            frequency: left.frequency + right.frequency,
            symbol: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    /// Verificar si es hoja
    fn is_leaf(&self) -> bool {
        self.symbol.is_some()
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}

impl Eq for HuffmanNode {}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invertir orden para que la cola sea min-heap
        other.frequency.cmp(&self.frequency)
    }
}

/// Código Huffman para un símbolo
#[derive(Debug, Clone)]
pub struct HuffmanCode {
    /// Bits del código (los primeros `length` bits son válidos)
    pub bits: u32,
    /// Longitud del código en bits
    pub length: u8,
}

impl HuffmanCode {
    /// Crear nuevo código
    fn new(bits: u32, length: u8) -> Self {
        Self { bits, length }
    }
}

/// Tabla de códigos Huffman
pub type CodeTable = HashMap<u16, HuffmanCode>;

/// Construir árbol Huffman desde frecuencias
fn build_tree(frequencies: &[u64; MAX_SYMBOLS]) -> Option<HuffmanNode> {
    let mut heap = BinaryHeap::new();

    // Añadir todos los símbolos con frecuencia > 0
    for (symbol, &freq) in frequencies.iter().enumerate() {
        if freq > 0 {
            heap.push(HuffmanNode::leaf(symbol as u16, freq));
        }
    }

    // Si no hay símbolos, retornar None
    if heap.is_empty() {
        return None;
    }

    // Si solo hay un símbolo, crear árbol mínimo
    if heap.len() == 1 {
        let node = heap.pop().unwrap();
        return Some(HuffmanNode::internal(
            node,
            HuffmanNode::leaf(EOF_SYMBOL, 1),
        ));
    }

    // Construir árbol combinando nodos
    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();
        heap.push(HuffmanNode::internal(left, right));
    }

    heap.pop()
}

/// Generar tabla de códigos desde el árbol
fn generate_codes(node: &HuffmanNode, code: u32, length: u8, table: &mut CodeTable) {
    if let Some(symbol) = node.symbol {
        table.insert(symbol, HuffmanCode::new(code, length.max(1)));
    } else {
        if let Some(ref left) = node.left {
            generate_codes(left, code << 1, length + 1, table);
        }
        if let Some(ref right) = node.right {
            generate_codes(right, (code << 1) | 1, length + 1, table);
        }
    }
}

/// Escritor de bits para codificación
struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_position: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            bit_position: 0,
        }
    }

    /// Escribir bits
    fn write_bits(&mut self, bits: u32, count: u8) {
        for i in (0..count).rev() {
            let bit = ((bits >> i) & 1) as u8;
            self.current_byte |= bit << (7 - self.bit_position);
            self.bit_position += 1;

            if self.bit_position == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_position = 0;
            }
        }
    }

    /// Finalizar y obtener bytes
    fn finish(mut self) -> Vec<u8> {
        if self.bit_position > 0 {
            self.buffer.push(self.current_byte);
        }
        self.buffer
    }
}

/// Lector de bits para decodificación
struct BitReader<'a> {
    data: &'a [u8],
    byte_index: usize,
    bit_position: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_index: 0,
            bit_position: 0,
        }
    }

    /// Leer un bit
    fn read_bit(&mut self) -> Option<u8> {
        if self.byte_index >= self.data.len() {
            return None;
        }

        let bit = (self.data[self.byte_index] >> (7 - self.bit_position)) & 1;
        self.bit_position += 1;

        if self.bit_position == 8 {
            self.byte_index += 1;
            self.bit_position = 0;
        }

        Some(bit)
    }
}

/// Codificador Huffman
pub struct HuffmanEncoder {
    /// Tabla de frecuencias
    frequencies: [u64; MAX_SYMBOLS],
}

impl HuffmanEncoder {
    /// Crear nuevo codificador
    pub fn new() -> Self {
        Self {
            frequencies: [0; MAX_SYMBOLS],
        }
    }

    /// Calcular frecuencias de los datos
    fn calculate_frequencies(&mut self, data: &[u8]) {
        self.frequencies = [0; MAX_SYMBOLS];
        for &byte in data {
            self.frequencies[byte as usize] += 1;
        }
        // Añadir marcador EOF
        self.frequencies[EOF_SYMBOL as usize] = 1;
    }

    /// Serializar tabla de frecuencias
    fn serialize_frequencies(&self) -> Vec<u8> {
        let mut output = Vec::new();

        // Contar símbolos con frecuencia > 0
        let count: u16 = self.frequencies.iter().filter(|&&f| f > 0).count() as u16;

        output.extend_from_slice(&count.to_be_bytes());

        // Escribir cada símbolo y su frecuencia
        for (symbol, &freq) in self.frequencies.iter().enumerate() {
            if freq > 0 {
                output.extend_from_slice(&(symbol as u16).to_be_bytes());
                output.extend_from_slice(&freq.to_be_bytes());
            }
        }

        output
    }

    /// Codificar datos
    pub fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Calcular frecuencias
        self.calculate_frequencies(data);

        // Construir árbol
        let tree = build_tree(&self.frequencies).ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "No se pudo construir árbol Huffman")
        })?;

        // Generar códigos
        let mut code_table = CodeTable::new();
        generate_codes(&tree, 0, 0, &mut code_table);

        // Serializar frecuencias (para reconstruir árbol en decodificación)
        let mut output = self.serialize_frequencies();

        // Escribir tamaño original
        output.extend_from_slice(&(data.len() as u32).to_be_bytes());

        // Codificar datos
        let mut writer = BitWriter::new();
        for &byte in data {
            let code = code_table
                .get(&(byte as u16))
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Símbolo sin código"))?;
            writer.write_bits(code.bits, code.length);
        }

        // Añadir EOF
        if let Some(eof_code) = code_table.get(&EOF_SYMBOL) {
            writer.write_bits(eof_code.bits, eof_code.length);
        }

        output.extend(writer.finish());
        Ok(output)
    }
}

impl Default for HuffmanEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodificador Huffman
pub struct HuffmanDecoder;

impl HuffmanDecoder {
    /// Crear nuevo decodificador
    pub fn new() -> Self {
        Self
    }

    /// Deserializar frecuencias
    fn deserialize_frequencies(data: &[u8]) -> Result<([u64; MAX_SYMBOLS], usize)> {
        if data.len() < 2 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Datos Huffman truncados",
            ));
        }

        let count = u16::from_be_bytes([data[0], data[1]]) as usize;
        let mut pos = 2;
        let mut frequencies = [0u64; MAX_SYMBOLS];

        for _ in 0..count {
            if pos + 10 > data.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Tabla de frecuencias truncada",
                ));
            }
            let symbol = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            let freq = u64::from_be_bytes([
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
                data[pos + 8],
                data[pos + 9],
            ]);

            if symbol < MAX_SYMBOLS {
                frequencies[symbol] = freq;
            }
            pos += 10;
        }

        Ok((frequencies, pos))
    }

    /// Decodificar datos
    pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Deserializar frecuencias
        let (frequencies, pos) = Self::deserialize_frequencies(data)?;

        if pos + 4 > data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Tamaño original faltante",
            ));
        }

        // Leer tamaño original
        let original_size =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;

        let encoded_data = &data[pos + 4..];

        // Reconstruir árbol
        let tree = build_tree(&frequencies)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "No se pudo reconstruir árbol"))?;

        // Decodificar
        let mut output = Vec::with_capacity(original_size);
        let mut reader = BitReader::new(encoded_data);
        let mut current = &tree;

        loop {
            if current.is_leaf() {
                let symbol = current.symbol.unwrap();
                if symbol == EOF_SYMBOL {
                    break;
                }
                if symbol < 256 {
                    output.push(symbol as u8);
                }
                current = &tree;

                if output.len() >= original_size {
                    break;
                }
            } else {
                match reader.read_bit() {
                    Some(0) => {
                        current = current
                            .left
                            .as_ref()
                            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Árbol corrupto"))?;
                    }
                    Some(1) => {
                        current = current
                            .right
                            .as_ref()
                            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Árbol corrupto"))?;
                    }
                    Some(_) => {
                        // Bit inválido (solo debería ser 0 o 1)
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Bit inválido en datos Huffman",
                        ));
                    }
                    None => {
                        break;
                    }
                }
            }
        }

        Ok(output)
    }
}

impl Default for HuffmanDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        let mut encoder = HuffmanEncoder::new();
        let result = encoder.encode(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_roundtrip_simple() {
        let mut encoder = HuffmanEncoder::new();
        let input = b"hello";

        let encoded = encoder.encode(input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
    }

    #[test]
    fn test_roundtrip_single_char() {
        let mut encoder = HuffmanEncoder::new();
        let input = b"aaaaaaa";

        let encoded = encoder.encode(input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
    }

    #[test]
    fn test_roundtrip_all_bytes() {
        let mut encoder = HuffmanEncoder::new();
        let input: Vec<u8> = (0..=255).collect();

        let encoded = encoder.encode(&input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
    }

    #[test]
    fn test_roundtrip_repeated() {
        let mut encoder = HuffmanEncoder::new();
        let input = b"abcabcabcabc";

        let encoded = encoder.encode(input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
    }

    #[test]
    fn test_compression_skewed() {
        let mut encoder = HuffmanEncoder::new();
        // Datos con frecuencia muy desigual - deberían comprimir bien
        let input: Vec<u8> = std::iter::repeat(b'a')
            .take(100)
            .chain(std::iter::repeat(b'b').take(10))
            .chain(std::iter::repeat(b'c').take(1))
            .collect();

        let encoded = encoder.encode(&input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
        // La tabla de frecuencias añade overhead, así que para datos pequeños
        // puede no haber compresión efectiva
    }

    #[test]
    fn test_bit_writer() {
        let mut writer = BitWriter::new();
        writer.write_bits(0b101, 3);
        writer.write_bits(0b1110, 4);
        writer.write_bits(0b1, 1);

        let bytes = writer.finish();
        assert_eq!(bytes, vec![0b10111101]);
    }

    #[test]
    fn test_bit_reader() {
        let data = vec![0b10110100];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bit(), Some(1));
        assert_eq!(reader.read_bit(), Some(0));
        assert_eq!(reader.read_bit(), Some(1));
        assert_eq!(reader.read_bit(), Some(1));
        assert_eq!(reader.read_bit(), Some(0));
        assert_eq!(reader.read_bit(), Some(1));
        assert_eq!(reader.read_bit(), Some(0));
        assert_eq!(reader.read_bit(), Some(0));
        assert_eq!(reader.read_bit(), None);
    }

    #[test]
    fn test_large_data() {
        let mut encoder = HuffmanEncoder::new();
        let input: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        let encoded = encoder.encode(&input).unwrap();
        let decoded = HuffmanDecoder::decode(&encoded).unwrap();

        assert_eq!(decoded, input);
    }
}
