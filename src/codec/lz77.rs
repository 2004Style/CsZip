//! Implementación del algoritmo LZ77
//!
//! LZ77 es un algoritmo de compresión basado en diccionario que encuentra
//! secuencias repetidas en una ventana deslizante.
//!
//! # Funcionamiento
//!
//! El algoritmo mantiene una ventana de búsqueda (search buffer) y una ventana
//! de lookahead. Para cada posición, busca la coincidencia más larga en el
//! search buffer y la codifica como (distancia, longitud) o como literal.

use crate::error::{Error, ErrorKind, Result};

/// Configuración del compresor LZ77
#[derive(Debug, Clone, Copy)]
pub struct Lz77Config {
    /// Tamaño de la ventana de búsqueda (máximo 32KB para compatibilidad)
    pub window_size: usize,
    /// Longitud mínima para considerar un match
    pub min_match_length: usize,
    /// Longitud máxima de un match
    pub max_match_length: usize,
    /// Nivel de esfuerzo de búsqueda (1-9)
    pub search_depth: usize,
}

impl Default for Lz77Config {
    fn default() -> Self {
        Self {
            window_size: 32768,    // 32 KB
            min_match_length: 3,   // Mínimo 3 bytes para match
            max_match_length: 258, // Compatible con DEFLATE
            search_depth: 64,      // Búsqueda moderada
        }
    }
}

impl Lz77Config {
    /// Crear configuración para nivel de compresión específico
    pub fn for_level(level: u8) -> Self {
        let level = level.clamp(1, 9);

        Self {
            window_size: match level {
                1..=3 => 4096,
                4..=6 => 16384,
                _ => 32768,
            },
            min_match_length: 3,
            max_match_length: 258,
            search_depth: match level {
                1 => 4,
                2 => 8,
                3 => 16,
                4 => 32,
                5 => 64,
                6 => 128,
                7 => 256,
                8 => 512,
                _ => 1024,
            },
        }
    }
}

/// Token LZ77 - puede ser un literal o una referencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lz77Token {
    /// Byte literal (no se encontró match)
    Literal(u8),
    /// Match: (distancia hacia atrás, longitud)
    Match {
        /// Distancia hacia atrás en el buffer
        distance: u16,
        /// Longitud del match
        length: u16,
    },
}

impl Lz77Token {
    /// Verifica si es un literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Lz77Token::Literal(_))
    }

    /// Verifica si es un match
    pub fn is_match(&self) -> bool {
        matches!(self, Lz77Token::Match { .. })
    }
}

/// Compresor LZ77
pub struct Lz77Compressor {
    config: Lz77Config,
}

impl Lz77Compressor {
    /// Crear nuevo compresor con configuración por defecto
    pub fn new() -> Self {
        Self {
            config: Lz77Config::default(),
        }
    }

    /// Crear compresor con configuración personalizada
    pub fn with_config(config: Lz77Config) -> Self {
        Self { config }
    }

    /// Crear compresor para nivel de compresión específico
    pub fn with_level(level: u8) -> Self {
        Self {
            config: Lz77Config::for_level(level),
        }
    }

    /// Comprimir datos y retornar tokens
    pub fn compress(&self, input: &[u8]) -> Vec<Lz77Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            // Buscar el mejor match
            if let Some((distance, length)) = self.find_best_match(input, pos) {
                tokens.push(Lz77Token::Match {
                    distance: distance as u16,
                    length: length as u16,
                });
                pos += length;
            } else {
                tokens.push(Lz77Token::Literal(input[pos]));
                pos += 1;
            }
        }

        tokens
    }

    /// Buscar el mejor match en la ventana de búsqueda
    fn find_best_match(&self, input: &[u8], pos: usize) -> Option<(usize, usize)> {
        if pos < 1 || pos + self.config.min_match_length > input.len() {
            return None;
        }

        let window_start = pos.saturating_sub(self.config.window_size);
        let max_len = (input.len() - pos).min(self.config.max_match_length);

        if max_len < self.config.min_match_length {
            return None;
        }

        let mut best_distance = 0;
        let mut best_length = 0;
        // Buscar hacia atrás en la ventana
        for (searches, search_pos) in (window_start..pos).rev().enumerate() {
            if searches >= self.config.search_depth {
                break;
            }

            let length = self.match_length(input, search_pos, pos, max_len);

            if length >= self.config.min_match_length && length > best_length {
                best_distance = pos - search_pos;
                best_length = length;

                // Match perfecto
                if length == max_len {
                    break;
                }
            }
        }

        if best_length >= self.config.min_match_length {
            Some((best_distance, best_length))
        } else {
            None
        }
    }

    /// Calcular longitud del match entre dos posiciones
    fn match_length(
        &self,
        input: &[u8],
        match_pos: usize,
        current_pos: usize,
        max_len: usize,
    ) -> usize {
        let mut length = 0;

        while length < max_len
            && match_pos + length < current_pos  // No sobrepasar posición actual
            && input.get(match_pos + length) == input.get(current_pos + length)
        {
            length += 1;
        }

        // También permitir matches que se extienden más allá (run-length)
        while length < max_len && current_pos + length < input.len() {
            let match_byte = input[match_pos + (length % (current_pos - match_pos))];
            if input[current_pos + length] == match_byte {
                length += 1;
            } else {
                break;
            }
        }

        length
    }

    /// Codificar tokens a bytes
    pub fn encode_tokens(&self, tokens: &[Lz77Token]) -> Vec<u8> {
        let mut output = Vec::new();

        for token in tokens {
            match token {
                Lz77Token::Literal(byte) => {
                    // Flag 0 = literal, seguido del byte
                    output.push(0x00);
                    output.push(*byte);
                }
                Lz77Token::Match { distance, length } => {
                    // Flag 1 = match, seguido de distancia (2 bytes) y longitud (2 bytes)
                    output.push(0x01);
                    output.extend_from_slice(&distance.to_be_bytes());
                    output.extend_from_slice(&length.to_be_bytes());
                }
            }
        }

        output
    }

    /// Comprimir datos directamente a bytes
    pub fn compress_to_bytes(&self, input: &[u8]) -> Vec<u8> {
        let tokens = self.compress(input);
        self.encode_tokens(&tokens)
    }
}

impl Default for Lz77Compressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Descompresor LZ77
pub struct Lz77Decompressor;

impl Lz77Decompressor {
    /// Crear nuevo descompresor
    pub fn new() -> Self {
        Self
    }

    /// Decodificar bytes a tokens
    pub fn decode_tokens(input: &[u8]) -> Result<Vec<Lz77Token>> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            let flag = input[pos];
            pos += 1;

            match flag {
                0x00 => {
                    // Literal
                    if pos >= input.len() {
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "EOF inesperado leyendo literal LZ77",
                        ));
                    }
                    tokens.push(Lz77Token::Literal(input[pos]));
                    pos += 1;
                }
                0x01 => {
                    // Match
                    if pos + 4 > input.len() {
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "EOF inesperado leyendo match LZ77",
                        ));
                    }
                    let distance = u16::from_be_bytes([input[pos], input[pos + 1]]);
                    let length = u16::from_be_bytes([input[pos + 2], input[pos + 3]]);
                    tokens.push(Lz77Token::Match { distance, length });
                    pos += 4;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Flag LZ77 inválido: 0x{:02X}", flag),
                    ));
                }
            }
        }

        Ok(tokens)
    }

    /// Descomprimir tokens a datos originales
    pub fn decompress_tokens(tokens: &[Lz77Token]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        for token in tokens {
            match token {
                Lz77Token::Literal(byte) => {
                    output.push(*byte);
                }
                Lz77Token::Match { distance, length } => {
                    let distance = *distance as usize;
                    let length = *length as usize;

                    if distance > output.len() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Distancia LZ77 inválida: {} > {}", distance, output.len()),
                        ));
                    }

                    let start = output.len() - distance;
                    for i in 0..length {
                        let byte = output[start + (i % distance)];
                        output.push(byte);
                    }
                }
            }
        }

        Ok(output)
    }

    /// Descomprimir bytes directamente
    pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
        let tokens = Self::decode_tokens(input)?;
        Self::decompress_tokens(&tokens)
    }
}

impl Default for Lz77Decompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_empty() {
        let compressor = Lz77Compressor::new();
        let tokens = compressor.compress(&[]);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_compress_single_byte() {
        let compressor = Lz77Compressor::new();
        let tokens = compressor.compress(&[0x42]);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Lz77Token::Literal(0x42));
    }

    #[test]
    fn test_compress_no_repeats() {
        let compressor = Lz77Compressor::new();
        let input = b"abcdefgh";
        let tokens = compressor.compress(input);

        // Sin repeticiones, todo son literales
        assert_eq!(tokens.len(), input.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, Lz77Token::Literal(input[i]));
        }
    }

    #[test]
    fn test_compress_simple_repeat() {
        let compressor = Lz77Compressor::new();
        let input = b"abcabcabc";
        let tokens = compressor.compress(input);

        // Debería encontrar matches
        let has_match = tokens.iter().any(|t| t.is_match());
        assert!(has_match, "Debería detectar repeticiones");
    }

    #[test]
    fn test_compress_long_repeat() {
        let compressor = Lz77Compressor::new();
        let input = b"aaaaaaaaaaaaaaaa"; // 16 'a's
        let tokens = compressor.compress(input);

        // Debería ser más corto que el original
        assert!(tokens.len() < input.len());
    }

    #[test]
    fn test_roundtrip_simple() {
        let compressor = Lz77Compressor::new();
        let input = b"Hello, World!";

        let tokens = compressor.compress(input);
        let output = Lz77Decompressor::decompress_tokens(&tokens).unwrap();

        assert_eq!(output, input);
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let compressor = Lz77Compressor::new();
        let input = b"abcabcabcabcabc";

        let tokens = compressor.compress(input);
        let output = Lz77Decompressor::decompress_tokens(&tokens).unwrap();

        assert_eq!(output, input);
    }

    #[test]
    fn test_roundtrip_bytes() {
        let compressor = Lz77Compressor::new();
        let input = b"test data test data test";

        let compressed = compressor.compress_to_bytes(input);
        let decompressed = Lz77Decompressor::decompress(&compressed).unwrap();

        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_roundtrip_all_bytes() {
        let compressor = Lz77Compressor::new();
        let input: Vec<u8> = (0..=255).collect();

        let tokens = compressor.compress(&input);
        let output = Lz77Decompressor::decompress_tokens(&tokens).unwrap();

        assert_eq!(output, input);
    }

    #[test]
    fn test_config_levels() {
        for level in 1..=9 {
            let config = Lz77Config::for_level(level);
            assert!(config.window_size > 0);
            assert!(config.search_depth > 0);
            assert!(config.min_match_length >= 3);
        }
    }

    #[test]
    fn test_decode_invalid_flag() {
        let input = vec![0xFF, 0x00];
        let result = Lz77Decompressor::decode_tokens(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated() {
        let input = vec![0x01, 0x00]; // Match incompleto
        let result = Lz77Decompressor::decode_tokens(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_invalid_distance() {
        let tokens = vec![
            Lz77Token::Literal(b'a'),
            Lz77Token::Match {
                distance: 100,
                length: 5,
            }, // Distancia inválida
        ];
        let result = Lz77Decompressor::decompress_tokens(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_compression_effectiveness() {
        let compressor = Lz77Compressor::with_level(6);

        // Datos muy repetitivos
        let input: Vec<u8> = vec![0xAA; 1000];
        let compressed = compressor.compress_to_bytes(&input);

        // Debería comprimir significativamente
        assert!(compressed.len() < input.len() / 2);
    }
}
