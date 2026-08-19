use std::path::Path;

/// Delegates conversion to anydoc and normalizes its errors for the CLI and COM layers.
pub(crate) fn convert(path: &Path) -> Result<String, String> {
    anydoc::to_markdown(path).map_err(|error| error.to_string())
}
