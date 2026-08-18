use std::path::Path;

pub(crate) fn convert(path: &Path) -> Result<String, String> {
    anydoc::to_markdown(path).map_err(|error| error.to_string())
}
