mod parser;

use crate::ParseResult;
use crate::ast::File;
use parser::ASTBuilder;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use std::borrow::Cow;

/// The template parser using Pest
#[derive(Parser)]
#[grammar = "parse/allay.pest"]
struct TemplateParser;

fn remove_html_comments(html: &'_ str) -> Cow<'_, str> {
    let re = Regex::new(r"(?s)<!--.*?-->").unwrap();
    re.replace_all(html, "")
}

/// Parse a source string into an AST [`File`].
pub fn parse_file(source: &str) -> ParseResult<File> {
    let source = remove_html_comments(source);
    let tokens = TemplateParser::parse(Rule::file, source.as_ref())
        .map_err(Box::new)?
        .next()
        .unwrap();
    File::build(tokens)
}
