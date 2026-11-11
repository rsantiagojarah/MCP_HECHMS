use std::path::Path;

use crate::error::HmsError;

/// Lee un archivo de proyecto. Acepta UTF-8 y, si no es válido, Windows-1252.
pub fn read_text(path: &Path) -> Result<String, HmsError> {
    let bytes = std::fs::read(path)?;
    decode(&bytes)
}

pub fn write_text(path: &Path, text: &str) -> Result<(), HmsError> {
    std::fs::write(path, text)?;
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<String, HmsError> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok(text.to_string());
    }
    let (text, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    Ok(text.into_owned())
}
