use std::path::Path;

pub fn safe_filename(file_path: &Path) -> String {
    file_path.file_name().unwrap_or_default().to_string_lossy().replace('"', "")
}
