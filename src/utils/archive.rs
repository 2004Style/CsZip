//! Soporte para compresión y descompresión de formatos ZIP y RAR
use crate::error::{Error, ErrorKind};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Comprime un archivo o directorio a formato .zip
pub fn compress_zip(input: &Path, output: &Path) -> Result<(), Error> {
    let zip_file = File::create(output).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("No se pudo crear el archivo zip de salida: {}", e),
        )
    })?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    if input.is_dir() {
        let mut buffer = Vec::new();
        compress_dir_recursive(input, input, &mut zip, options, &mut buffer)?;
    } else {
        let filename = input
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Nombre de archivo no válido"))?;

        zip.start_file(filename, options).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("No se pudo escribir en el zip: {}", e),
            )
        })?;

        let mut f = File::open(input).map_err(|e| {
            Error::new(
                ErrorKind::FileNotFound,
                format!("No se pudo abrir el archivo de entrada: {}", e),
            )
        })?;
        io::copy(&mut f, &mut zip).map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error al copiar datos al zip: {}", e),
            )
        })?;
    }

    zip.finish().map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("No se pudo finalizar el archivo zip: {}", e),
        )
    })?;

    Ok(())
}

/// Helper recursivo para comprimir directorios en zip
fn compress_dir_recursive<W: Write + std::io::Seek>(
    base_dir: &Path,
    current_dir: &Path,
    zip: &mut zip::ZipWriter<W>,
    options: zip::write::FileOptions,
    buffer: &mut Vec<u8>,
) -> Result<(), Error> {
    for entry in std::fs::read_dir(current_dir).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("Error leyendo directorio: {}", e),
        )
    })? {
        let entry = entry.map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Error en entrada de directorio: {}", e),
            )
        })?;
        let path = entry.path();

        let name = path.strip_prefix(base_dir).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Error al procesar ruta: {}", e),
            )
        })?;
        let name_str = name
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Ruta no UTF-8"))?;

        if path.is_dir() {
            zip.add_directory(name_str, options).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("Error añadiendo directorio al zip: {}", e),
                )
            })?;
            compress_dir_recursive(base_dir, &path, zip, options, buffer)?;
        } else {
            zip.start_file(name_str, options).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("Error empezando archivo en el zip: {}", e),
                )
            })?;
            let mut f = File::open(&path).map_err(|e| {
                Error::new(
                    ErrorKind::FileNotFound,
                    format!("Error abriendo archivo: {}", e),
                )
            })?;
            f.read_to_end(buffer).map_err(|e| {
                Error::new(ErrorKind::IoError, format!("Error leyendo archivo: {}", e))
            })?;
            zip.write_all(buffer).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("Error escribiendo archivo al zip: {}", e),
                )
            })?;
            buffer.clear();
        }
    }
    Ok(())
}

/// Descomprime un archivo .zip a un directorio de destino
pub fn decompress_zip(input: &Path, output_dir: &Path) -> Result<(), Error> {
    let file = File::open(input).map_err(|e| {
        Error::new(
            ErrorKind::FileNotFound,
            format!("No se pudo abrir el archivo zip: {}", e),
        )
    })?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("El archivo no parece ser un zip válido: {}", e),
        )
    })?;

    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("No se pudo crear el directorio de salida: {}", e),
        )
    })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Error leyendo archivo en zip: {}", e),
            )
        })?;

        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("No se pudo crear directorio: {}", e),
                )
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| {
                        Error::new(
                            ErrorKind::IoError,
                            format!("No se pudo crear directorio: {}", e),
                        )
                    })?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("No se pudo crear archivo descomprimido: {}", e),
                )
            })?;
            io::copy(&mut file, &mut outfile).map_err(|e| {
                Error::new(
                    ErrorKind::IoError,
                    format!("Error escribiendo archivo descomprimido: {}", e),
                )
            })?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(())
}

/// Descomprime un archivo .rar usando el comando de sistema 'unrar'
pub fn decompress_rar(input: &Path, output_dir: &Path) -> Result<(), Error> {
    let output = std::process::Command::new("unrar")
        .arg("--version")
        .output();

    if output.is_err() {
        return Err(Error::new(
            ErrorKind::UnsupportedAlgorithm,
            "La extracción de archivos .rar requiere que la utilidad 'unrar' esté instalada en el sistema. Por favor, instálala (ej: sudo apt install unrar) para continuar."
        ));
    }

    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::new(
            ErrorKind::IoError,
            format!("No se pudo crear el directorio de salida: {}", e),
        )
    })?;

    let status = std::process::Command::new("unrar")
        .arg("x")
        .arg("-y")
        .arg(input)
        .arg(output_dir)
        .status()
        .map_err(|e| {
            Error::new(
                ErrorKind::IoError,
                format!("Fallo al iniciar el comando unrar: {}", e),
            )
        })?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("El comando 'unrar' falló con código de salida: {}", status),
        ));
    }

    Ok(())
}
