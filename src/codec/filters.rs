//! Filtros de preprocesamiento para mejorar la compresión
//!
//! Los filtros transforman los datos antes de la compresión para hacerlos
//! más compresibles. Por ejemplo, el filtro delta es efectivo para datos
//! con valores incrementales.

use crate::error::{Error, ErrorKind, Result};

/// Tipos de filtro disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    /// Sin filtro
    None = 0,
    /// Filtro delta (diferencia entre bytes consecutivos)
    Delta = 1,
    /// Filtro sub (diferencia con byte anterior, para imágenes PNG)
    Sub = 2,
    /// Filtro up (diferencia con fila anterior)
    Up = 3,
    /// Filtro average (promedio de vecinos)
    Average = 4,
    /// Filtro Burrows-Wheeler Transform (mejora LZ77)
    Bwt = 5,
}

impl FilterType {
    /// Crear desde ID
    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::Delta),
            2 => Ok(Self::Sub),
            3 => Ok(Self::Up),
            4 => Ok(Self::Average),
            5 => Ok(Self::Bwt),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Tipo de filtro desconocido: {}", id),
            )),
        }
    }

    /// Obtener ID del filtro
    pub fn id(&self) -> u8 {
        *self as u8
    }
}

/// Trait para filtros
pub trait Filter {
    /// Aplicar filtro (preprocesamiento)
    fn apply(&self, data: &[u8]) -> Vec<u8>;
    
    /// Revertir filtro (postprocesamiento)
    fn revert(&self, data: &[u8]) -> Result<Vec<u8>>;
}

/// Filtro nulo (sin transformación)
pub struct NoneFilter;

impl Filter for NoneFilter {
    fn apply(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    fn revert(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

/// Filtro delta
///
/// Almacena la diferencia entre bytes consecutivos.
/// Efectivo para datos con cambios graduales.
pub struct DeltaFilter;

impl Filter for DeltaFilter {
    fn apply(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(data.len());
        output.push(data[0]);

        for i in 1..data.len() {
            output.push(data[i].wrapping_sub(data[i - 1]));
        }

        output
    }

    fn revert(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(data.len());
        output.push(data[0]);

        for i in 1..data.len() {
            output.push(data[i].wrapping_add(output[i - 1]));
        }

        Ok(output)
    }
}

/// Filtro Sub (PNG)
///
/// Similar a delta pero con interpretación de byte anterior.
pub struct SubFilter;

impl Filter for SubFilter {
    fn apply(&self, data: &[u8]) -> Vec<u8> {
        DeltaFilter.apply(data)
    }

    fn revert(&self, data: &[u8]) -> Result<Vec<u8>> {
        DeltaFilter.revert(data)
    }
}

/// Filtro Up para datos 2D
///
/// Requiere conocer el ancho de fila.
pub struct UpFilter {
    row_width: usize,
}

impl UpFilter {
    /// Crear filtro con ancho de fila específico
    pub fn new(row_width: usize) -> Self {
        Self { row_width }
    }
}

impl Filter for UpFilter {
    fn apply(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() || self.row_width == 0 {
            return data.to_vec();
        }

        let mut output = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            if i < self.row_width {
                output.push(byte);
            } else {
                output.push(byte.wrapping_sub(data[i - self.row_width]));
            }
        }

        output
    }

    fn revert(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() || self.row_width == 0 {
            return Ok(data.to_vec());
        }

        let mut output = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            if i < self.row_width {
                output.push(byte);
            } else {
                output.push(byte.wrapping_add(output[i - self.row_width]));
            }
        }

        Ok(output)
    }
}

/// Filtro Average
///
/// Usa el promedio de byte anterior y byte de fila anterior.
pub struct AverageFilter {
    row_width: usize,
}

impl AverageFilter {
    /// Crear filtro con ancho de fila
    pub fn new(row_width: usize) -> Self {
        Self { row_width }
    }
}

impl Filter for AverageFilter {
    fn apply(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() || self.row_width == 0 {
            return data.to_vec();
        }

        let mut output = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            let left = if i > 0 && i % self.row_width != 0 {
                data[i - 1] as u16
            } else {
                0
            };

            let up = if i >= self.row_width {
                data[i - self.row_width] as u16
            } else {
                0
            };

            let avg = ((left + up) / 2) as u8;
            output.push(byte.wrapping_sub(avg));
        }

        output
    }

    fn revert(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() || self.row_width == 0 {
            return Ok(data.to_vec());
        }

        let mut output = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            let left = if i > 0 && i % self.row_width != 0 {
                output[i - 1] as u16
            } else {
                0
            };

            let up = if i >= self.row_width {
                output[i - self.row_width] as u16
            } else {
                0
            };

            let avg = ((left + up) / 2) as u8;
            output.push(byte.wrapping_add(avg));
        }

        Ok(output)
    }
}

/// Move-to-front transform
///
/// Reorganiza símbolos basándose en recencia de uso.
/// Mejora compresión después de BWT.
pub struct MtfTransform;

impl MtfTransform {
    /// Aplicar MTF
    pub fn apply(data: &[u8]) -> Vec<u8> {
        let mut table: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(data.len());

        for &byte in data {
            let pos = table.iter().position(|&b| b == byte).unwrap();
            output.push(pos as u8);
            
            // Mover al frente
            table.remove(pos);
            table.insert(0, byte);
        }

        output
    }

    /// Revertir MTF
    pub fn revert(data: &[u8]) -> Vec<u8> {
        let mut table: Vec<u8> = (0..=255).collect();
        let mut output = Vec::with_capacity(data.len());

        for &pos in data {
            let byte = table[pos as usize];
            output.push(byte);
            
            // Mover al frente
            table.remove(pos as usize);
            table.insert(0, byte);
        }

        output
    }
}

/// Run-length encoding simple
///
/// Codifica secuencias repetidas.
pub struct RleEncoder;

impl RleEncoder {
    /// Codificar con RLE
    pub fn encode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let byte = data[i];
            let mut count = 1;

            while i + count < data.len() && data[i + count] == byte && count < 255 {
                count += 1;
            }

            if count >= 4 {
                // Escape + byte + count
                output.push(0xFF);
                output.push(byte);
                output.push(count as u8);
            } else {
                // Bytes literales
                for _ in 0..count {
                    if byte == 0xFF {
                        output.push(0xFF);
                        output.push(0xFF);
                        output.push(1);
                    } else {
                        output.push(byte);
                    }
                }
            }

            i += count;
        }

        output
    }

    /// Decodificar RLE
    pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0xFF {
                if i + 2 >= data.len() {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "RLE truncado"));
                }
                let byte = data[i + 1];
                let count = data[i + 2] as usize;
                output.extend(std::iter::repeat(byte).take(count));
                i += 3;
            } else {
                output.push(data[i]);
                i += 1;
            }
        }

        Ok(output)
    }
}

/// Crear filtro desde tipo
pub fn create_filter(filter_type: FilterType) -> Box<dyn Filter> {
    match filter_type {
        FilterType::None => Box::new(NoneFilter),
        FilterType::Delta | FilterType::Sub => Box::new(DeltaFilter),
        FilterType::Up => Box::new(UpFilter::new(0)),
        FilterType::Average => Box::new(AverageFilter::new(0)),
        FilterType::Bwt => Box::new(NoneFilter), // BWT requiere implementación separada
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_filter_roundtrip() {
        let filter = DeltaFilter;
        let input = vec![10, 12, 15, 14, 20, 25];
        
        let filtered = filter.apply(&input);
        let reverted = filter.revert(&filtered).unwrap();
        
        assert_eq!(reverted, input);
    }

    #[test]
    fn test_delta_filter_empty() {
        let filter = DeltaFilter;
        let filtered = filter.apply(&[]);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_delta_filter_sequential() {
        let filter = DeltaFilter;
        let input: Vec<u8> = (0..10).collect();
        
        let filtered = filter.apply(&input);
        // Después del primer byte, todos deberían ser 1
        assert_eq!(filtered[0], 0);
        for &b in &filtered[1..] {
            assert_eq!(b, 1);
        }
    }

    #[test]
    fn test_up_filter_roundtrip() {
        let filter = UpFilter::new(4);
        let input = vec![
            1, 2, 3, 4,
            2, 3, 4, 5,
            3, 4, 5, 6,
        ];
        
        let filtered = filter.apply(&input);
        let reverted = filter.revert(&filtered).unwrap();
        
        assert_eq!(reverted, input);
    }

    #[test]
    fn test_average_filter_roundtrip() {
        let filter = AverageFilter::new(4);
        let input = vec![
            10, 20, 30, 40,
            15, 25, 35, 45,
        ];
        
        let filtered = filter.apply(&input);
        let reverted = filter.revert(&filtered).unwrap();
        
        assert_eq!(reverted, input);
    }

    #[test]
    fn test_mtf_roundtrip() {
        let input = b"hello world";
        
        let transformed = MtfTransform::apply(input);
        let reverted = MtfTransform::revert(&transformed);
        
        assert_eq!(reverted, input);
    }

    #[test]
    fn test_mtf_repeated() {
        let input = vec![b'a'; 10];
        
        let transformed = MtfTransform::apply(&input);
        // Primer 'a' tiene índice alto, resto son 0
        assert_eq!(transformed[0], b'a' as u8);
        for &b in &transformed[1..] {
            assert_eq!(b, 0);
        }
    }

    #[test]
    fn test_rle_roundtrip() {
        let input = b"aaaaabbbbccc".to_vec();
        
        let encoded = RleEncoder::encode(&input);
        let decoded = RleEncoder::decode(&encoded).unwrap();
        
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_rle_no_repeats() {
        let input = b"abcdef".to_vec();
        
        let encoded = RleEncoder::encode(&input);
        let decoded = RleEncoder::decode(&encoded).unwrap();
        
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_rle_long_run() {
        let input = vec![0x42; 200];
        
        let encoded = RleEncoder::encode(&input);
        let decoded = RleEncoder::decode(&encoded).unwrap();
        
        assert_eq!(decoded, input);
        // Debería ser más corto que el original
        assert!(encoded.len() < input.len());
    }

    #[test]
    fn test_filter_type_from_id() {
        assert_eq!(FilterType::from_id(0).unwrap(), FilterType::None);
        assert_eq!(FilterType::from_id(1).unwrap(), FilterType::Delta);
        assert!(FilterType::from_id(99).is_err());
    }

    #[test]
    fn test_none_filter() {
        let filter = NoneFilter;
        let input = b"test data";
        
        let filtered = filter.apply(input);
        let reverted = filter.revert(&filtered).unwrap();
        
        assert_eq!(filtered, input);
        assert_eq!(reverted, input);
    }
}
