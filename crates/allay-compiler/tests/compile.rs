use allay_base::file;
use allay_compiler::Compiler;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn create_include_dir<P: AsRef<Path>>(temp_dir: P) -> PathBuf {
    let include_dir = temp_dir.as_ref().join("includes");
    file::create_dir_recursively(&include_dir).unwrap();
    include_dir
}

fn create_shortcode_dir<P: AsRef<Path>>(temp_dir: P) -> PathBuf {
    let shortcode_dir = temp_dir.as_ref().join("shortcodes");
    file::create_dir_recursively(&shortcode_dir).unwrap();
    shortcode_dir
}

fn create_test_file<P: AsRef<Path>>(temp_dir: P, filename: &str, content: &str) -> PathBuf {
    let file_path = temp_dir.as_ref().join(filename);
    file::write_file(&file_path, content).unwrap();
    file_path
}

fn get_compile_res(content: &str) -> String {
    let temp_dir = tempdir().unwrap();
    let temp_dir = temp_dir.path();
    let include_dir = create_include_dir(temp_dir);
    let shortcode_dir = create_shortcode_dir(temp_dir);
    let source = create_test_file(temp_dir, "test.md", content);
    Compiler::raw(source, include_dir, shortcode_dir).unwrap()
}

fn to_tokens(s: String) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_string()).collect()
}

#[test]
fn test_simple() {
    assert_eq!(get_compile_res("Hello, World!"), "<p>Hello, World!</p>\n");
    let content = "{- set $var = 10 -} {: $var :}";
    assert_eq!(to_tokens(get_compile_res(content)), vec!["<p>10</p>"]);
}

#[test]
fn test_algorithm() {
    let content = "{- set $sum = 5+--(-6)*10 -} {: $sum :}";
    assert_eq!(to_tokens(get_compile_res(content)), vec!["<p>-55</p>"]);
    // let content = "{- set $a = 10 -} {- set $b = 20 -} {: if $a < $b :}Less{: else :}Greater{: /if :}";
    let content = r#"{- set $a = 10 -}
{- set $b = 20 -}
{: ($a + $b) % 7 :}
{- if $a == $b -}Equal{- else -}NotEq{- end -}"#;
    assert_eq!(
        to_tokens(get_compile_res(content)),
        vec!["<p>2", "NotEq</p>"]
    );
}

#[test]
fn test_shortcode() {
    let temp_dir = tempdir().unwrap();
    let include_dir = create_include_dir(&temp_dir);
    let shortcode_dir = create_shortcode_dir(&temp_dir);

    create_test_file(&shortcode_dir, "test.md", "Shortcode");
    let source_file = create_test_file(&temp_dir, "source.md", "{< test />}");

    let res = Compiler::raw(source_file, include_dir, shortcode_dir).unwrap();
    assert_eq!(to_tokens(res), vec!["<p>Shortcode</p>"]);
}

#[test]
fn test_shortcode_with_params_and_inner() {
    let temp_dir = tempdir().unwrap();
    let include_dir = create_include_dir(&temp_dir);
    let shortcode_dir = create_shortcode_dir(&temp_dir);

    create_test_file(
        &shortcode_dir,
        "test.md",
        "{: param.0 :} {: .inner :} {: param.1 :}",
    );
    let source_file = create_test_file(
        &temp_dir,
        "source.md",
        "{< test \"hello\" 114*1000+514 >} something {</ test >}",
    );

    let res = Compiler::raw(source_file, include_dir, shortcode_dir).unwrap();
    assert_eq!(
        to_tokens(res),
        vec!["<p>hello", "<p>something</p>", "114514</p>"]
    );
}

#[test]
fn test_include() {
    let temp_dir = tempdir().unwrap();
    let include_dir = create_include_dir(&temp_dir);
    let shortcode_dir = create_shortcode_dir(&temp_dir);

    create_test_file(&include_dir, "header.md", "<header>Header</header>");
    let source_file = create_test_file(&temp_dir, "source.md", "{- include \"header\" -} Body");

    let res = Compiler::raw(source_file, include_dir, shortcode_dir).unwrap();
    assert_eq!(to_tokens(res), vec!["<header>Header</header>", "Body"]);
}

#[test]
fn test_recursive_include() {
    let temp_dir = tempdir().unwrap();
    let include_dir = create_include_dir(&temp_dir);
    let shortcode_dir = create_shortcode_dir(&temp_dir);

    create_test_file(&include_dir, "part1.md", "Part1. {- include \"part2\" -}");
    create_test_file(&include_dir, "part2.md", "Part2. {- include \"part3\" -}");
    create_test_file(&include_dir, "part3.md", "Part3.");
    let source_file = create_test_file(&temp_dir, "source.md", "{- include \"part1\" -}");

    let res = Compiler::raw(source_file, include_dir, shortcode_dir).unwrap();
    assert_eq!(
        to_tokens(res),
        vec!["<p>Part1.", "<p>Part2.", "<p>Part3.</p></p></p>",]
    )
}

#[test]
fn test_markdown_meta() {
    let temp_dir = tempdir().unwrap();
    let include_dir = create_include_dir(&temp_dir);
    let shortcode_dir = create_shortcode_dir(&temp_dir);

    let source_file = create_test_file(
        &temp_dir,
        "source.md",
        r#"---
name: "Test Page"
---
{: .name :}
    "#,
    );
    let res = Compiler::raw(source_file, include_dir, shortcode_dir).unwrap();
    assert_eq!(to_tokens(res), vec!["<p>Test", "Page</p>"])
}
