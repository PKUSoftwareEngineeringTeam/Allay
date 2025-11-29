mod parser;

use crate::ParseResult;
use crate::ast::File;
use parser::ASTBuilder;
use pest::Parser;
use pest_derive::Parser;

/// The template parser using Pest
#[derive(Parser)]
#[grammar = "parse/allay.pest"]
struct TemplateParser;

/// Parse a source string into an AST [`File`].
pub fn parse_file(source: &str) -> ParseResult<File> {
    let tokens = TemplateParser::parse(Rule::file, source).map_err(Box::new)?.next().unwrap();
    File::build(tokens)
}
