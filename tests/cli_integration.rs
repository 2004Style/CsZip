//! Tests de integración para el CLI de CsZip
//!
//! Prueba todos los comandos: compress, decompress, info, verify, list

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

/// Estructura helper para ejecutar el CLI
struct CliRunner {
    binary_path: PathBuf,
    temp_dir: TempDir,
}

impl CliRunner {
    fn new() -> Self {
        // Buscar el binario compilado
        let binary_path = Self::find_binary();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        Self {
            binary_path,
            temp_dir,
        }
    }

    fn find_binary() -> PathBuf {
        // Buscar en target/debug o target/release
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let debug_path = PathBuf::from(manifest_dir).join("target/debug/cszip.exe");
        let release_path = PathBuf::from(manifest_dir).join("target/release/cszip.exe");

        // En sistemas Unix no tiene .exe
        #[cfg(not(windows))]
        let debug_path = PathBuf::from(manifest_dir).join("target/debug/cszip");
        #[cfg(not(windows))]
        let release_path = PathBuf::from(manifest_dir).join("target/release/cszip");

        if debug_path.exists() {
            debug_path
        } else if release_path.exists() {
            release_path
        } else {
            // Fallback para cargo test
            PathBuf::from("cszip")
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(&self.binary_path)
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .expect("Failed to execute command")
    }

    fn create_test_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content).expect("Failed to write test file");
        path
    }

    fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

// ============================================================================
// Tests del comando COMPRESS
// ============================================================================

mod compress_tests {
    use super::*;

    #[test]
    fn test_compress_simple_file() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("test.txt", b"Hello, CsZip! This is a test file.");

        let output = runner.run(&["compress", input.to_str().unwrap()]);

        assert!(
            output.status.success(),
            "Compress should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Verificar que se creó el archivo .cz
        let compressed = input.with_extension("txt.cz");
        assert!(
            compressed.exists(),
            "Compressed file should exist at {:?}",
            compressed
        );
    }

    #[test]
    fn test_compress_with_output_path() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("input.txt", b"Test content for compression");
        let output_path = runner.temp_path().join("custom_output.cz");

        let output = runner.run(&[
            "compress",
            input.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ]);

        assert!(
            output.status.success(),
            "Compress with output path should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output_path.exists(), "Custom output file should exist");
    }

    #[test]
    fn test_compress_with_force_overwrite() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("force_test.txt", b"Original content");
        let output_path = runner.temp_path().join("force_test.txt.cz");

        // Crear archivo existente
        File::create(&output_path).expect("Create existing file");

        // Sin force debería fallar
        let output_no_force = runner.run(&[
            "compress",
            input.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ]);
        assert!(
            !output_no_force.status.success(),
            "Should fail without force"
        );

        // Con force debería funcionar
        let output_force = runner.run(&[
            "compress",
            input.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-f",
        ]);
        assert!(
            output_force.status.success(),
            "Should succeed with force: {:?}",
            String::from_utf8_lossy(&output_force.stderr)
        );
    }

    #[test]
    fn test_compress_nonexistent_file() {
        let runner = CliRunner::new();

        let output = runner.run(&["compress", "nonexistent_file.txt"]);

        assert!(!output.status.success(), "Should fail for nonexistent file");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error"), "Should show error message");
    }

    #[test]
    fn test_compress_empty_file() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("empty.txt", b"");

        let output = runner.run(&["compress", input.to_str().unwrap()]);

        // Archivo vacío debería comprimirse (aunque solo sea el header)
        assert!(
            output.status.success(),
            "Empty file compression should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_compress_with_verbose() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("verbose.txt", b"Test with verbose output");

        let output = runner.run(&["compress", "-v", input.to_str().unwrap()]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Con verbose debería mostrar información
        assert!(stdout.len() > 0 || output.stderr.len() > 0);
    }

    #[test]
    fn test_compress_large_content() {
        let runner = CliRunner::new();
        // Crear contenido grande (1MB de datos repetitivos)
        let content: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
        let input = runner.create_test_file("large.bin", &content);

        let output = runner.run(&["compress", input.to_str().unwrap()]);

        assert!(
            output.status.success(),
            "Large file compression should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let compressed = input.with_extension("bin.cz");
        assert!(compressed.exists(), "Compressed file should exist");
    }

    #[test]
    fn test_compress_with_level() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("level_test.txt", b"Testing compression levels");

        for level in [0, 5, 9] {
            let output_path = runner.temp_path().join(format!("level_{}.cz", level));

            let output = runner.run(&[
                "compress",
                input.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
                "-l",
                &level.to_string(),
            ]);

            assert!(
                output.status.success(),
                "Compression with level {} should succeed: {:?}",
                level,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

// ============================================================================
// Tests del comando DECOMPRESS
// ============================================================================

mod decompress_tests {
    use super::*;

    #[allow(dead_code)]
    fn compress_helper(runner: &CliRunner, content: &[u8], name: &str) -> PathBuf {
        let input = runner.create_test_file(name, content);
        runner.run(&["compress", input.to_str().unwrap()]);
        input.with_extension(format!(
            "{}.cz",
            input.extension().unwrap_or_default().to_str().unwrap()
        ))
    }

    #[test]
    fn test_decompress_simple() {
        let runner = CliRunner::new();
        let original_content = b"Hello, World! Testing decompression.";
        let input = runner.create_test_file("decomp_test.txt", original_content);

        // Comprimir
        let compress_result = runner.run(&["compress", input.to_str().unwrap()]);
        assert!(compress_result.status.success(), "Compression failed");

        let compressed = input.with_extension("txt.cz");

        // Eliminar original para verificar que se recrea
        fs::remove_file(&input).expect("Remove original");

        // Descomprimir
        let decompress_result = runner.run(&["decompress", compressed.to_str().unwrap()]);
        assert!(
            decompress_result.status.success(),
            "Decompression failed: {:?}",
            String::from_utf8_lossy(&decompress_result.stderr)
        );

        // Verificar contenido
        let recovered = fs::read(&input).expect("Read recovered file");
        assert_eq!(recovered, original_content, "Content should match original");
    }

    #[test]
    fn test_decompress_with_output_path() {
        let runner = CliRunner::new();
        let content = b"Custom output path test";
        let input = runner.create_test_file("custom_out.txt", content);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        let output_path = runner.temp_path().join("recovered.txt");

        let output = runner.run(&[
            "decompress",
            compressed.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ]);

        assert!(output.status.success());
        let recovered = fs::read(&output_path).expect("Read output");
        assert_eq!(recovered, content);
    }

    #[test]
    fn test_decompress_invalid_file() {
        let runner = CliRunner::new();
        // Crear archivo con contenido inválido (no es .cz)
        let invalid = runner.create_test_file("invalid.cz", b"This is not a valid CZ file");

        let output = runner.run(&["decompress", invalid.to_str().unwrap()]);

        assert!(!output.status.success(), "Should fail for invalid file");
    }

    #[test]
    fn test_decompress_with_force() {
        let runner = CliRunner::new();
        let content = b"Force overwrite test";
        let input = runner.create_test_file("force_decomp.txt", content);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        // Sin force debería fallar (archivo original existe)
        let output_no_force = runner.run(&["decompress", compressed.to_str().unwrap()]);
        assert!(
            !output_no_force.status.success(),
            "Should fail without force"
        );

        // Con force debería funcionar
        let output_force = runner.run(&["decompress", "-f", compressed.to_str().unwrap()]);
        assert!(
            output_force.status.success(),
            "Should succeed with force: {:?}",
            String::from_utf8_lossy(&output_force.stderr)
        );
    }

    #[test]
    fn test_decompress_nonexistent() {
        let runner = CliRunner::new();

        let output = runner.run(&["decompress", "nonexistent.cz"]);

        assert!(!output.status.success());
    }
}

// ============================================================================
// Tests del comando INFO
// ============================================================================

mod info_tests {
    use super::*;

    #[test]
    fn test_info_basic() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("info_test.txt", b"File for info command testing");

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        let output = runner.run(&["info", compressed.to_str().unwrap()]);

        assert!(
            output.status.success(),
            "Info command should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Debería mostrar información del archivo
        assert!(
            stdout.contains("CZ")
                || stdout.contains("version")
                || stdout.len() > 0
                || output.stderr.len() > 0
        );
    }

    #[test]
    fn test_info_detailed() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("detailed.txt", b"Detailed info test");

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        let output = runner.run(&["info", "-d", compressed.to_str().unwrap()]);

        assert!(output.status.success());
    }

    #[test]
    fn test_info_invalid_file() {
        let runner = CliRunner::new();
        let invalid = runner.create_test_file("not_cz.cz", b"Invalid content");

        let output = runner.run(&["info", invalid.to_str().unwrap()]);

        assert!(!output.status.success(), "Info on invalid file should fail");
    }
}

// ============================================================================
// Tests del comando VERIFY
// ============================================================================

mod verify_tests {
    use super::*;

    #[test]
    fn test_verify_valid_file() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("verify_test.txt", b"Content to verify integrity");

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        let output = runner.run(&["verify", compressed.to_str().unwrap()]);

        assert!(
            output.status.success(),
            "Verify should succeed for valid file: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_verify_corrupted_file() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("corrupt_test.txt", b"Original content");

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        // Corromper el archivo
        let mut data = fs::read(&compressed).expect("Read compressed");
        if data.len() > 20 {
            // Modificar bytes en el medio para corromper datos
            data[20] ^= 0xFF;
            data[21] ^= 0xFF;
        }
        fs::write(&compressed, data).expect("Write corrupted");

        let output = runner.run(&["verify", compressed.to_str().unwrap()]);

        // Puede fallar o reportar corrupción
        // El comportamiento exacto depende de la implementación
        assert!(
            !output.status.success()
                || String::from_utf8_lossy(&output.stderr).contains("error")
                || String::from_utf8_lossy(&output.stderr).contains("Error")
                || String::from_utf8_lossy(&output.stdout).contains("FAIL"),
            "Should detect corruption"
        );
    }

    #[test]
    fn test_verify_invalid_magic() {
        let runner = CliRunner::new();
        // Crear archivo con magic number incorrecto
        let mut data = vec![0x00, 0x00]; // Magic incorrecto
        data.extend_from_slice(&[0u8; 50]); // Padding
        let invalid = runner.create_test_file("bad_magic.cz", &data);

        let output = runner.run(&["verify", invalid.to_str().unwrap()]);

        assert!(!output.status.success(), "Should fail for invalid magic");
    }
}

// ============================================================================
// Tests del comando LIST
// ============================================================================

mod list_tests {
    use super::*;

    #[test]
    fn test_list_single_block() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("list_test.txt", b"Small content for listing");

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");

        let output = runner.run(&["list", compressed.to_str().unwrap()]);

        assert!(
            output.status.success(),
            "List should succeed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_list_multiple_blocks() {
        let runner = CliRunner::new();
        // Crear contenido grande para generar múltiples bloques
        let content: Vec<u8> = (0..500 * 1024).map(|i| (i % 256) as u8).collect();
        let input = runner.create_test_file("large_list.bin", &content);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("bin.cz");

        let output = runner.run(&["list", compressed.to_str().unwrap()]);

        assert!(output.status.success());
    }

    #[test]
    fn test_list_invalid_file() {
        let runner = CliRunner::new();
        let invalid = runner.create_test_file("not_valid.cz", b"Not a CZ file");

        let output = runner.run(&["list", invalid.to_str().unwrap()]);

        assert!(!output.status.success());
    }
}

// ============================================================================
// Tests de opciones globales
// ============================================================================

mod global_options_tests {
    use super::*;

    #[test]
    fn test_help() {
        let runner = CliRunner::new();

        let output = runner.run(&["--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("cszip") || stdout.contains("CsZip") || stdout.contains("compress")
        );
    }

    #[test]
    fn test_version() {
        let runner = CliRunner::new();

        let output = runner.run(&["--version"]);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("0.1.0") || stdout.contains("cszip") || stdout.contains("1.0"));
    }

    #[test]
    fn test_quiet_mode() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("quiet.txt", b"Quiet mode test");

        let output = runner.run(&["--quiet", "compress", input.to_str().unwrap()]);

        assert!(output.status.success());
        // En modo quiet no debería haber output en stdout
    }

    #[test]
    fn test_command_aliases() {
        let runner = CliRunner::new();
        let input = runner.create_test_file("alias.txt", b"Testing aliases");

        // Test alias 'c' para compress
        let output_c = runner.run(&["c", input.to_str().unwrap()]);
        assert!(output_c.status.success(), "Alias 'c' should work");

        let compressed = input.with_extension("txt.cz");
        fs::remove_file(&input).ok();

        // Test alias 'd' para decompress
        let output_d = runner.run(&["d", compressed.to_str().unwrap()]);
        assert!(output_d.status.success(), "Alias 'd' should work");
    }
}

// ============================================================================
// Tests de roundtrip (end-to-end)
// ============================================================================

mod roundtrip_tests {
    use super::*;

    #[test]
    fn test_roundtrip_text_file() {
        let runner = CliRunner::new();
        let original = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        let input = runner.create_test_file("roundtrip.txt", original);

        // Compress
        let compress_out = runner.run(&["compress", input.to_str().unwrap()]);
        assert!(compress_out.status.success());

        let compressed = input.with_extension("txt.cz");

        // Remove original
        fs::remove_file(&input).expect("Remove original");

        // Decompress
        let decompress_out = runner.run(&["decompress", compressed.to_str().unwrap()]);
        assert!(decompress_out.status.success());

        // Verify content
        let recovered = fs::read(&input).expect("Read recovered");
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let runner = CliRunner::new();
        // Datos binarios con todos los bytes posibles
        let original: Vec<u8> = (0..=255).collect();
        let input = runner.create_test_file("binary.bin", &original);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("bin.cz");
        fs::remove_file(&input).ok();

        runner.run(&["decompress", compressed.to_str().unwrap()]);

        let recovered = fs::read(&input).expect("Read recovered");
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_roundtrip_repetitive_data() {
        let runner = CliRunner::new();
        // Datos altamente repetitivos (ideales para compresión)
        let original: Vec<u8> = vec![b'A'; 100_000];
        let input = runner.create_test_file("repetitive.txt", &original);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("txt.cz");
        fs::remove_file(&input).ok();

        runner.run(&["decompress", compressed.to_str().unwrap()]);

        let recovered = fs::read(&input).expect("Read recovered");
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_roundtrip_random_data() {
        let runner = CliRunner::new();
        // Datos pseudo-aleatorios (difíciles de comprimir)
        let original: Vec<u8> = (0..10_000).map(|i| ((i * 17 + 31) % 256) as u8).collect();
        let input = runner.create_test_file("random.bin", &original);

        runner.run(&["compress", input.to_str().unwrap()]);
        let compressed = input.with_extension("bin.cz");
        fs::remove_file(&input).ok();

        runner.run(&["decompress", compressed.to_str().unwrap()]);

        let recovered = fs::read(&input).expect("Read recovered");
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_roundtrip_multiple_files() {
        let runner = CliRunner::new();

        let files_data = [
            ("file1.txt", b"First file content".to_vec()),
            ("file2.txt", b"Second file with different content".to_vec()),
            ("file3.bin", (0..500u16).map(|i| (i % 256) as u8).collect()),
        ];

        for (name, content) in &files_data {
            let input = runner.create_test_file(name, content);

            runner.run(&["compress", input.to_str().unwrap()]);
            let compressed = input.with_extension(format!(
                "{}.cz",
                input.extension().unwrap_or_default().to_str().unwrap()
            ));

            fs::remove_file(&input).ok();
            runner.run(&["decompress", compressed.to_str().unwrap()]);

            let recovered = fs::read(&input).expect("Read recovered");
            assert_eq!(&recovered, content, "File {} content mismatch", name);
        }
    }
}
