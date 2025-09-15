use crate::ast;
use crate::error::CompileError;
use pest::Parser;
use pest_derive::Parser;

/// The template parser using Pest
#[derive(Parser)]
#[grammar = "allay.pest"]
pub(crate) struct TemplateParser;

pub(crate) fn parse_template(source: &str) -> Result<ast::File, CompileError> {
    let _tokens = TemplateParser::parse(Rule::file, source).map_err(Box::new)?;
    todo!()
}
