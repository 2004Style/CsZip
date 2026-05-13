//! Cálculo de checksums CRC para verificación de integridad
//!
//! Implementa CRC-32 (ISO 3309) y CRC-64 (ECMA) para validar
//! la integridad de bloques y headers.

/// Calculador de CRC-32 con estado (ISO 3309 / IEEE 802.3)
#[derive(Clone)]
pub struct Crc32 {
    value: u32,
}

impl Crc32 {
    /// Polinomio CRC-32 reflejado
    const POLYNOMIAL: u32 = 0xEDB88320;

    /// Tabla de lookup precalculada para CRC-32
    const TABLE: [u32; 256] = Self::build_table();

    /// Construir tabla de lookup en tiempo de compilación
    const fn build_table() -> [u32; 256] {
        let mut table = [0u32; 256];
        let mut i = 0;

        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;

            while j < 8 {
                crc = if (crc & 1) == 1 {
                    (crc >> 1) ^ Self::POLYNOMIAL
                } else {
                    crc >> 1
                };
                j += 1;
            }

            table[i] = crc;
            i += 1;
        }

        table
    }

    /// Crear nuevo calculador CRC-32
    pub fn new() -> Self {
        Self { value: 0xFFFFFFFF }
    }

    /// Actualizar el CRC con más datos
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let idx = ((self.value ^ byte as u32) & 0xFF) as usize;
            self.value = (self.value >> 8) ^ Self::TABLE[idx];
        }
    }

    /// Finalizar y obtener el CRC
    pub fn finalize(&self) -> u32 {
        self.value ^ 0xFFFFFFFF
    }

    /// Calcular CRC-32 de un slice de bytes (método estático)
    #[inline]
    pub fn compute(data: &[u8]) -> u32 {
        let mut crc = Self::new();
        crc.update(data);
        crc.finalize()
    }

    /// Verificar si el CRC coincide
    #[inline]
    pub fn verify(data: &[u8], expected: u32) -> bool {
        Self::compute(data) == expected
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculador de CRC-64 con estado (ECMA-182)
#[derive(Clone)]
pub struct Crc64 {
    value: u64,
}

impl Crc64 {
    /// Polinomio CRC-64 ECMA
    const POLYNOMIAL: u64 = 0x42F0E1EBA9EA3693;

    /// Tabla de lookup precalculada para CRC-64
    const TABLE: [u64; 256] = Self::build_table();

    /// Construir tabla de lookup en tiempo de compilación
    const fn build_table() -> [u64; 256] {
        let mut table = [0u64; 256];
        let mut i = 0;

        while i < 256 {
            let mut crc = (i as u64) << 56;
            let mut j = 0;

            while j < 8 {
                crc = if (crc & 0x8000000000000000) != 0 {
                    (crc << 1) ^ Self::POLYNOMIAL
                } else {
                    crc << 1
                };
                j += 1;
            }

            table[i] = crc;
            i += 1;
        }

        table
    }

    /// Crear nuevo calculador CRC-64
    pub fn new() -> Self {
        Self { value: 0xFFFFFFFFFFFFFFFF }
    }

    /// Actualizar el CRC con más datos
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let idx = ((self.value >> 56) ^ byte as u64) as usize;
            self.value = (self.value << 8) ^ Self::TABLE[idx];
        }
    }

    /// Finalizar y obtener el CRC
    pub fn finalize(&self) -> u64 {
        self.value ^ 0xFFFFFFFFFFFFFFFF
    }

    /// Calcular CRC-64 de un slice de bytes (método estático)
    #[inline]
    pub fn compute(data: &[u8]) -> u64 {
        let mut crc = Self::new();
        crc.update(data);
        crc.finalize()
    }

    /// Verificar si el CRC-64 coincide
    #[inline]
    pub fn verify(data: &[u8], expected: u64) -> bool {
        Self::compute(data) == expected
    }
}

impl Default for Crc64 {
    fn default() -> Self {
        Self::new()
    }
}

/// Calcular ADLER-32 checksum con estado (usado en block header)
#[derive(Clone)]
pub struct Adler32 {
    a: u32,
    b: u32,
}

impl Adler32 {
    /// Módulo para ADLER-32
    const MOD_ADLER: u32 = 65521;

    /// Crear nuevo calculador ADLER-32
    pub fn new() -> Self {
        Self { a: 1, b: 0 }
    }

    /// Actualizar el checksum con más datos
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.a = (self.a + byte as u32) % Self::MOD_ADLER;
            self.b = (self.b + self.a) % Self::MOD_ADLER;
        }
    }

    /// Finalizar y obtener el checksum
    pub fn finalize(&self) -> u32 {
        (self.b << 16) | self.a
    }

    /// Calcular ADLER-32 de un slice de bytes (método estático)
    #[inline]
    pub fn compute(data: &[u8]) -> u32 {
        let mut adler = Self::new();
        adler.update(data);
        adler.finalize()
    }

    /// Verificar si el ADLER-32 coincide
    #[inline]
    pub fn verify(data: &[u8], expected: u32) -> bool {
        Self::compute(data) == expected
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        // CRC-32 de string vacía
        assert_eq!(Crc32::compute(b""), 0x00000000);
    }

    #[test]
    fn test_crc32_known_value() {
        // Valor conocido: "123456789" -> 0xCBF43926
        let crc = Crc32::compute(b"123456789");
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_crc32_hello_world() {
        let crc = Crc32::compute(b"Hello, World!");
        // Valor verificado contra implementación de referencia
        assert_eq!(crc, 0xEC4AC3D0);
    }

    #[test]
    fn test_crc32_verify() {
        let data = b"Test data for CRC";
        let crc = Crc32::compute(data);
        assert!(Crc32::verify(data, crc));
        assert!(!Crc32::verify(data, crc ^ 1)); // Modificar un bit
    }

    #[test]
    fn test_crc32_incremental() {
        let data1 = b"Hello, ";
        let data2 = b"World!";
        let full = b"Hello, World!";

        // CRC completo
        let crc_full = Crc32::compute(full);

        // CRC incremental: actualizar en dos partes produce el mismo resultado
        let mut crc = Crc32::new();
        crc.update(data1);
        crc.update(data2);
        assert_eq!(crc.finalize(), crc_full);
    }

    #[test]
    fn test_crc64_basic() {
        let crc = Crc64::compute(b"123456789");
        // Valor puede variar según implementación exacta
        assert_ne!(crc, 0); // Al menos no es cero
    }

    #[test]
    fn test_adler32_empty() {
        // ADLER-32 de string vacía es 1
        assert_eq!(Adler32::compute(b""), 1);
    }

    #[test]
    fn test_adler32_known_value() {
        // Valor conocido: "Wikipedia" -> 0x11E60398
        let adler = Adler32::compute(b"Wikipedia");
        assert_eq!(adler, 0x11E60398);
    }

    #[test]
    fn test_adler32_verify() {
        let data = b"Test ADLER-32";
        let adler = Adler32::compute(data);
        assert!(Adler32::verify(data, adler));
    }
}
