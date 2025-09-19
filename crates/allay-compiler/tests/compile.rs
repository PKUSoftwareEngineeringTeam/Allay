use allay_base::file;
use allay_compiler::compile;
use std::{path::PathBuf, sync::OnceLock};
use tempfile::{TempDir, tempdir};

fn test_temp_dir() -> &'static TempDir {
    static TEMP_DIR: OnceLock<TempDir> = OnceLock::new();
    TEMP_DIR.get_or_init(|| tempdir().unwrap())
}

fn create_include_dir() -> PathBuf {
    let include_dir = test_temp_dir().path().join("includes");
    file::create_dir_if_not_exists(&include_dir).unwrap();
    include_dir
}

fn create_shortcode_dir() -> PathBuf {
    let shortcode_dir = test_temp_dir().path().join("shortcodes");
    file::create_dir_if_not_exists(&shortcode_dir).unwrap();
    shortcode_dir
}

fn create_test_file(filename: &str, content: &str) -> PathBuf {
    let dir = test_temp_dir();
    let file_path = dir.path().join(filename);
    file::write_file(&file_path, content).unwrap();
    file_path
}

fn get_compile_res(content: &str) -> String {
    let include_dir = create_include_dir();
    let shortcode_dir = create_shortcode_dir();
    let source = create_test_file("test.md", content);
    compile(source, include_dir, shortcode_dir).unwrap()
}

fn get_compile_res_tokens(content: &str) -> Vec<String> {
    let res = get_compile_res(content);
    res.trim().split_whitespace().map(|s| s.to_string()).collect()
}

#[test]
fn test_simple() {
    assert_eq!(get_compile_res("Hello, World!"), "<p>Hello, World!</p>\n");
    let content = "{- set $var = 10 -} {: $var :}";
    assert_eq!(get_compile_res_tokens(content), vec!["<p>", "10", "</p>"]);
}

#[test]
fn test_algorithm() {
    let content = "{- set $sum = 5+--(-6)*10 -} {: $sum :}";
    assert_eq!(get_compile_res_tokens(content), vec!["<p>", "-55", "</p>"]);
    // let content = "{- set $a = 10 -} {- set $b = 20 -} {: if $a < $b :}Less{: else :}Greater{: /if :}";
    let content = r#"{- set $a = 10 -}
{- set $b = 20 -}
{: ($a + $b) % 7 :}
{- if $a == $b -}Equal{- else -}NotEq{- end -}"#; // FIXME: "<" will be transformed to "&lt;"
    assert_eq!(
        get_compile_res_tokens(content),
        vec!["<p>", "2", "NotEq", "</p>"]
    );
}
