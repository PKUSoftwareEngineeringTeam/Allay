use allay_parser::AllayMdParser;

#[test]
fn test_basic() {
    let markdown = "# Hello, world!\nThis is a new **test**.";
    assert_eq!(
        AllayMdParser::default().parse(markdown).trim(),
        r#"<h1>Hello, world!</h1>
<p>This is a new <strong>test</strong>.</p>"#
    );
}

// #[test]
// fn test_shortcode() {
//     let shortcode = Shortcode::new("note".into(), "".into());
//     let markdown = "{{ note }} This is a note. {{ /note }}";
// }
