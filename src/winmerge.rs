use std::fs;
use std::path::Path;

use crate::converter;

pub fn unpack_file(src: &Path, dst: &Path) -> Result<(), String> {
    let markdown = converter::convert(src)?;

    fs::write(dst, markdown).map_err(|e| e.to_string())
}
