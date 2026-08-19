use std::fs;
use std::path::Path;

use crate::converter;

/// Converts a supported document to Markdown and writes the unpacked file
/// expected by WinMerge.
pub fn unpack_file(src: &Path, dst: &Path) -> Result<(), String> {
    let markdown = converter::convert(src)?;

    fs::write(dst, markdown).map_err(|e| e.to_string())
}
