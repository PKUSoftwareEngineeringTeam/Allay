use std::path::Path;

pub fn safe_filename(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace('"', "")
}
