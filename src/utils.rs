//! Funciones de utilidad para CsZip
//!
//! Proporciona helpers comunes utilizados en todo el proyecto.

use std::path::Path;

pub mod archive;

/// Formatea un tamaño en bytes de forma legible
///
/// # Ejemplos
///
/// ```
/// use cszip::utils::format_size;
///
/// assert_eq!(format_size(0), "0 B");
/// assert_eq!(format_size(1024), "1.00 KiB");
/// assert_eq!(format_size(1048576), "1.00 MiB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Calcula el ratio de compresión como porcentaje
///
/// Retorna el porcentaje del tamaño comprimido respecto al original.
/// Un valor < 100 indica compresión efectiva.
///
/// # Ejemplos
///
/// ```
/// use cszip::utils::compression_ratio;
///
/// assert_eq!(compression_ratio(100, 50), 50.0);  // 50% del original
/// assert_eq!(compression_ratio(100, 100), 100.0); // Sin compresión
/// ```
pub fn compression_ratio(original: u64, compressed: u64) -> f64 {
    if original == 0 {
        return 100.0;
    }
    (compressed as f64 / original as f64) * 100.0
}

/// Calcula el ahorro de espacio como porcentaje
///
/// # Ejemplos
///
/// ```
/// use cszip::utils::space_savings;
///
/// assert_eq!(space_savings(100, 50), 50.0);  // 50% ahorrado
/// assert_eq!(space_savings(100, 100), 0.0);  // Sin ahorro
/// ```
pub fn space_savings(original: u64, compressed: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    ((original - compressed.min(original)) as f64 / original as f64) * 100.0
}

/// Formatea duración en formato legible
pub fn format_duration(millis: u64) -> String {
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60_000 {
        format!("{:.2}s", millis as f64 / 1000.0)
    } else {
        let mins = millis / 60_000;
        let secs = (millis % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Calcula throughput en bytes por segundo
pub fn throughput(bytes: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 0.0;
    }
    (bytes as f64 / duration_ms as f64) * 1000.0
}

/// Formatea throughput de forma legible
pub fn format_throughput(bytes_per_sec: f64) -> String {
    format!("{}/s", format_size(bytes_per_sec as u64))
}

/// Obtiene la extensión de un archivo
pub fn get_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|e| e.to_str())
}

/// Verifica si un path tiene extensión .cz
pub fn is_cz_file(path: &Path) -> bool {
    get_extension(path)
        .map(|e| e.eq_ignore_ascii_case("cz"))
        .unwrap_or(false)
}

/// Genera el nombre del archivo de salida para compresión
pub fn output_path_for_compress(input: &Path) -> std::path::PathBuf {
    let mut output = input.as_os_str().to_os_string();
    output.push(".cz");
    std::path::PathBuf::from(output)
}

/// Genera el nombre del archivo de salida para descompresión
pub fn output_path_for_decompress(input: &Path) -> Option<std::path::PathBuf> {
    let input_str = input.to_string_lossy();
    input_str.strip_suffix(".cz").map(std::path::PathBuf::from)
}

/// Verifica si un número es potencia de 2
pub fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Calcula el log2 de un número (redondeado hacia abajo)
pub fn log2(n: usize) -> u32 {
    if n == 0 {
        return 0;
    }
    (std::mem::size_of::<usize>() * 8) as u32 - n.leading_zeros() - 1
}

/// Alinea un valor hacia arriba al múltiplo más cercano
pub fn align_up(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) / alignment * alignment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KiB");
        assert_eq!(format_size(1536), "1.50 KiB");
        assert_eq!(format_size(1048576), "1.00 MiB");
        assert_eq!(format_size(1073741824), "1.00 GiB");
    }

    #[test]
    fn test_compression_ratio() {
        assert_eq!(compression_ratio(100, 50), 50.0);
        assert_eq!(compression_ratio(100, 100), 100.0);
        assert_eq!(compression_ratio(100, 200), 200.0);
        assert_eq!(compression_ratio(0, 50), 100.0);
    }

    #[test]
    fn test_space_savings() {
        assert_eq!(space_savings(100, 50), 50.0);
        assert_eq!(space_savings(100, 100), 0.0);
        assert_eq!(space_savings(100, 0), 100.0);
        assert_eq!(space_savings(0, 50), 0.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_is_cz_file() {
        assert!(is_cz_file(Path::new("file.cz")));
        assert!(is_cz_file(Path::new("path/to/file.CZ")));
        assert!(!is_cz_file(Path::new("file.txt")));
        assert!(!is_cz_file(Path::new("file")));
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(1024));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(5));
    }

    #[test]
    fn test_log2() {
        assert_eq!(log2(1), 0);
        assert_eq!(log2(2), 1);
        assert_eq!(log2(4), 2);
        assert_eq!(log2(8), 3);
        assert_eq!(log2(1024), 10);
        assert_eq!(log2(0), 0);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }

    #[test]
    fn test_output_paths() {
        let input = Path::new("file.txt");
        let output = output_path_for_compress(input);
        assert_eq!(output.to_string_lossy(), "file.txt.cz");

        let cz_file = Path::new("file.txt.cz");
        let original = output_path_for_decompress(cz_file);
        assert_eq!(original.unwrap().to_string_lossy(), "file.txt");
    }
}
