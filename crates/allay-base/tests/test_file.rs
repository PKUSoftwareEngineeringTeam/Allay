use allay_base::file::FileUtils;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn create_test_files() -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    let sub_dir = dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let files = vec![
        (dir.path().join("test1.txt"), "Hello World\nThis is a test"),
        (
            dir.path().join("test2.rs"),
            "fn main() {\n    println!(\"Hello\");\n}",
        ),
        (sub_dir.join("test3.md"), "# Markdown\nSome content"),
    ];

    for (path, content) in files {
        let mut file = File::create(path).unwrap();
        writeln!(file, "{}", content).unwrap();
    }

    dir
}

#[test]
fn test_walk_dir() {
    let test_dir = create_test_files();
    let files = FileUtils::walk_dir(test_dir.path()).unwrap();

    assert_eq!(files.len(), 3);
    assert!(
        files
            .iter()
            .any(|f| f.relative_path.to_str().unwrap() == "subdir/test3.md")
    );
    assert!(files.iter().any(|f| f.extension == Some("rs".to_string())));
}

#[test]
fn test_read_file() {
    let test_dir = create_test_files();
    let test_file = test_dir.path().join("test1.txt");

    let content = FileUtils::read_file(&test_file).unwrap();
    assert!(content.content.contains("Hello World"));
    assert_eq!(content.line_count, 2);
}
